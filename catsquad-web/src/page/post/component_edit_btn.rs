use crate::{BtnPrimary, BtnSecondary};
use leptos::prelude::*;

#[component]
pub fn EditSaveCancel(
    #[prop(optional, into)] when: Signal<bool>,
    #[prop(optional, into)] disable_save_when: Signal<bool>,
    #[prop(optional, into)] class_save: Signal<String>,
    #[prop(optional, into)] class_cancel: Signal<String>,
    #[prop(optional, into)] class_edit: Signal<String>,
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] on_save: Option<Callback<()>>,
    #[prop(optional, into)] on_cancel: Option<Callback<()>>,
    #[prop(optional, into)] on_edit: Option<Callback<()>>,
) -> impl IntoView {
    // let global_state = expect_context::<GlobalState>();
    let class_save = move || format!("w-[5rem] {}", class_save.get());
    let class_cancel = move || format!("w-[5rem] {}", class_cancel.get());
    let class_edit = move || format!("w-[5rem] {}", class_edit.get());

    let on_save_fn = move |e| {
        if let Some(f) = on_save {
            f.run(());
        }
    };
    let on_cancel_fn = move |_| {
        if let Some(f) = on_cancel {
            f.run(());
        }
    };
    let on_edit_fn = move |_| {
        if let Some(f) = on_edit {
            f.run(());
        }
    };

    let when_fn = move || when.get();
    let id_fn = move || id.get();
    let id_save_fn = move || format!("btn_save_{}", id_fn());
    let id_cancel_fn = move || format!("btn_cancel_{}", id_fn());
    let id_edit_fn = move || format!("btn_edit_{}", id_fn());
    let disable_save_when_fn = move || disable_save_when.get();

    view! {
        <Show when=when_fn >
            <BtnPrimary disabled=disable_save_when_fn class=class_save id=id_save_fn on_click=on_save_fn>
                "Save"
            </BtnPrimary>
            <BtnSecondary class=class_cancel id=id_cancel_fn on_click=on_cancel_fn>
                "Cancel"
            </BtnSecondary>
        </Show>
        <Show when=move || !when_fn() >
            <BtnSecondary class=class_edit id=id_edit_fn on_click=on_edit_fn>
                "Edit"
            </BtnSecondary>
        </Show>
    }
}
