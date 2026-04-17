use autosurgeon::{Hydrate, Reconcile};

use super::bill::{Bill, SplitMethod};

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Amendment {
    pub id: String,
    pub new_amount_cents: Option<i64>,
    pub new_description: Option<String>,
    /// To change participants, supply a new `new_split_method` — participants
    /// are always embedded in the `SplitMethod` variant. There is no separate
    /// `new_participants` field to avoid ambiguity between the two.
    pub new_split_method: Option<SplitMethod>,
    pub author_user_id: String,
    pub created_at: i64,
    pub reason: Option<String>,
}

/// User-facing input for creating an Amendment via the service layer.
#[derive(Clone, Debug)]
pub struct BillAmendment {
    pub new_amount_cents: Option<i64>,
    pub new_description: Option<String>,
    pub new_split_method: Option<SplitMethod>,
    pub author_user_id: String,
    pub reason: Option<String>,
}

/// The rendered view of a Bill after all amendments have been applied.
/// This is what frontends display — never raw `Bill` structs.
///
/// # Projection rules (see DESIGN.md §4.4)
/// - Amendments are sorted by `created_at` ascending; ties broken by `id` lexically.
/// - Each amendment field, if `Some`, overwrites the current value.
/// - `is_deleted` can only go `false → true` in v1 (tombstone is irreversible).
#[derive(Clone, Debug)]
pub struct EffectiveBill {
    pub id: String,
    pub payer_user_id: String,
    pub amount_cents: i64,
    pub description: String,
    pub participants: Vec<String>,
    pub split_method: SplitMethod,
    pub was_amended: bool,
    pub is_deleted: bool,
    pub last_modified_at: i64,
    pub history: Vec<AmendmentSummary>,
}

#[derive(Clone, Debug)]
pub struct AmendmentSummary {
    pub id: String,
    pub author_user_id: String,
    pub created_at: i64,
    pub reason: Option<String>,
}

/// Derive the participant list from a `SplitMethod`.
/// All variants are self-contained — no external participant list needed.
pub fn participants_from_split(method: &SplitMethod) -> Vec<String> {
    match method {
        SplitMethod::Equal(users) => users.clone(),
        SplitMethod::Shares(shares) => shares.iter().map(|s| s.user_id.clone()).collect(),
        SplitMethod::Exact(exacts) => exacts.iter().map(|e| e.user_id.clone()).collect(),
    }
}

