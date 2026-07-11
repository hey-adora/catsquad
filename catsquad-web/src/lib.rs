use std::{cell::RefCell, sync::RwLock};

use catsquad_log::prelude::*;
use leptos;
use leptos::prelude::*;
use std::sync::{Arc, LazyLock};
use wasm_bindgen::prelude::*;

// use app::App;
#[component]
fn App() -> impl IntoView {
    view! {
        "yo wtf 3"
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn csr() {
    console_error_panic_hook::set_once();
    init_log();
    info!("starting web app...");
    leptos::mount::mount_to_body(App);
}
