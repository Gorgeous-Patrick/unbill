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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IconButtonKind {
    More,
    Back,
    Close,
    Sync,
    Share,
    CopyUrl,
    Add,
    Save,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IconPrimitive {
    Path(&'static str),
    Circle {
        cx: &'static str,
        cy: &'static str,
        r: &'static str,
    },
    Rect {
        width: &'static str,
        height: &'static str,
        x: &'static str,
        y: &'static str,
        rx: &'static str,
        ry: &'static str,
    },
}

impl IconPrimitive {
    fn into_view(self) -> AnyView {
        match self {
            Self::Path(d) => view! { <path d=d /> }.into_any(),
            Self::Circle { cx, cy, r } => view! { <circle cx=cx cy=cy r=r /> }.into_any(),
            Self::Rect {
                width,
                height,
                x,
                y,
                rx,
                ry,
            } => view! {
                <rect width=width height=height x=x y=y rx=rx ry=ry />
            }
            .into_any(),
        }
    }
}

impl IconButtonKind {
    fn label(self) -> &'static str {
        match self {
            Self::More => "More",
            Self::Back => "Back",
            Self::Close => "Close",
            Self::Sync => "Sync",
            Self::Share => "Share",
            Self::CopyUrl => "Copy URL",
            Self::Add => "Add",
            Self::Save => "Save",
        }
    }

    fn lucide_slug(self) -> &'static str {
        match self {
            Self::More => "ellipsis",
            Self::Back => "arrow-left",
            Self::Close => "x",
            Self::Sync => "refresh-cw",
            Self::Share => "share",
            Self::CopyUrl => "copy",
            Self::Add => "plus",
            Self::Save => "check",
        }
    }

    fn icon_name(self) -> &'static str {
        match self {
            Self::More => "Ellipsis",
            Self::Back => "ArrowLeft",
            Self::Close => "X",
            Self::Sync => "RefreshCw",
            Self::Share => "Share",
            Self::CopyUrl => "Copy",
            Self::Add => "Plus",
            Self::Save => "Check",
        }
    }

    fn icon_primitives(self) -> &'static [IconPrimitive] {
        match self {
            Self::More => &[
                IconPrimitive::Circle {
                    cx: "12",
                    cy: "12",
                    r: "1",
                },
                IconPrimitive::Circle {
                    cx: "19",
                    cy: "12",
                    r: "1",
                },
                IconPrimitive::Circle {
                    cx: "5",
                    cy: "12",
                    r: "1",
                },
            ],
            Self::Back => &[
                IconPrimitive::Path("m12 19-7-7 7-7"),
                IconPrimitive::Path("M19 12H5"),
            ],
            Self::Close => &[
                IconPrimitive::Path("M18 6 6 18"),
                IconPrimitive::Path("m6 6 12 12"),
            ],
            Self::Sync => &[
                IconPrimitive::Path("M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"),
                IconPrimitive::Path("M21 3v5h-5"),
                IconPrimitive::Path("M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"),
                IconPrimitive::Path("M8 16H3v5"),
            ],
            Self::Share => &[
                IconPrimitive::Path("M12 2v13"),
                IconPrimitive::Path("m16 6-4-4-4 4"),
                IconPrimitive::Path("M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"),
            ],
            Self::CopyUrl => &[
                IconPrimitive::Rect {
                    width: "14",
                    height: "14",
                    x: "8",
                    y: "8",
                    rx: "2",
                    ry: "2",
                },
                IconPrimitive::Path("M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"),
            ],
            Self::Add => &[
                IconPrimitive::Path("M5 12h14"),
                IconPrimitive::Path("M12 5v14"),
            ],
            Self::Save => &[IconPrimitive::Path("M20 6 9 17l-5-5")],
        }
    }

    fn icon_view(self) -> AnyView {
        let slug = self.lucide_slug();
        view! {
            <svg
                class="lucide-icon"
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                data-lucide=slug
            >
                {self
                    .icon_primitives()
                    .iter()
                    .copied()
                    .map(IconPrimitive::into_view)
                    .collect_view()}
            </svg>
        }
        .into_any()
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
                    <IconButton
                        kind=IconButtonKind::Close
                        on_press=Callback::new(move |event| {
                            if let Some(handler) = on_close.as_ref() {
                                handler.run(event);
                            }
                        })
                    />
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
pub fn IconButton(
    kind: IconButtonKind,
    #[prop(default = ButtonTone::Quiet)] tone: ButtonTone,
    #[prop(optional)] on_press: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    let label = kind.label();
    let class_name = format!("icon-button {}", tone.class_name());

    view! {
        <button
            type="button"
            class=class_name
            aria-label=label
            title=label
            data-icon=kind.icon_name()
            on:click=move |event| {
                if let Some(handler) = on_press.as_ref() {
                    handler.run(event);
                }
            }
        >
            <span class="icon-button-svg" aria-hidden="true">{kind.icon_view()}</span>
            <span class="sr-only">{label}</span>
        </button>
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

#[cfg(test)]
mod tests {
    use super::IconButtonKind;

    #[test]
    fn icon_actions_map_to_lucide_icons() {
        assert_eq!(IconButtonKind::More.icon_name(), "Ellipsis");
        assert_eq!(IconButtonKind::Back.icon_name(), "ArrowLeft");
        assert_eq!(IconButtonKind::Close.icon_name(), "X");
    }

    #[test]
    fn icon_actions_have_accessible_labels() {
        let kinds = [
            IconButtonKind::More,
            IconButtonKind::Back,
            IconButtonKind::Close,
            IconButtonKind::Sync,
            IconButtonKind::Share,
            IconButtonKind::CopyUrl,
            IconButtonKind::Add,
            IconButtonKind::Save,
        ];

        for kind in kinds {
            assert!(!kind.label().trim().is_empty());
        }
    }

    #[test]
    fn more_icon_uses_three_static_lucide_dots() {
        assert_eq!(IconButtonKind::More.lucide_slug(), "ellipsis");

        let dot_count = IconButtonKind::More
            .icon_primitives()
            .iter()
            .filter(|primitive| {
                matches!(
                    primitive,
                    super::IconPrimitive::Circle {
                        cy: "12",
                        r: "1",
                        ..
                    }
                )
            })
            .count();

        assert_eq!(dot_count, 3);
    }
}
