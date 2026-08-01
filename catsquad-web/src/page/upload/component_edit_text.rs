use std::time::Duration;

use catsquad_web_utils::prelude::*;
use leptos::prelude::*;
use web_sys::HtmlTextAreaElement;

use super::component_edit_area::EditArea;
use super::upload_state::FieldSaved;
use crate::page::upload::AUTO_SAVE_TIME;
use crate::{AutoTextArea, Errs, LengthCounter};

#[derive(Default, Clone, Copy, strum::EnumIs)]
pub enum ValidState {
    Empty,
    #[default]
    Error,
    Valid,
}

#[component]
pub fn TextEdit(
    #[prop(optional, into)] title: String,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] on_set: Option<Callback<String>>,
    #[prop(optional, into)] on_update: Option<Callback<()>>,
    #[prop(default = 1)] min_height_rem: u64,
    #[prop(default = 100)] max_length: usize,
    saved_metadata: RwSignal<FieldSaved>,
    input_text: RwSignal<String>,
    errors: RwSignal<String>,
) -> impl IntoView {
    let title_clone1 = title.clone();
    let title_clone2 = title.clone();

    let _handle = interval::new(
        move || {
            let time = time_now_ns();
            let Some(meta_data) = saved_metadata.try_get_untracked() else {
                return;
            };
            let elapsed = time.saturating_sub(meta_data.set_at) >= AUTO_SAVE_TIME;
            let result = saved_metadata.try_update(|v| v.checked_at = time);
            if result.is_none() {
                return;
            }
            if elapsed && !meta_data.saved {
                if let Some(f) = on_update {
                    f.run(());
                }
            }
        },
        Duration::from_secs(1),
    );

    let is_valid = move || -> ValidState {
        if input_text.with(|v| v.is_empty()) {
            return ValidState::Empty;
        }

        if errors.with(|v| !v.is_empty()) {
            return ValidState::Error;
        }

        ValidState::Valid
    };

    let fn_on_input = move |e: HtmlTextAreaElement| {
        let value = e.value();
        if let Some(f) = on_set {
            f.run(value);
        }
    };

    let fn_on_focusout = move |e: HtmlTextAreaElement| {
        if let Some(f) = on_update {
            f.run(());
        }
    };

    let fn_track = move || {
        input_text.track();
    };

    let required_text_color = move || match is_valid() {
        ValidState::Valid => "text-base0B",
        ValidState::Error => "text-base08",
        ValidState::Empty => match required {
            true => "text-base08",
            false => "text-base0A",
        },
    };

    let required_text = move || match is_valid() {
        ValidState::Valid => "is valid",
        ValidState::Error => "is invalid",
        ValidState::Empty => match required {
            true => "is required",
            false => "is optional",
        },
    };

    let saved_text = move || {
        if !errors.with(|v| v.is_empty()) {
            return "invalid".to_string();
        }
        let data = saved_metadata.get();
        // trace!("saved text {} {data:?}", title_clone1);
        match data.saved {
            true => "saved.".to_string(),
            false => {
                let time = time_now_ns();
                let saving_in = AUTO_SAVE_TIME.saturating_sub(time.saturating_sub(data.set_at));
                let saving_in = ns_to_str(saving_in);
                format!("saving in {}", saving_in)
            }
        }
    };

    let saved_text_color = move || {
        if !errors.with(|v| v.is_empty()) {
            return "text-base08";
        }
        let saved = saved_metadata.with(|v| v.saved);
        match saved {
            true => "text-base0B",
            false => "text-base05",
        }
    };

    view! {
        <div class="flex flex-col ">
            <div class="flex gap-2 flex-wrap place-items-center">
                <p class="text-[1.3rem] text-base0F ">{ title_clone1 }</p>
                <ul>
                    <li class=move|| format!("ml-[1rem] list-disc {}", required_text_color()) >{required_text}</li>
                </ul>
            </div>
            <Errs class=move||"mb-2" error=move||errors.get()/>


            <EditArea
                required
                is_valid=move||is_valid()
                class=move||"flex flex-col gap-2"
                >
                <AutoTextArea
                    class=move||"w-full select-none"
                    placeholder=move||title.to_lowercase()
                    track=fn_track
                    on_focusout=fn_on_focusout
                    on_input=fn_on_input
                    min_height=rem_to_px(min_height_rem).unwrap_or_default()>
                    { move || input_text.get() }
                </AutoTextArea>
                <div class="flex justify-between ">
                    <LengthCounter
                        id=move||format!("{}_counter", title_clone2)
                        counter_current=move||input_text.with(|v|v.trim().len())
                        counter_max=move||max_length
                        />
                    <div class=move||format!("{}", saved_text_color())>{saved_text}</div>
                </div>

            </EditArea>

        </div>
    }
}
