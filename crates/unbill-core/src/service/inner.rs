// UnbillService: top-level facade consumed by CLI and Tauri.

use std::collections::HashMap;
use std::sync::Arc;

use rand::TryRng as _;
use tokio::sync::broadcast;

use crate::conflict::{self, ConflictGroup};
use crate::doc::LedgerDoc;
use crate::error::{Result, UnbillError};
use crate::model::{
    Currency, Device, EffectiveBills, Invitation, InviteToken, LedgerMeta, NewBill, NewDevice,
    NewUser, NodeId, Timestamp, Ulid, User,
};
use crate::settlement;
use crate::storage::LedgerStore;
use unbill_event::ServiceEvent;
use unbill_network::{
    EndpointIdExt, load_device_labels, load_pending_invitations, save_device_labels,
    save_pending_invitations,
};

pub struct UnbillService {
    pub(crate) store: Arc<dyn LedgerStore>,
    pub(crate) device_id: NodeId,
    pub(crate) secret_key: iroh::SecretKey,
    pub(crate) events: broadcast::Sender<ServiceEvent>,
}

impl UnbillService {
    /// Open the service: load or create the device key.
    ///
    /// All store-backed data (ledgers, pending invitations, pending user
    /// tokens, local users) is loaded on demand and never cached in memory.
    pub async fn open(store: Arc<dyn LedgerStore>) -> Result<Arc<Self>> {
        let (device_id, secret_key) = load_or_create_device_key(&*store).await?;
        let (events, _) = broadcast::channel(256);
        Ok(Arc::new(Self {
            store,
            device_id,
            secret_key,
            events,
        }))
    }

    // -----------------------------------------------------------------------
    // Ledger lifecycle
    // -----------------------------------------------------------------------

    pub async fn create_ledger(&self, name: String, currency: String) -> Result<String> {
        let currency = Currency::from_code(&currency).ok_or_else(|| {
            UnbillError::Other(anyhow::anyhow!("unknown currency code: {currency}"))
        })?;
        let ledger_id = Ulid::new();
        let now = Timestamp::now();

        let mut doc = LedgerDoc::new(ledger_id, name.clone(), currency, now)?;
        doc.add_device(
            NewDevice {
                node_id: self.device_id.clone(),
            },
            now,
        )?;

        let meta = LedgerMeta {
            ledger_id,
            name,
            currency,
            created_at: now,
            updated_at: now,
        };
        let id = ledger_id.to_string();
        self.store.save_ledger_meta(&meta).await?;
        self.store.save_ledger(&id, &mut doc).await?;

        Ok(id)
    }

    pub async fn list_ledgers(&self) -> Result<Vec<LedgerMeta>> {
        Ok(self.store.list_ledgers().await?)
    }

