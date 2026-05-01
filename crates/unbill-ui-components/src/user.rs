use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserItem {
    pub id: String,
    pub display_name: String,
}

#[component]
pub fn UserAvatar(user: UserItem) -> impl IntoView {
    let initial = user
        .display_name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    view! {
        <div class="user-avatar" aria-label=user.display_name.clone()>
            {initial}
        </div>
    }
}

#[component]
pub fn UserRow(user: UserItem, on_tap: Callback<String>) -> impl IntoView {
    let id = user.id.clone();
    view! {
        <div
            class="user-row"
            on:click=move |_| on_tap.run(id.clone())
        >
            <UserAvatar user=user.clone() />
            <span class="user-name">{user.display_name.clone()}</span>
        </div>
    }
}

#[component]
pub fn UserList(
    #[prop(into)] users: Signal<Vec<UserItem>>,
    on_tap: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="user-list">
            {move || users.get().into_iter().map(|user| {
                view! { <UserRow user=user.clone() on_tap=on_tap /> }
            }).collect_view()}
        </div>
    }
}
