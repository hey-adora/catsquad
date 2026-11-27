use crate::{ModalConfirmDto, ModalConfirmLevel, SVGSpinner};
use catsquad_log::prelude::*;
use catsquad_shared::MODAL_QUERY_PARAM_NAME;
use catsquad_web_utils::prelude::RwQuery;
use leptos::prelude::*;
use web_sys::MouseEvent;

#[derive(Clone, Copy, Default)]
pub enum BtnSize {
    Small,
    #[default]
    Normal,
}

// #[derive(Clone, Copy, Default)]
// pub enum BtnSize {
//     Small,
//     #[default]
//     Normal,
// }

#[component]
pub fn Btn(
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] is_loading: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] basic: Signal<bool>,
    #[prop(optional, into)] primary: Signal<bool>,
    #[prop(optional, into)] secondary: Signal<bool>,
    #[prop(optional, into)] active: Signal<bool>,
    #[prop(optional, into)] warning: Signal<bool>,
    #[prop(optional, into)] danger: Signal<bool>,
    #[prop(optional, into)] modal_enable: Signal<bool>,
    #[prop(optional, into)] modal_title: Signal<String>,
    #[prop(optional, into)] modal_text: Signal<String>,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, into)] on_modal_open: Option<Callback<()>>,
    #[prop(optional, into)] class_on_disable: Signal<String>,
    #[prop(optional, into)] class_on_active: Signal<String>,
    #[prop(optional, into)] size: Signal<BtnSize>,
    children: Children,
) -> impl IntoView {
    let id_fn = move || id.get();
    let class_fn = move || class.get();
    let class_on_disable = move || class_on_disable.get();
    let class_on_active = move || class_on_active.get();
    let is_loading_fn = move || is_loading.get();
    let disabled_fn = move || disabled.get();

    let is_disabled_fn = move || is_loading_fn() || disabled_fn();

    let size_class = move || match size.get() {
        BtnSize::Small => "text-[0.8rem] leading-[0.8rem] px-[0.8rem] py-[0.25rem]",
        BtnSize::Normal => "text-[1rem] leading-[1rem] px-[1rem] py-[0.5rem]",
    };

    let base_class = move || {
        format!(
            "flex gap-2 place-content-center rounded-xl {}",
            size_class()
        )
    };

    let class_on_disable = move || {
        let class_dark_on_disable = move || "bg-base03 font-bold text-base01";
        let class_darker_on_disable = move || "bg-base02 text-base01";

        if basic.get() {
            return format!("{} {}", base_class(), class_on_disable());
        }

        if primary.get() || active.get() || warning.get() || danger.get() {
            return format!(
                "{} {} {}",
                base_class(),
                class_dark_on_disable(),
                class_on_disable()
            );
        }

        if secondary.get() {
            return format!(
                "{} {} {}",
                base_class(),
                class_darker_on_disable(),
                class_on_disable()
            );
        }

        class_on_disable()
    };

    let class_on_active = move || {
        let class_primary_on_active = move || "hover:bg-base05 bg-base0D font-bold text-base01";
        let class_secondary_on_active = move || "hover:bg-base0D bg-base03 text-base05";
        let class_active_on_active = move || "hover:bg-base05 bg-base0B font-bold text-base01";
        let class_warning_on_active = move || "hover:bg-base05 bg-base0A font-bold text-base01";
        let class_danger_on_active = move || "hover:bg-base05 bg-base08 font-bold text-base01";

        if basic.get() {
            return format!("{} {}", base_class(), class_on_active());
        }

        if primary.get() {
            return format!(
                "{} {} {}",
                base_class(),
                class_primary_on_active(),
                class_on_active()
            );
        }

        if secondary.get() {
            return format!(
                "{} {} {}",
                base_class(),
                class_secondary_on_active(),
                class_on_active()
            );
        }

        if active.get() {
            return format!(
                "{} {} {}",
                base_class(),
                class_active_on_active(),
                class_on_active()
            );
        }

        if active.get() {
            return format!(
                "{} {} {}",
                base_class(),
                class_active_on_active(),
                class_on_active()
            );
        }

        if warning.get() {
            return format!(
                "{} {} {}",
                base_class(),
                class_warning_on_active(),
                class_on_active()
            );
        }

        if danger.get() {
            return format!(
                "{} {} {}",
                base_class(),
                class_danger_on_active(),
                class_on_active()
            );
        }

        class_on_active()
    };

    let class_fn = move || {
        format!(
            "{} {}",
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

    let run_callback_fn = move |e: MouseEvent| {
        trace!("on click 5");
        if let Some(on_click) = on_click {
            trace!("on click 6");
            on_click.run(e.clone());
        }
    };

    let open_modal_fn = move |e: MouseEvent| {
        trace!("on click 4");
        modal_confirm.run(
            modal_rw_query,
            move || run_callback_fn(e.clone()),
            modal_title.get_untracked(),
            modal_text.get_untracked(),
            ModalConfirmLevel::High,
        )
    };

    let on_click = move |e: MouseEvent| {
        trace!("on click 1");
        if modal_enable.get_untracked() {
            if let Some(on_modal_open) = on_modal_open {
                trace!("on click 6");
                on_modal_open.run(());
            }
            trace!("on click 2");
            open_modal_fn(e);
        } else {
            trace!("on click 3");
            run_callback_fn(e);
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
