use catsquad_shared::{link_relative_profile_gallery, link_relative_user_pfp_get_bytes};
use leptos::prelude::*;

#[component]
pub fn Pfp(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] username: Signal<String>,
    #[prop(optional, into)] hash: Signal<i64>,
    #[prop(optional, into)] no_link: Signal<bool>,
    #[prop(optional, into)] is_loading: Signal<bool>,
    #[prop(default = true.into(), into)] proccesed: Signal<bool>,
) -> impl IntoView {
    let link_img = move || {
        let hash = hash.get();
        let link = if hash == 0 {
            link_relative_user_pfp_get_bytes(username.get())
        } else {
            format!(
                "{}?t={}",
                link_relative_user_pfp_get_bytes(username.get()),
                hash
            )
        };
        format!("url(\"{}\")", link)
    };
    let is_proccesed = move || {
        !is_loading.get() && proccesed.get() && !username.with(|v| v.is_empty()) && hash.get() != 0
    };
    let link_profile = move || link_relative_profile_gallery(username.get());
    let class_proccesed = move || format!("bg-cover bg-center rounded-full {}", class.get());
    let class_unproccesed = move || {
        format!(
            "overflow-hidden bg-base05 text-base01 text-[0.7rem] rounded-full grid place-items-center {}",
            class.get()
        )
    };
    let when_pfp_is_unproccesed = move || !is_proccesed();
    let when_no_link = move || no_link.get();
    let when_yes_link = move || !when_no_link();
    let text = move || match (is_loading.get(), proccesed.get()) {
        (false, false) => "proccessing...",
        (true, _) => "uploading...",
        _ => "",
    };

    view! {
        <Show when=is_proccesed>
            <Show when=when_yes_link>
                <a
                    href=link_profile
                    class=class_proccesed
                    style:background-image=link_img
                >
                </a>
            </Show>
            <Show when=when_no_link>
                <div
                    class=class_proccesed
                    style:background-image=link_img
                >
                </div>
            </Show>
        </Show>
        <Show when=when_pfp_is_unproccesed>
            <div class=class_unproccesed>
                {text}
            </div>
        </Show>
    }
}
