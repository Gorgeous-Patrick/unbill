mod api;
mod app;
mod components;
mod pages;

use std::sync::Arc;

use leptos::prelude::*;
use unbill_core::service::UnbillService;
use unbill_store_http::HttpStore;

fn main() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        let base_url = match option_env!("UNBILL_SERVER_URL") {
            Some(url) if !url.is_empty() => url.to_owned(),
            _ => web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_default(),
        };
        let api_key = option_env!("UNBILL_API_KEY").unwrap_or("").to_owned();
        let store = Arc::new(HttpStore::new(base_url, api_key));
        let service = UnbillService::open(store)
            .await
            .expect("failed to initialize unbill service");
        api::init(service);
        mount_to_body(|| view! { <app::App /> });
    });
}
