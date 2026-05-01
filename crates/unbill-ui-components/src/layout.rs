use leptos::prelude::*;

#[component]
pub fn Page(children: Children) -> impl IntoView {
    view! {
        <div class="page">
            {children()}
        </div>
    }
}

#[component]
pub fn SafeAreaContainer(children: Children) -> impl IntoView {
    view! {
        <div class="safe-area-container">
            {children()}
        </div>
    }
}

/// A bottom sheet / drawer overlay.
///
/// The wrapper is always in the DOM so CSS transitions can animate it in/out.
/// Add a `hidden` style to `.sheet-wrapper.hidden` in your stylesheet.
#[component]
pub fn Sheet(
    #[prop(into)] open: Signal<bool>,
    on_close: Callback<()>,
    children: Children,
) -> impl IntoView {
    let children = children();
    view! {
        <div class="sheet-wrapper" class:hidden=move || !open.get()>
            <div class="sheet-backdrop" on:click=move |_| on_close.run(())></div>
            <div class="sheet">
                {children}
            </div>
        </div>
    }
}
