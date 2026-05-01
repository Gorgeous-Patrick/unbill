use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettlementItem {
    pub from_user_id: String,
    pub from_display_name: String,
    pub to_user_id: String,
    pub to_display_name: String,
    pub amount_cents: i64,
    pub currency_code: String,
}

#[component]
pub fn SettlementRow(item: SettlementItem) -> impl IntoView {
    view! {
        <div class="settlement-row">
            <span class="settlement-from">{item.from_display_name.clone()}</span>
            <span class="settlement-arrow">" → "</span>
            <span class="settlement-to">{item.to_display_name.clone()}</span>
            <span class="settlement-amount">
                {format!("{:.2}", item.amount_cents as f64 / 100.0)}
                " "
                {item.currency_code.clone()}
            </span>
        </div>
    }
}

#[component]
pub fn SettlementList(#[prop(into)] items: Signal<Vec<SettlementItem>>) -> impl IntoView {
    view! {
        <div class="settlement-list">
            {move || items.get().into_iter().map(|item| {
                view! { <SettlementRow item=item.clone() /> }
            }).collect_view()}
        </div>
    }
}
