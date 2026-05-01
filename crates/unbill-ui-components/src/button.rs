use leptos::prelude::*;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonTone {
    #[default]
    Primary,
    Secondary,
    Quiet,
}

impl ButtonTone {
    pub fn class_name(self) -> &'static str {
        match self {
            Self::Primary => "action-button-primary",
            Self::Secondary => "action-button-secondary",
            Self::Quiet => "action-button-quiet",
        }
    }
}

#[component]
pub fn ActionButton(
    #[prop(into)] label: String,
    #[prop(default = ButtonTone::Primary)] tone: ButtonTone,
    on_click: Callback<()>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let class = tone.class_name();
    view! {
        <button
            class=class
            disabled=disabled
            on:click=move |_| on_click.run(())
        >
            {label}
        </button>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IconButtonKind {
    More,
    Back,
    Close,
    Sync,
    CopyUrl,
    Add,
    Save,
}

impl IconButtonKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::More => "More",
            Self::Back => "Back",
            Self::Close => "Close",
            Self::Sync => "Sync",
            Self::CopyUrl => "Copy URL",
            Self::Add => "Add",
            Self::Save => "Save",
        }
    }
}

#[component]
pub fn IconButton(
    kind: IconButtonKind,
    on_click: Callback<()>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let label = kind.label();
    view! {
        <button
            aria-label=label
            disabled=disabled
            on:click=move |_| on_click.run(())
        >
            {label}
        </button>
    }
}
