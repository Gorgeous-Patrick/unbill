use leptos::prelude::*;

#[component]
pub fn TextInput(
    #[prop(into)] value: Signal<String>,
    on_change: Callback<String>,
    #[prop(into, default = String::new())] placeholder: String,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    view! {
        <input
            type="text"
            prop:value=value
            placeholder=placeholder
            disabled=disabled
            on:input=move |ev| on_change.run(event_target_value(&ev))
        />
    }
}

#[component]
pub fn AmountInput(
    /// Value in cents.
    #[prop(into)]
    value_cents: Signal<i64>,
    on_change: Callback<i64>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let display = move || {
        let cents = value_cents.get();
        format!("{:.2}", cents as f64 / 100.0)
    };

    view! {
        <input
            type="number"
            step="0.01"
            min="0"
            prop:value=display
            disabled=disabled
            on:input=move |ev| {
                let raw = event_target_value(&ev);
                if let Ok(f) = raw.parse::<f64>() {
                    on_change.run((f * 100.0).round() as i64);
                }
            }
        />
    }
}