impl EffectiveBill {
    /// Project a `Bill` (with its amendment log) into the effective view.
    pub fn from(bill: &Bill) -> Self {
        let mut amount_cents = bill.amount_cents;
        let mut description = bill.description.clone();
        let mut split_method = bill.split_method.clone();
        let mut is_deleted = bill.deleted;
        let mut last_modified_at = bill.created_at;
        let mut was_amended = false;

        // Sort amendments: primary key = created_at asc, secondary = id lexical asc.
        let mut sorted_amendments = bill.amendments.clone();
        sorted_amendments.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });

        let mut history = Vec::with_capacity(sorted_amendments.len());

        for amend in &sorted_amendments {
            was_amended = true;
            if let Some(v) = amend.new_amount_cents {
                amount_cents = v;
            }
            if let Some(ref v) = amend.new_description {
                description = v.clone();
            }
            if let Some(ref v) = amend.new_split_method {
                split_method = v.clone();
            }
            // Tombstone is irreversible in v1.
            // (restore_bill creates a new amendment that sets deleted = false at service level;
            //  the Amendment struct itself does not carry a `deleted` field deliberately —
            //  the Bill.deleted field is the canonical tombstone.)
            if amend.created_at > last_modified_at {
                last_modified_at = amend.created_at;
            }
            history.push(AmendmentSummary {
                id: amend.id.clone(),
                author_user_id: amend.author_user_id.clone(),
                created_at: amend.created_at,
                reason: amend.reason.clone(),
            });
        }

        // Derive participants from the (possibly amended) split method — it is the
        // single source of truth. EffectiveBill.participants is a convenience view.
        let participants = participants_from_split(&split_method);

        EffectiveBill {
            id: bill.id.clone(),
            payer_user_id: bill.payer_user_id.clone(),
            amount_cents,
            description,
            participants,
            split_method,
            was_amended,
            is_deleted,
            last_modified_at,
            history,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::bill::{Bill, SplitMethod};

    fn base_bill() -> Bill {
        Bill {
            id: "bill-1".into(),
            payer_user_id: "alice".into(),
            amount_cents: 3000,
            description: "Dinner".into(),
            participant_user_ids: vec!["alice".into(), "bob".into()],
            split_method: SplitMethod::Equal(vec!["alice".into(), "bob".into()]),
            created_at: 1000,
            created_by_device: "device-a".into(),
            deleted: false,
            amendments: vec![],
        }
    }

    fn amend(id: &str, ts: i64) -> Amendment {
        Amendment {
            id: id.into(),
            new_amount_cents: None,
            new_description: None,
            new_split_method: None,
            author_user_id: "alice".into(),
            created_at: ts,
            reason: None,
        }
    }

    #[test]
    fn test_effective_bill_no_amendments() {
        let bill = base_bill();
        let eff = EffectiveBill::from(&bill);
        assert_eq!(eff.amount_cents, 3000);
        assert_eq!(eff.description, "Dinner");
        assert!(!eff.was_amended);
        assert!(!eff.is_deleted);
        assert!(eff.history.is_empty());
        assert_eq!(eff.last_modified_at, 1000);
    }

    #[test]
    fn test_effective_bill_single_amendment_updates_fields() {
        let mut bill = base_bill();
        bill.amendments.push(Amendment {
            new_amount_cents: Some(4500),
            new_description: Some("Dinner + drinks".into()),
            ..amend("a1", 2000)
        });
        let eff = EffectiveBill::from(&bill);
        assert_eq!(eff.amount_cents, 4500);
        assert_eq!(eff.description, "Dinner + drinks");
        assert!(eff.was_amended);
        assert_eq!(eff.last_modified_at, 2000);
        assert_eq!(eff.history.len(), 1);
    }

    #[test]
    fn test_effective_bill_amendments_applied_in_timestamp_order() {
        // Later amendment wins.
        let mut bill = base_bill();
        bill.amendments.push(Amendment {
            new_amount_cents: Some(9999),
            ..amend("a2", 3000) // later
        });
        bill.amendments.push(Amendment {
            new_amount_cents: Some(4500),
            ..amend("a1", 2000) // earlier
        });
        let eff = EffectiveBill::from(&bill);
        assert_eq!(eff.amount_cents, 9999, "later amendment should win");
    }

    #[test]
    fn test_effective_bill_tie_broken_by_id_lexical() {
        // Two amendments at the same timestamp: the one with the lexically greater id wins.
        let mut bill = base_bill();
        bill.amendments.push(Amendment {
            new_amount_cents: Some(100),
            ..amend("zzz", 2000)
        });
        bill.amendments.push(Amendment {
            new_amount_cents: Some(200),
            ..amend("aaa", 2000)
        });
        // "aaa" < "zzz" lexically, so "aaa" is applied first, then "zzz" overwrites.
        let eff = EffectiveBill::from(&bill);
        assert_eq!(eff.amount_cents, 100, "lexically later id should win on tie");
    }

    #[test]
    fn test_effective_bill_partial_amendment_leaves_other_fields_unchanged() {
        let mut bill = base_bill();
        bill.amendments.push(Amendment {
            new_description: Some("Updated description".into()),
            ..amend("a1", 2000)
        });
        let eff = EffectiveBill::from(&bill);
        // amount unchanged
        assert_eq!(eff.amount_cents, 3000);
        // description updated
        assert_eq!(eff.description, "Updated description");
        // participants unchanged (derived from the unchanged Equal split method)
        assert_eq!(eff.participants, vec!["alice", "bob"]);
    }

    #[test]
    fn test_effective_bill_preserves_deleted_tombstone() {
        let mut bill = base_bill();
        bill.deleted = true;
        let eff = EffectiveBill::from(&bill);
        assert!(eff.is_deleted);
    }

    #[test]
    fn test_effective_bill_history_in_applied_order() {
        let mut bill = base_bill();
        bill.amendments.push(amend("a2", 3000));
        bill.amendments.push(amend("a1", 2000));
        let eff = EffectiveBill::from(&bill);
        assert_eq!(eff.history[0].id, "a1"); // earlier first
        assert_eq!(eff.history[1].id, "a2");
    }

    #[test]
    fn test_effective_bill_last_modified_at_is_latest_amendment_ts() {
        let mut bill = base_bill(); // created_at = 1000
        bill.amendments.push(amend("a1", 5000));
        bill.amendments.push(amend("a2", 3000));
        let eff = EffectiveBill::from(&bill);
        assert_eq!(eff.last_modified_at, 5000);
    }
}
