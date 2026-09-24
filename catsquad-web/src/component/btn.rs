use crate::{ModalConfirmDto, ModalConfirmLevel, SVGSpinner};
use catsquad_log::prelude::*;
use catsquad_shared::MODAL_QUERY_PARAM_NAME;
use catsquad_web_utils::prelude::RwQuery;
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn Btn(
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] is_loading: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] modal_enable: Signal<bool>,
    #[prop(optional, into)] modal_title: Signal<String>,
    #[prop(optional, into)] modal_text: Signal<String>,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, into)] class_on_disable: Signal<String>,
    #[prop(optional, into)] class_on_active: Signal<String>,
    children: Children,
) -> impl IntoView {
    let id_fn = move || id.get();
    let class_fn = move || class.get();
    let class_on_disable = move || class_on_disable.get();
    let class_on_active = move || class_on_active.get();
    let is_loading_fn = move || is_loading.get();
    let disabled_fn = move || disabled.get();

    let is_disabled_fn = move || is_loading_fn() || disabled_fn();

    let class_fn = move || {
        format!(
            "flex gap-2 place-content-center rounded-xl text-[1rem] leading-[1rem] px-[1rem] py-[0.5rem]  {} {}",
            if is_disabled_fn() {
                class_on_disable()
            } else {
                class_on_active()
            },
            class_fn()
        )
    };

    let modal_confirm = ModalConfirmDto::get();
    let modal_rw_query = RwQuery::new(MODAL_QUERY_PARAM_NAME);

    let run_on_click = move |e: MouseEvent| {
        trace!("on click 5");
        if let Some(on_click) = on_click {
            trace!("on click 6");
            on_click.run(e.clone());
        }
    };

    let on_click_modal = move |e: MouseEvent| {
        trace!("on click 4");
        modal_confirm.run(
            modal_rw_query,
            move || run_on_click(e.clone()),
            modal_title.get_untracked(),
            modal_text.get_untracked(),
            ModalConfirmLevel::High,
        )
    };

    let on_click = move |e: MouseEvent| {
        trace!("on click 1");
        if modal_enable.get_untracked() {
            trace!("on click 2");
            on_click_modal(e);
        } else {
            trace!("on click 3");
            run_on_click(e);
        }
    };
    // on:click=on_click_handler

    view! {
        <button
            id=id_fn
            disabled=is_disabled_fn
            on:click=on_click
            class=class_fn>
            <Show when=is_loading_fn>
                <SVGSpinner class=move||"size-4"/>
            </Show>
            {children()}
        </button>
    }
}
