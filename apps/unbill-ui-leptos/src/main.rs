use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <main class="shell">
            <section class="panel">
                <p class="eyebrow">"Shared Leptos UI"</p>
                <h1 class="title">"Unbill"</h1>
                <p class="copy">
                    "This shared Unbill UI is built with Leptos + Trunk and can be consumed"
                    " by platform-specific shells like Tauri."
                </p>
                <div class="status">"Ready for shell integration"</div>
            </section>
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
