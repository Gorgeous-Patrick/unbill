use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BillItem {
    pub id: String,
    pub description: String,
    pub amount_cents: i64,
    pub currency_code: String,
    pub is_superseded: bool,
}

#[component]
pub fn BillRow(bill: BillItem, on_tap: Callback<String>) -> impl IntoView {
    let id = bill.id.clone();
    view! {
        <div
            class="bill-row"
            on:click=move |_| on_tap.run(id.clone())
        >
            <span class="bill-description">{bill.description.clone()}</span>
            <span class="bill-amount">
                {format!("{:.2}", bill.amount_cents as f64 / 100.0)}
                " "
                {bill.currency_code.clone()}
            </span>
        </div>
    }
}

#[component]
pub fn BillList(
    #[prop(into)] bills: Signal<Vec<BillItem>>,
    on_tap: Callback<String>,
    #[prop(optional)] on_refresh: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="bill-list">
            {move || bills.get().into_iter().map(|bill| {
                view! { <BillRow bill=bill.clone() on_tap=on_tap /> }
            }).collect_view()}
            {on_refresh.map(|cb| view! {
                <button on:click=move |_| cb.run(())>"Refresh"</button>
            })}
        </div>
    }
}
