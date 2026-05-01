use leptos::prelude::*;

#[component]
pub fn StatusStrip(status: Option<String>, error: Option<String>, busy: bool) -> impl IntoView {
    let message = error.clone().or(status);
    let class_name = if message.is_some() {
        if error.is_some() {
            "status-strip status-strip-error"
        } else {
            "status-strip status-strip-info"
        }
    } else {
        "status-strip status-strip-hidden"
    };

    view! {
        <section class=class_name>
            <div class="status-copy">
                {message.unwrap_or_default()}
                {busy.then(|| view! { <span class="status-chip">"Working"</span> })}
            </div>
        </section>
    }
}
