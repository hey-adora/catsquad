use catsquad_shared::link_relative_user_pfp_get_bytes;
use leptos::prelude::*;

use crate::PageState;

#[component]
pub fn Pfp(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] username: Signal<String>,
    #[prop(optional, into)] cache_breaker: Signal<i64>,
    #[prop(optional, into)] is_loading: Signal<bool>,
    #[prop(default = true.into(), into)] proccesed: Signal<bool>,
) -> impl IntoView {
    let link = move || {
        let cache_breaker = cache_breaker.get();
        if cache_breaker == 0 {
            link_relative_user_pfp_get_bytes(username.get())
        } else {
            format!(
                "{}?t={}",
                link_relative_user_pfp_get_bytes(username.get()),
                cache_breaker
            )
        }
    };
    let class = move || format!("rounded-full {}", class.get());
    let when_pfp_is_proccesed =
        move || !is_loading.get() && proccesed.get() && !username.with(|v| v.is_empty());
    let when_pfp_is_unproccesed = move || !when_pfp_is_proccesed();

    view! {
        <Show when=when_pfp_is_proccesed>
            <img class=class src=link />
        </Show>
        <Show when=when_pfp_is_unproccesed>
            <div class=class ></div>
        </Show>
    }
}
