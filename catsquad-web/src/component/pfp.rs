use catsquad_shared::link_relative_user_pfp_get_bytes;
use leptos::prelude::*;

#[component]
pub fn Pfp(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] username: Signal<String>,
    #[prop(optional, into)] hash: Signal<i64>,
    #[prop(optional, into)] is_loading: Signal<bool>,
    #[prop(default = true.into(), into)] proccesed: Signal<bool>,
) -> impl IntoView {
    let link = move || {
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
    let class_proccesed = move || format!("bg-cover bg-center rounded-full {}", class.get());
    let class_unproccesed = move || {
        format!(
            "bg-base05 text-base01 text-[0.7rem] rounded-full grid place-items-center {}",
            class.get()
        )
    };
    let when_pfp_is_proccesed =
        move || !is_loading.get() && proccesed.get() && !username.with(|v| v.is_empty());
    let when_pfp_is_unproccesed = move || !when_pfp_is_proccesed();
    let when_is_loading = move || is_loading.get();

    view! {
        <Show when=when_pfp_is_proccesed>
            <div
                class=class_proccesed
                style:background-image=link
            >
            </div>
        </Show>
        <Show when=when_pfp_is_unproccesed>
            <div class=class_unproccesed>
                <Show
                    when=when_is_loading
                    fallback=move||"proccessing..."
                    >
                    "uploading..."
                </Show>
            </div>
        </Show>
    }
}