    pub async fn delete_ledger(&self, ledger_id: &str) -> Result<()> {
        self.store.delete_ledger(ledger_id).await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Bills
    // -----------------------------------------------------------------------

    pub async fn add_bill(&self, ledger_id: &str, input: NewBill) -> Result<String> {
        let mut doc = self.load_doc(ledger_id).await?;
        let bill_id = doc.add_bill(input, self.device_id.clone(), Timestamp::now())?;
        self.store.save_ledger(ledger_id, &mut doc).await?;
        self.touch_meta(ledger_id).await?;
        Ok(bill_id.to_string())
    }

    pub async fn list_bills(&self, ledger_id: &str) -> Result<EffectiveBills> {
        self.load_doc(ledger_id).await?.list_bills()
    }

    // -----------------------------------------------------------------------
    // Users
    // -----------------------------------------------------------------------

    pub async fn add_user(&self, ledger_id: &str, input: NewUser) -> Result<()> {
        let mut doc = self.load_doc(ledger_id).await?;
        doc.add_user(input, Timestamp::now())?;
        self.store.save_ledger(ledger_id, &mut doc).await?;
        self.touch_meta(ledger_id).await?;
        Ok(())
    }

    pub async fn list_users(&self, ledger_id: &str) -> Result<Vec<User>> {
        self.load_doc(ledger_id).await?.list_users()
    }

    /// Create a brand-new user, add them to the ledger, and return the created `User`.
    pub async fn create_user(&self, ledger_id: &str, display_name: String) -> Result<User> {
        let user_id = Ulid::new();
        let now = Timestamp::now();
        let mut doc = self.load_doc(ledger_id).await?;
        doc.add_user(
            NewUser {
                user_id,
                display_name: display_name.clone(),
            },
            now,
        )?;
        self.store.save_ledger(ledger_id, &mut doc).await?;
        self.touch_meta(ledger_id).await?;
        Ok(User {
            user_id,
            display_name,
            added_at: now,
        })
    }

    /// Return all unique users across every ledger on this device, deduplicated by `user_id`.
    ///
    /// Use this to populate the user picker when adding a member to a ledger.
    pub async fn list_all_users(&self) -> Result<Vec<User>> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for meta in self.store.list_ledgers().await? {
            let id = meta.ledger_id.to_string();
            let doc = self.load_doc(&id).await?;
            for user in doc.list_users()? {
                if seen.insert(user.user_id) {
                    result.push(user);
                }
            }
        }
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Devices
    // -----------------------------------------------------------------------

    pub async fn add_device(&self, ledger_id: &str, input: NewDevice) -> Result<()> {
        let mut doc = self.load_doc(ledger_id).await?;
        doc.add_device(input, Timestamp::now())?;
        self.store.save_ledger(ledger_id, &mut doc).await?;
        self.touch_meta(ledger_id).await?;
        Ok(())
    }

    pub async fn list_devices(&self, ledger_id: &str) -> Result<Vec<Device>> {
        self.load_doc(ledger_id).await?.list_devices()
    }

    pub async fn list_device_labels(&self) -> Result<HashMap<String, String>> {
        load_device_labels(&*self.store).await
    }

    pub async fn set_device_label(&self, node_id: NodeId, label: String) -> Result<()> {
        let mut labels = load_device_labels(&*self.store).await?;
        let key = node_id.to_string();
        let trimmed = label.trim();
        if trimmed.is_empty() {
            labels.remove(&key);
        } else {
            labels.insert(key, trimmed.to_owned());
        }
        save_device_labels(&*self.store, &labels).await
    }

    // -----------------------------------------------------------------------
    // Settlement
    // -----------------------------------------------------------------------

    /// Compute net settlement for a user across all ledgers they participate in,
    /// grouped by currency. Returns one `Settlement` per currency.
    pub async fn compute_settlement_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<settlement::Settlement>> {
        let user_ulid = parse_ulid(user_id)?;
        let mut by_currency: std::collections::HashMap<
            Currency,
            std::collections::HashMap<crate::model::Ulid, i64>,
        > = std::collections::HashMap::new();

        for meta in self.store.list_ledgers().await? {
            let id = meta.ledger_id.to_string();
            let doc = self.load_doc(&id).await?;
            let users = doc.list_users()?;
            // Only aggregate ledgers where this user is active.
            if users.iter().any(|user| user.user_id == user_ulid) {
                let currency = doc.get_ledger()?.currency;
                let bills = doc.list_bills()?;
                let balances = by_currency.entry(currency).or_default();
                settlement::accumulate_balances(&users, &bills, balances);
            }
        }

        let mut results = Vec::new();
        for (currency, balances) in by_currency {
            let full = settlement::compute_from_balances(currency, balances);
            let transactions: Vec<_> = full
                .transactions
                .into_iter()
                .filter(|t| t.from_user_id == user_ulid || t.to_user_id == user_ulid)
                .collect();
            if !transactions.is_empty() {
                results.push(settlement::Settlement {
                    currency,
                    transactions,
                });
            }
        }
        Ok(results)
    }

    /// Compute settlement for a single ledger.
    pub async fn settle_ledger(
        &self,
        ledger_id: &str,
    ) -> crate::error::Result<settlement::Settlement> {
        let doc = self.load_doc(ledger_id).await?;
        let ledger = doc.get_ledger()?;
        Ok(settlement::compute_settlement(&ledger))
    }

    // -----------------------------------------------------------------------
    // Conflict detection
    // -----------------------------------------------------------------------

    pub async fn detect_conflicts(&self, ledger_id: &str) -> Result<Vec<ConflictGroup>> {
        let doc = self.load_doc(ledger_id).await?;
        let all_bills = doc.list_all_bills()?;
        Ok(conflict::detect(&all_bills))
    }

    // -----------------------------------------------------------------------
    // Invitations and sync
    // -----------------------------------------------------------------------

    /// Generate a join invite URL for `ledger_id` and store the pending invitation.
    ///
    /// URL format: `unbill://join/<ledger_id>/<host_node_id>/<token_hex>`
    pub async fn create_invitation(&self, ledger_id: &str) -> Result<String> {
        let ledger_ulid = parse_ulid(ledger_id)?;
        // Check the ledger exists locally.
        let _ = self.load_doc(ledger_id).await?;
        let token = InviteToken::generate();
        let now = Timestamp::now();
        let invitation = Invitation {
            token: token.clone(),
            ledger_id: ledger_ulid,
            created_by_device: self.device_id.clone(),
            created_at: now,
            expires_at: Timestamp::from_millis(now.as_millis() + 24 * 3600 * 1000),
        };
        {
            let mut map = load_pending_invitations(&*self.store).await?;
            map.insert(token.to_string(), invitation);
            save_pending_invitations(&*self.store, &map).await?;
        }
        Ok(format!(
            "unbill://join/{}/{}/{}",
            ledger_id, self.device_id, token
        ))
    }

    /// Accept a join invite URL and join the ledger hosted by the inviting device.
    ///
    /// URL format: `unbill://join/<ledger_id>/<host_node_id>/<token_hex>`
    /// `label` is an optional device-local nickname for the host device.
    pub async fn join_ledger(self: &Arc<Self>, url: &str, label: String) -> Result<()> {
        use crate::net::{JoinRequest, UnbillEndpoint};
        let (ledger_id, host, token) = parse_join_url(url)?;
        let local_label = (!label.trim().is_empty()).then_some(label.trim().to_owned());
        let request = JoinRequest { token, ledger_id };
        let ep = UnbillEndpoint::bind(self.secret_key.clone())
            .await
            .map_err(UnbillError::Other)?;
        let result = ep
            .join_ledger_inner(host, local_label, request, &self.store, &self.events)
            .await;
        ep.close().await;
        result.map_err(UnbillError::Other)
    }

    /// Dial `peer` and run the full sync exchange for all shared ledgers.
    pub async fn sync_once(self: &Arc<Self>, peer: NodeId) -> Result<()> {
        use crate::net::UnbillEndpoint;
        let ep = UnbillEndpoint::bind(self.secret_key.clone())
            .await
            .map_err(UnbillError::Other)?;
        let result = ep.sync_once_inner(peer, &self.store, &self.events).await;
        ep.close().await;
        result.map_err(UnbillError::Other)
    }

    /// Open an endpoint and accept incoming sync/join/user-transfer connections until
    /// an error occurs or the process is interrupted.
    ///
    /// Prints the local `NodeId` to stdout so peers know what to dial.
    pub async fn accept_loop(self: &Arc<Self>) -> Result<()> {
        use crate::net::UnbillEndpoint;
        let ep = UnbillEndpoint::bind(self.secret_key.clone())
            .await
            .map_err(UnbillError::Other)?;
        ep.wait_for_ready().await;
        println!("listening on: {}", ep.node_id());
        let result = ep
            .accept_loop_inner(Arc::clone(&self.store), self.events.clone())
            .await;
        ep.close().await;
        result.map_err(UnbillError::Other)
    }

    // -----------------------------------------------------------------------
    // Events
    // -----------------------------------------------------------------------

    pub fn device_id(&self) -> NodeId {
        self.device_id.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServiceEvent> {
        self.events.subscribe()
    }

    // -----------------------------------------------------------------------
    // Internals
    // -----------------------------------------------------------------------

    async fn load_doc(&self, ledger_id: &str) -> Result<LedgerDoc> {
        self.store
            .load_ledger(ledger_id)
            .await?
            .ok_or_else(|| UnbillError::LedgerNotFound(ledger_id.to_string()))
    }

    /// Update `updated_at` in the stored metadata for a ledger.
    async fn touch_meta(&self, ledger_id: &str) -> Result<()> {
        let mut metas = self.store.list_ledgers().await?;
        if let Some(meta) = metas
            .iter_mut()
            .find(|m| m.ledger_id.to_string() == ledger_id)
        {
            meta.updated_at = Timestamp::now();
            self.store.save_ledger_meta(meta).await?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn load_or_create_device_key(store: &dyn LedgerStore) -> Result<(NodeId, iroh::SecretKey)> {
    if let Some(bytes) = store.load_device_meta("device_key.bin").await? {
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| UnbillError::Other(anyhow::anyhow!("device_key.bin: wrong length")))?;
        let secret = iroh::SecretKey::from(arr);
        Ok((secret.public().to_node_id(), secret))
    } else {
        let mut arr = [0u8; 32];
        rand::rngs::SysRng
            .try_fill_bytes(&mut arr)
            .expect("system RNG should generate device keys");
        let secret = iroh::SecretKey::from(arr);
        store.save_device_meta("device_key.bin", &arr).await?;
        Ok((secret.public().to_node_id(), secret))
    }
}

fn parse_ulid(s: &str) -> Result<Ulid> {
    Ulid::from_string(s).map_err(|e| UnbillError::Other(anyhow::anyhow!("invalid ULID {s:?}: {e}")))
}

/// Parse `unbill://join/<ledger_id>/<host_node_id>/<token_hex>`.
fn parse_join_url(url: &str) -> Result<(String, NodeId, String)> {
    let path = url
        .strip_prefix("unbill://join/")
        .ok_or_else(|| UnbillError::Other(anyhow::anyhow!("invalid join URL: {url:?}")))?;
    let parts: Vec<&str> = path.splitn(3, '/').collect();
    if parts.len() != 3 {
        return Err(UnbillError::Other(anyhow::anyhow!(
            "invalid join URL (expected ledger_id/host_node_id/token): {url:?}"
        )));
    }
    let ledger_id = parts[0].to_string();
    let host = parts[1]
        .parse::<NodeId>()
        .map_err(|e| UnbillError::Other(anyhow::anyhow!("invalid host node ID in URL: {e}")))?;
    let token = parts[2].to_string();
    Ok((ledger_id, host, token))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Share;
    use unbill_store_memory::InMemoryStore;

    fn mem_store() -> Arc<dyn LedgerStore> {
        Arc::new(InMemoryStore::default())
    }

    async fn open() -> Arc<UnbillService> {
        UnbillService::open(mem_store()).await.unwrap()
    }

    fn usd() -> &'static str {
        "USD"
    }

    fn two_way_bill(desc: &str, amount_cents: i64, ledger_id: &str) -> (String, NewBill) {
        let _ = ledger_id;
        let bill = NewBill {
            amount_cents,
            description: desc.to_owned(),
            payers: vec![Share {
                user_id: Ulid::from_u128(1),
                shares: 1,
            }],
            payees: vec![
                Share {
                    user_id: Ulid::from_u128(1),
                    shares: 1,
                },
                Share {
                    user_id: Ulid::from_u128(2),
                    shares: 1,
                },
            ],
            prev: vec![],
        };
        (ledger_id.to_owned(), bill)
    }

    async fn seed_users(svc: &UnbillService, ledger_id: &str) {
        for (n, name) in [(1u128, "Alice"), (2, "Bob")] {
            svc.add_user(
                ledger_id,
                NewUser {
                    user_id: Ulid::from_u128(n),
                    display_name: name.into(),
                },
            )
            .await
            .unwrap();
        }
    }

    // --- create / list / delete ledger ---

    #[tokio::test]
    async fn test_create_ledger_appears_in_list() {
        let svc = open().await;
        let id = svc
            .create_ledger("Household".into(), usd().into())
            .await
            .unwrap();
        let ledgers = svc.list_ledgers().await.unwrap();
        assert_eq!(ledgers.len(), 1);
        assert_eq!(ledgers[0].ledger_id.to_string(), id);
        assert_eq!(ledgers[0].name, "Household");
        assert_eq!(ledgers[0].currency.code(), "USD");
    }

    #[tokio::test]
    async fn test_delete_ledger_removes_it() {
        let svc = open().await;
        let id = svc
            .create_ledger("Trip".into(), usd().into())
            .await
            .unwrap();
        svc.delete_ledger(&id).await.unwrap();
        assert!(svc.list_ledgers().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_unknown_ledger_returns_not_found() {
        let svc = open().await;
        let result = svc.list_bills("00000000000000000000000000").await;
        assert!(matches!(result, Err(UnbillError::LedgerNotFound(_))));
    }

    #[tokio::test]
    async fn test_invalid_currency_returns_error() {
        let svc = open().await;
        let result = svc.create_ledger("Bad".into(), "ZZZ".into()).await;
        assert!(result.is_err());
    }

    // --- bills ---

    #[tokio::test]
    async fn test_add_bill_and_list() {
        let svc = open().await;
        let lid = svc
            .create_ledger("Test".into(), usd().into())
            .await
            .unwrap();
        seed_users(&svc, &lid).await;
        let (_, bill) = two_way_bill("Dinner", 6000, &lid);
        let bill_id = svc.add_bill(&lid, bill).await.unwrap();

        let bills = svc.list_bills(&lid).await.unwrap();
        assert_eq!(bills.0.len(), 1);
        assert_eq!(bills.0[0].id.to_string(), bill_id);
        assert_eq!(bills.0[0].amount_cents, 6000);
    }

    #[tokio::test]
    async fn test_amend_bill_supersedes_original() {
        let svc = open().await;
        let lid = svc
            .create_ledger("Test".into(), usd().into())
            .await
            .unwrap();
        seed_users(&svc, &lid).await;
        let (_, bill) = two_way_bill("Lunch", 3000, &lid);
        let original_id = svc.add_bill(&lid, bill).await.unwrap();

        let amendment_id = svc
            .add_bill(
                &lid,
                NewBill {
                    amount_cents: 4000,
                    description: "Lunch".into(),
                    payers: vec![Share {
                        user_id: Ulid::from_u128(1),
                        shares: 1,
                    }],
                    payees: vec![
                        Share {
                            user_id: Ulid::from_u128(1),
                            shares: 1,
                        },
                        Share {
                            user_id: Ulid::from_u128(2),
                            shares: 1,
                        },
                    ],
                    prev: vec![Ulid::from_string(&original_id).unwrap()],
                },
            )
            .await
            .unwrap();

        let bills = svc.list_bills(&lid).await.unwrap();
        assert_eq!(bills.0.len(), 1, "original should be superseded");
        assert_eq!(bills.0[0].id.to_string(), amendment_id);
        assert_eq!(bills.0[0].amount_cents, 4000);
    }

    // --- settlement ---

    #[tokio::test]
    async fn test_compute_settlement_no_bills_is_empty() {
        let svc = open().await;
        let lid = svc
            .create_ledger("Empty".into(), usd().into())
            .await
            .unwrap();
        seed_users(&svc, &lid).await;
        let alice = Ulid::from_u128(1).to_string();
        let s = svc.compute_settlement_for_user(&alice).await.unwrap();
        assert!(s.iter().all(|g| g.transactions.is_empty()));
    }

    #[tokio::test]
    async fn test_compute_settlement_cross_ledger() {
        let svc = open().await;

        // Ledger 1: Alice pays $60, Alice+Bob split → Bob owes Alice $30.
        let lid1 = svc.create_ledger("L1".into(), usd().into()).await.unwrap();
        seed_users(&svc, &lid1).await;
        let (_, bill1) = two_way_bill("Rent", 6000, &lid1);
        svc.add_bill(&lid1, bill1).await.unwrap();

        // Ledger 2: Bob pays $20, Alice+Bob split → Alice owes Bob $10.
        let lid2 = svc.create_ledger("L2".into(), usd().into()).await.unwrap();
        seed_users(&svc, &lid2).await;
        let bob_pays = NewBill {
            amount_cents: 2000,
            description: "Utilities".into(),
            payers: vec![Share {
                user_id: Ulid::from_u128(2),
                shares: 1,
            }],
            payees: vec![
                Share {
                    user_id: Ulid::from_u128(1),
                    shares: 1,
                },
                Share {
                    user_id: Ulid::from_u128(2),
                    shares: 1,
                },
            ],
            prev: vec![],
        };
        svc.add_bill(&lid2, bob_pays).await.unwrap();

        // Net: Bob owes Alice $30 - $10 = $20. Both ledgers are USD → one group.
        let alice = Ulid::from_u128(1).to_string();
        let settlements = svc.compute_settlement_for_user(&alice).await.unwrap();
        assert_eq!(settlements.len(), 1);
        let s = &settlements[0];
        assert_eq!(s.currency.code(), "USD");
        assert_eq!(s.transactions.len(), 1);
        assert_eq!(s.transactions[0].amount_cents, 2000);
        assert_eq!(s.transactions[0].from_user_id, Ulid::from_u128(2));
        assert_eq!(s.transactions[0].to_user_id, Ulid::from_u128(1));
    }

    #[tokio::test]
    async fn test_settlement_groups_separate_currencies() {
        let svc = open().await;

        // USD ledger: Alice pays $60, Alice+Bob split → Bob owes Alice $30.
        let lid_usd = svc
            .create_ledger("USD ledger".into(), "USD".into())
            .await
            .unwrap();
        seed_users(&svc, &lid_usd).await;
        let (_, usd_bill) = two_way_bill("Rent", 6000, &lid_usd);
        svc.add_bill(&lid_usd, usd_bill).await.unwrap();

        // EUR ledger: Alice pays €40, Alice+Bob split → Bob owes Alice €20.
        let lid_eur = svc
            .create_ledger("EUR ledger".into(), "EUR".into())
            .await
            .unwrap();
        seed_users(&svc, &lid_eur).await;
        let (_, eur_bill) = two_way_bill("Hotel", 4000, &lid_eur);
        svc.add_bill(&lid_eur, eur_bill).await.unwrap();

        let alice = Ulid::from_u128(1).to_string();
        let mut settlements = svc.compute_settlement_for_user(&alice).await.unwrap();

        // Must produce two separate groups — one USD, one EUR — not a single merged total.
        assert_eq!(settlements.len(), 2);
        settlements.sort_by_key(|s| s.currency.code());

        let eur = &settlements[0];
        assert_eq!(eur.currency.code(), "EUR");
        assert_eq!(eur.transactions.len(), 1);
        assert_eq!(eur.transactions[0].amount_cents, 2000);

        let usd = &settlements[1];
        assert_eq!(usd.currency.code(), "USD");
        assert_eq!(usd.transactions.len(), 1);
        assert_eq!(usd.transactions[0].amount_cents, 3000);
    }

    // --- persistence round-trip ---

    #[tokio::test]
    async fn test_ledger_survives_service_restart() {
        let store = mem_store();
        let lid = {
            let svc = UnbillService::open(Arc::clone(&store)).await.unwrap();
            let lid = svc
                .create_ledger("Persistent".into(), usd().into())
                .await
                .unwrap();
            seed_users(&svc, &lid).await;
            let (_, bill) = two_way_bill("Rent", 120000, &lid);
            svc.add_bill(&lid, bill).await.unwrap();
            lid
        };
        // Re-open with the same store (simulates a restart).
        let svc2 = UnbillService::open(Arc::clone(&store)).await.unwrap();
        let bills = svc2.list_bills(&lid).await.unwrap();
        assert_eq!(bills.0.len(), 1);
        assert_eq!(bills.0[0].amount_cents, 120000);
    }

    #[tokio::test]
    async fn test_device_key_stable_across_restarts() {
        let store = mem_store();
        let svc1 = UnbillService::open(Arc::clone(&store)).await.unwrap();
        let svc2 = UnbillService::open(Arc::clone(&store)).await.unwrap();
        assert_eq!(svc1.device_id, svc2.device_id);
    }

    #[tokio::test]
    async fn test_device_labels_survive_restart() {
        let store = mem_store();
        let peer = NodeId::from_seed(9);
        {
            let svc = UnbillService::open(Arc::clone(&store)).await.unwrap();
            svc.set_device_label(peer.clone(), "Kitchen iPad".into())
                .await
                .unwrap();
        }
        let svc2 = UnbillService::open(Arc::clone(&store)).await.unwrap();
        let labels = svc2.list_device_labels().await.unwrap();
        assert_eq!(
            labels.get(&peer.to_string()).map(String::as_str),
            Some("Kitchen iPad")
        );
    }

    // --- create_user / list_all_users ---

    #[tokio::test]
    async fn test_create_user_appears_in_ledger() {
        let svc = open().await;
        let lid = svc
            .create_ledger("Trip".into(), usd().into())
            .await
            .unwrap();
        let user = svc.create_user(&lid, "Alice".into()).await.unwrap();
        let list = svc.list_users(&lid).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].user_id, user.user_id);
        assert_eq!(list[0].display_name, "Alice");
    }

    #[tokio::test]
    async fn test_list_all_users_aggregates_across_ledgers() {
        let svc = open().await;
        let lid1 = svc.create_ledger("L1".into(), usd().into()).await.unwrap();
        let lid2 = svc.create_ledger("L2".into(), usd().into()).await.unwrap();
        svc.create_user(&lid1, "Alice".into()).await.unwrap();
        svc.create_user(&lid2, "Bob".into()).await.unwrap();
        let all = svc.list_all_users().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_list_all_users_deduplicates_by_user_id() {
        let svc = open().await;
        let lid1 = svc.create_ledger("L1".into(), usd().into()).await.unwrap();
        let lid2 = svc.create_ledger("L2".into(), usd().into()).await.unwrap();
        // Create Alice in L1, then add the same user to L2.
        let alice = svc.create_user(&lid1, "Alice".into()).await.unwrap();
        svc.add_user(
            &lid2,
            NewUser {
                user_id: alice.user_id,
                display_name: "Alice".into(),
            },
        )
        .await
        .unwrap();
        let all = svc.list_all_users().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].user_id, alice.user_id);
    }
}
