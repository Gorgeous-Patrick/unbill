use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use unbill_core::model::{NewBill, NewUser, Ulid};
use unbill_core::service::UnbillService;
use wasm_bindgen_futures::JsFuture;

thread_local! {
    static SERVICE: RefCell<Option<Arc<UnbillService>>> = const { RefCell::new(None) };
}

pub fn init(service: Arc<UnbillService>) {
    SERVICE.with(|cell| *cell.borrow_mut() = Some(service));
}

fn get_service() -> Result<Arc<UnbillService>, String> {
    SERVICE.with(|cell| {
        cell.borrow()
            .clone()
            .ok_or_else(|| "service not initialized".to_owned())
    })
}

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppBootstrap {
    pub device_id: String,
    pub ledgers: Vec<LedgerSummary>,
    pub all_users: Vec<User>,
    pub devices: Vec<SyncDevice>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LedgerSummary {
    pub ledger_id: String,
    pub name: String,
    pub currency: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub user_count: usize,
    pub latest_bill_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LedgerDetail {
    pub summary: LedgerSummary,
    pub users: Vec<User>,
    pub devices: Vec<SyncDevice>,
    pub bills: Vec<Bill>,
    pub settlement: Vec<Transaction>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub from_name: String,
    pub to_name: String,
    pub amount_cents: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncDevice {
    pub node_id: String,
    pub label: String,
    pub ledger_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub user_id: String,
    pub display_name: String,
    pub added_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Share {
    pub user_id: String,
    pub shares: u32,
    pub display_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Bill {
    pub id: String,
    pub amount_cents: i64,
    pub description: String,
    pub created_at_ms: i64,
    pub payers: Vec<Share>,
    pub payees: Vec<Share>,
    pub prev: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLedgerInput {
    pub name: String,
    pub currency: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserInput {
    pub ledger_id: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddUserInput {
    pub ledger_id: String,
    pub user_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBillInput {
    pub ledger_id: String,
    pub description: String,
    pub amount_cents: i64,
    pub payers: Vec<BillShareInput>,
    pub payees: Vec<BillShareInput>,
    pub prev_bill_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BillShareInput {
    pub user_id: String,
    pub shares: u32,
}

// ---------------------------------------------------------------------------
// API functions
// ---------------------------------------------------------------------------

pub fn subscribe() -> tokio::sync::broadcast::Receiver<unbill_core::service::ServiceEvent> {
    get_service().expect("service not initialized").subscribe()
}

pub async fn load_ledger_summary(ledger_id: &str) -> Result<LedgerSummary, String> {
    let svc = get_service()?;
    let meta = svc
        .list_ledgers()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|item| item.ledger_id.to_string() == ledger_id)
        .ok_or_else(|| format!("ledger {ledger_id} not found"))?;
    summarize_ledger(&svc, meta).await
}

pub async fn load_all_users() -> Result<Vec<User>, String> {
    let svc = get_service()?;
    svc.list_all_users()
        .await
        .map_err(|e| e.to_string())
        .map(|users| users.into_iter().map(user_to_dto).collect())
}

pub async fn bootstrap_app() -> Result<AppBootstrap, String> {
    let svc = get_service()?;
    let metas = svc.list_ledgers().await.map_err(|e| e.to_string())?;
    let mut ledgers = Vec::with_capacity(metas.len());
    for meta in metas {
        ledgers.push(summarize_ledger(&svc, meta).await?);
    }
    let all_users = svc
        .list_all_users()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(user_to_dto)
        .collect();
    Ok(AppBootstrap {
        device_id: svc.device_id().to_string(),
        ledgers,
        all_users,
        devices: vec![],
    })
}

pub async fn create_ledger(input: CreateLedgerInput) -> Result<LedgerSummary, String> {
    let svc = get_service()?;
    let ledger_id = svc
        .create_ledger(input.name, input.currency)
        .await
        .map_err(|e| e.to_string())?;
    load_ledger_detail(&ledger_id)
        .await
        .map(|detail| detail.summary)
}

pub async fn load_ledger_detail(ledger_id: &str) -> Result<LedgerDetail, String> {
    let svc = get_service()?;
    let meta = svc
        .list_ledgers()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|item| item.ledger_id.to_string() == ledger_id)
        .ok_or_else(|| format!("ledger {ledger_id} not found"))?;

    let summary = summarize_ledger(&svc, meta).await?;
    let local_node_id = svc.device_id().to_string();
    let device_labels = svc.list_device_labels().await.map_err(|e| e.to_string())?;
    let devices = load_devices_for_ledger(&svc, ledger_id, &local_node_id, &device_labels).await?;
    let users = svc.list_users(ledger_id).await.map_err(|e| e.to_string())?;
    let bills = svc.list_bills(ledger_id).await.map_err(|e| e.to_string())?;

    let user_name_lookup: HashMap<Ulid, String> = users
        .iter()
        .map(|u| (u.user_id, u.display_name.clone()))
        .collect();

    let settlement = svc
        .settle_ledger(ledger_id)
        .await
        .map_err(|e| e.to_string())?
        .transactions
        .into_iter()
        .map(|t| Transaction {
            from_name: user_name_lookup
                .get(&t.from_user_id)
                .cloned()
                .unwrap_or_else(|| t.from_user_id.to_string()),
            to_name: user_name_lookup
                .get(&t.to_user_id)
                .cloned()
                .unwrap_or_else(|| t.to_user_id.to_string()),
            amount_cents: t.amount_cents,
        })
        .collect();

    let user_dtos = users.into_iter().map(user_to_dto).collect();
    let bill_dtos = map_bills(bills, &user_name_lookup);

    Ok(LedgerDetail {
        summary,
        users: user_dtos,
        devices,
        bills: bill_dtos,
        settlement,
    })
}

pub async fn create_user(input: CreateUserInput) -> Result<User, String> {
    let svc = get_service()?;
    svc.create_user(&input.ledger_id, input.display_name)
        .await
        .map(user_to_dto)
        .map_err(|e| e.to_string())
}

pub async fn add_user(input: AddUserInput) -> Result<User, String> {
    let svc = get_service()?;
    let user_id = parse_ulid(&input.user_id)?;
    let existing = svc
        .list_all_users()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|u| u.user_id == user_id)
        .ok_or_else(|| format!("user not found: {}", input.user_id))?;
    svc.add_user(
        &input.ledger_id,
        NewUser {
            user_id,
            display_name: existing.display_name,
        },
    )
    .await
    .map_err(|e| e.to_string())?;
    let added = svc
        .list_users(&input.ledger_id)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|u| u.user_id == user_id)
        .ok_or_else(|| "new user missing after add".to_owned())?;
    Ok(user_to_dto(added))
}

pub async fn create_invitation(ledger_id: &str) -> Result<String, String> {
    let svc = get_service()?;
    svc.create_invitation(ledger_id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn save_bill(input: SaveBillInput) -> Result<String, String> {
    let svc = get_service()?;
    let payers = input
        .payers
        .into_iter()
        .map(|item| {
            parse_ulid(&item.user_id).map(|user_id| unbill_core::model::Share {
                user_id,
                shares: item.shares,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let payees = input
        .payees
        .into_iter()
        .map(|item| {
            parse_ulid(&item.user_id).map(|user_id| unbill_core::model::Share {
                user_id,
                shares: item.shares,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let prev = input
        .prev_bill_id
        .into_iter()
        .map(|id| parse_ulid(&id))
        .collect::<Result<Vec<_>, _>>()?;
    svc.add_bill(
        &input.ledger_id,
        NewBill {
            amount_cents: input.amount_cents,
            description: input.description,
            payers,
            payees,
            prev,
        },
    )
    .await
    .map_err(|e| e.to_string())
}

pub async fn write_clipboard_text(text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("no browser window")?;
    let clipboard = window.navigator().clipboard();
    JsFuture::from(clipboard.write_text(text))
        .await
        .map(|_| ())
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "clipboard write failed".to_owned())
        })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn summarize_ledger(
    svc: &Arc<UnbillService>,
    meta: unbill_core::model::LedgerMeta,
) -> Result<LedgerSummary, String> {
    let ledger_id = meta.ledger_id.to_string();
    let users = svc
        .list_users(&ledger_id)
        .await
        .map_err(|e| e.to_string())?;
    let bills = svc
        .list_bills(&ledger_id)
        .await
        .map_err(|e| e.to_string())?;
    let latest_bill_at_ms = bills.iter().map(|bill| bill.created_at.as_millis()).max();
    Ok(LedgerSummary {
        ledger_id,
        name: meta.name,
        currency: meta.currency.code().to_owned(),
        created_at_ms: meta.created_at.as_millis(),
        updated_at_ms: meta.updated_at.as_millis(),
        user_count: users.len(),
        latest_bill_at_ms,
    })
}

async fn load_devices_for_ledger(
    svc: &Arc<UnbillService>,
    ledger_id: &str,
    local_node_id: &str,
    device_labels: &HashMap<String, String>,
) -> Result<Vec<SyncDevice>, String> {
    let devices = svc
        .list_devices(ledger_id)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter_map(|device| {
            let node_id = device.node_id.to_string();
            if node_id == local_node_id {
                return None;
            }
            Some(SyncDevice {
                label: device_labels.get(&node_id).cloned().unwrap_or_default(),
                node_id,
                ledger_names: vec![],
            })
        })
        .collect();
    Ok(devices)
}

fn map_bills(
    bills: unbill_core::model::EffectiveBills,
    user_lookup: &HashMap<Ulid, String>,
) -> Vec<Bill> {
    let mut items = bills
        .into_vec()
        .into_iter()
        .map(|bill| {
            let to_share = |share: unbill_core::model::Share| Share {
                display_name: user_lookup
                    .get(&share.user_id)
                    .cloned()
                    .unwrap_or_else(|| share.user_id.to_string()),
                user_id: share.user_id.to_string(),
                shares: share.shares,
            };
            Bill {
                id: bill.id.to_string(),
                amount_cents: bill.amount_cents,
                description: bill.description,
                created_at_ms: bill.created_at.as_millis(),
                payers: bill.payers.into_iter().map(to_share).collect(),
                payees: bill.payees.into_iter().map(to_share).collect(),
                prev: bill.prev.into_iter().map(|p| p.to_string()).collect(),
            }
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|item| std::cmp::Reverse(item.created_at_ms));
    items
}

fn user_to_dto(user: unbill_core::model::User) -> User {
    User {
        user_id: user.user_id.to_string(),
        display_name: user.display_name,
        added_at_ms: user.added_at.as_millis(),
    }
}

fn parse_ulid(value: &str) -> Result<Ulid, String> {
    Ulid::from_string(value).map_err(|e| format!("invalid ULID {value:?}: {e}"))
}

pub fn format_money(amount_cents: i64, currency: &str) -> String {
    let sign = if amount_cents < 0 { "-" } else { "" };
    let absolute = amount_cents.abs();
    let units = absolute / 100;
    let cents = absolute % 100;
    format!("{sign}{currency} {units}.{cents:02}")
}

pub fn format_timestamp(timestamp_ms: i64) -> String {
    let seconds = timestamp_ms / 1000;
    let day = seconds / 86_400;
    format!("day {day}")
}
