use catsquad_log::prelude::*;
use leptos;
use page::App;

mod component;
mod hook;
mod page;
mod page_state;

pub use component::errors::Errs;
pub use component::nav::Nav;
pub use page_state::PageState;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn csr() {
    console_error_panic_hook::set_once();
    init_log();
    info!("starting web app...");
    leptos::mount::mount_to_body(App);
}

#[cfg(test)]
pub fn init_owner() -> leptos::prelude::Owner {
    use hydration_context::SsrSharedContext;
    use leptos::prelude::Owner;
    use std::sync::Arc;
    Owner::new_root(Some(Arc::new(SsrSharedContext::new())))
}
