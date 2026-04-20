use leptos::{ev, prelude::*};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonTone {
    #[default]
    Primary,
    Secondary,
    Quiet,
}

impl ButtonTone {
    fn class_name(self) -> &'static str {
        match self {
            Self::Primary => "action-button-primary",
            Self::Secondary => "action-button-secondary",
            Self::Quiet => "action-button-quiet",
        }
    }
}

#[component]
pub fn ScreenFrame(
    #[prop(into)] title: String,
    #[prop(optional, into)] eyebrow: Option<String>,
    #[prop(optional, into)] subtitle: Option<String>,
    #[prop(optional)] leading: Option<AnyView>,
    #[prop(optional)] trailing: Option<AnyView>,
    children: Children,
    #[prop(optional)] footer: Option<AnyView>,
) -> impl IntoView {
    let eyebrow_view =
        eyebrow.map(|text| view! { <p class="screen-eyebrow">{text}</p> }.into_any());
    let subtitle_view =
        subtitle.map(|text| view! { <p class="screen-subtitle">{text}</p> }.into_any());
    let footer_view =
        footer.map(|content| view! { <footer class="screen-footer">{content}</footer> }.into_any());

    view! {
        <section class="screen-frame">
            <header class="screen-topbar">
                <div class="screen-leading">{leading}</div>
                <div class="screen-copy">
                    {eyebrow_view}
                    <h2 class="screen-title">{title}</h2>
                    {subtitle_view}
                </div>
                <div class="screen-trailing">{trailing}</div>
            </header>

            <div class="screen-content">{children()}</div>

            {footer_view}
        </section>
    }
}

#[component]
pub fn ModalSheet(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional)] on_close: Option<Callback<ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let description_view =
        description.map(|text| view! { <p class="sheet-description">{text}</p> }.into_any());

    view! {
        <div class="sheet-overlay">
            <div class="sheet-backdrop"></div>
            <section class="sheet-panel">
                <header class="sheet-header">
                    <div class="sheet-copy">
                        <p class="section-kicker">"Action"</p>
                        <h3 class="sheet-title">{title}</h3>
                        {description_view}
                    </div>
                    <button
                        type="button"
                        class="topbar-button"
                        on:click=move |event| {
                            if let Some(handler) = on_close.as_ref() {
                                handler.run(event);
                            }
                        }
                    >
                        "Close"
                    </button>
                </header>
                <div class="sheet-body">{children()}</div>
            </section>
        </div>
    }
}

#[component]
pub fn SectionCard(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] kicker: Option<String>,
    #[prop(optional, into)] description: Option<String>,
    children: Children,
) -> impl IntoView {
    let kicker_view = kicker.map(|text| view! { <p class="section-kicker">{text}</p> }.into_any());
    let title_view = title.map(|text| view! { <h3 class="section-title">{text}</h3> }.into_any());
    let description_view =
        description.map(|text| view! { <p class="section-description">{text}</p> }.into_any());

    view! {
        <section class="section-card">
            <div class="section-header">
                {kicker_view}
                {title_view}
                {description_view}
            </div>
            <div class="section-body">{children()}</div>
        </section>
    }
}

#[component]
pub fn ActionButton(
    #[prop(into)] label: String,
    #[prop(optional)] tone: ButtonTone,
    #[prop(optional)] full_width: bool,
    #[prop(optional)] on_press: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    let class_name = if full_width {
        format!("action-button {} action-button-block", tone.class_name())
    } else {
        format!("action-button {}", tone.class_name())
    };

    view! {
        <button
            type="button"
            class=class_name
            on:click=move |event| {
                if let Some(handler) = on_press.as_ref() {
                    handler.run(event);
                }
            }
        >
            {label}
        </button>
    }
}

#[component]
pub fn TopBarButton(
    #[prop(into)] label: String,
    #[prop(optional)] on_press: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class="topbar-button"
            on:click=move |event| {
                if let Some(handler) = on_press.as_ref() {
                    handler.run(event);
                }
            }
        >
            {label}
        </button>
    }
}

#[component]
pub fn ListRow(
    #[prop(into)] title: String,
    #[prop(optional, into)] meta: Option<String>,
    #[prop(optional, into)] detail: Option<String>,
    #[prop(optional)] trailing: Option<AnyView>,
    #[prop(optional)] selected: bool,
    #[prop(optional)] on_press: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    let meta_view = meta.map(|text| view! { <p class="row-meta">{text}</p> }.into_any());
    let detail_view = detail.map(|text| view! { <p class="row-detail">{text}</p> }.into_any());
    let class_name = if selected {
        "list-row list-row-selected"
    } else {
        "list-row"
    };

    view! {
        <button
            type="button"
            class=class_name
            on:click=move |event| {
                if let Some(handler) = on_press.as_ref() {
                    handler.run(event);
                }
            }
        >
            <div class="row-copy">
                <p class="row-title">{title}</p>
                {meta_view}
                {detail_view}
            </div>
            <div class="row-trailing">{trailing}</div>
        </button>
    }
}

#[component]
pub fn FieldBlock(
    #[prop(into)] label: String,
    #[prop(optional, into)] hint: Option<String>,
    children: Children,
) -> impl IntoView {
    let hint_view = hint.map(|text| view! { <p class="field-hint">{text}</p> }.into_any());

    view! {
        <label class="field-block">
            <span class="field-label">{label}</span>
            {hint_view}
            <div class="field-control">{children()}</div>
        </label>
    }
}

#[component]
pub fn TagPill(#[prop(into)] label: String, #[prop(optional)] active: bool) -> impl IntoView {
    let class_name = if active {
        "tag-pill tag-pill-active"
    } else {
        "tag-pill"
    };

    view! { <span class=class_name>{label}</span> }
}
