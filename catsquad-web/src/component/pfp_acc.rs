use crate::{PageState, Pfp};
use leptos::prelude::*;

#[component]
pub fn PfpAcc(
    #[prop(optional, into)] is_loading: Signal<bool>,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    let page = PageState::get();
    let username = move || page.acc_username.get();
    let hash = move || page.acc_pfp_hash.get();
    let proccesed = move || page.acc_pfp_proccesed.get();

    view! {
        <Pfp username proccesed hash=hash class=class/>
    }
}
