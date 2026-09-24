use crate::{BtnDelete, BtnPrimary, ErrGeneral, LinkSecondary, Modal};
use catsquad_log::prelude::*;
use catsquad_shared::{MODAL_QUERY_PARAM_NAME, ModalQueryParamValue, modal_query_params};
use catsquad_web_utils::prelude::RwQuery;
use leptos::prelude::*;
use leptos_router::{hooks::use_location, location::Location};
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, Debug)]
pub struct ModalConfirmDto {
    // pub location: Location,
    pub inner: RwSignal<ModalConfirmDtoInner>,
    // pub rw_query: RwQuery<ModalQueryParamValue>,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum ModalConfirmLevel {
    Normal,
    High,
    Extreme,
}

impl ModalConfirmDto {
    pub fn new() -> Self {
        Self {
            // location: use_location(),
            // rw_query: RwQuery::new(MODAL_QUERY_PARAM_NAME),
            inner: RwSignal::new(ModalConfirmDtoInner::default()),
        }
    }

    pub fn set() {
        provide_context(Self::new());
    }

    pub fn get() -> Self {
        expect_context::<Self>()
    }

    // pub fn is_enabled(&self) -> bool {
    //     self.inner.with(|v| v.enabled)
    // }

    pub fn run(
        &self,
        rw_query: RwQuery<ModalQueryParamValue>,
        callback: impl Into<Callback<()>>,
        title: impl Into<String>,
        text: impl Into<String>,
        level: impl Into<ModalConfirmLevel>,
    ) {
        let current_url = window()
            .location()
            .href()
            .unwrap()
            .replace(modal_query_params(), "");
        // self.location.ba;
        // let parsed_url = url::Url::parse(&current_url);
        // let mut parsed_url = match parsed_url {
        //     Ok(v) => v,
        //     Err(err) => {
        //         error!("failed to parse current url {current_url} {err}");
        //         return;
        //     }
        // };
        // parsed_url.set_query(Some(modal_query_params()));
        // let new_url = parsed_url.to_string();

        self.inner.update(|v| {
            v.level = level.into();
            v.prev_link = current_url;
            v.callback = callback.into();
            v.title = title.into();
            v.message = text.into();
        });

        rw_query.set(ModalQueryParamValue::Enabled);

        // location.n

        // window().location().set_href(new_url.as_str()).unwrap();
        // window()
        //     .history()
        //     .unwrap()
        //     .push_state_with_url(&JsValue::null(), "", Some(new_url.as_str()))
        //     .unwrap();
    }
}

#[derive(Clone, Debug)]
pub struct ModalConfirmDtoInner {
    pub level: ModalConfirmLevel,
    pub prev_link: String,
    pub title: String,
    pub message: String,
    pub callback: Callback<()>,
}

impl Default for ModalConfirmDtoInner {
    fn default() -> Self {
        Self {
            level: ModalConfirmLevel::Normal,
            prev_link: String::new(),
            title: String::new(),
            message: String::new(),
            callback: Callback::from(move || {}),
        }
    }
}

// #[prop(optional, into)] id: Signal<String>

#[component]
pub fn ModalConfirm() -> impl IntoView {
    let confirm_modal = ModalConfirmDto::get();
    // let on_click = move || confirm_modal.run();
    let stage_param = RwQuery::<ModalQueryParamValue>::new(MODAL_QUERY_PARAM_NAME);
    let when_enabled = move || {
        stage_param.get_or_default().is_enabled()
            && confirm_modal.inner.with(|v| !v.message.is_empty())
    };
    let link_back = move || confirm_modal.inner.with(|v| v.prev_link.clone());
    let text = move || confirm_modal.inner.with(|v| v.message.clone());
    let title = move || confirm_modal.inner.with(|v| v.title.clone());
    let level = move || confirm_modal.inner.with(|v| v.level);
    let on_confirm = move |_e| {
        trace!("modal 7");
        confirm_modal.inner.with_untracked(|v| v.callback.run(()));
        stage_param.clear();
    };

    let color = move || match level() {
        ModalConfirmLevel::Normal => "text-base0F",
        ModalConfirmLevel::High => "text-base08",
        ModalConfirmLevel::Extreme => "text-base08",
    };

    let when_primary = move || level() == ModalConfirmLevel::Normal;
    let when_delete = move || level() > ModalConfirmLevel::Normal;

    let class_title = move || format!("text-[1.5rem] text-center {}", color());

    // link=link_back
    view! {
        <Show when=when_enabled >
            <Modal link=link_back>
                <p class=class_title>{title}</p>
                // <ErrGeneral id=move||"passowrd_change_general_error" error=general_errs/>
                {text}
                <div class="flex justify-between gap-2">
                    <Show when=when_delete>
                        <BtnDelete id="modal_confirm_btn" on_click=on_confirm>"Confirm"</BtnDelete>
                    </Show>
                    <Show when=when_primary>
                        <BtnPrimary id="modal_confirm_btn" on_click=on_confirm>"Confirm"</BtnPrimary>
                    </Show>
                    <LinkSecondary id=move||"close_modal_btn" link=link_back>"Cancel"</LinkSecondary>
                </div>
            </Modal>
        </Show>
    }
}
