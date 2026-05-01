use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LedgerItem {
    pub id: String,
    pub name: String,
    pub currency_code: String,
}

#[component]
pub fn LedgerRow(ledger: LedgerItem, on_tap: Callback<String>) -> impl IntoView {
    let id = ledger.id.clone();
    view! {
        <div
            class="ledger-row"
            on:click=move |_| on_tap.run(id.clone())
        >
            <span class="ledger-name">{ledger.name.clone()}</span>
            <span class="ledger-currency">{ledger.currency_code.clone()}</span>
        </div>
    }
}

#[component]
pub fn LedgerList(
    #[prop(into)] ledgers: Signal<Vec<LedgerItem>>,
    on_tap: Callback<String>,
    #[prop(optional)] on_refresh: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="ledger-list">
            {move || ledgers.get().into_iter().map(|ledger| {
                view! { <LedgerRow ledger=ledger.clone() on_tap=on_tap /> }
            }).collect_view()}
            {on_refresh.map(|cb| view! {
                <button on:click=move |_| cb.run(())>"Refresh"</button>
            })}
        </div>
    }
}
