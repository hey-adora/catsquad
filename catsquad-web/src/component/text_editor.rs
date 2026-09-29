use crate::{AutoTextArea, Errs, LengthCounter, page::post::component_edit_btn::EditSaveCancel};
use leptos::{html, prelude::*};
use web_sys::HtmlTextAreaElement;

#[component]
pub fn TextEditor(
    #[prop(into)] id_prefix: Signal<String>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] text: RwSignal<String>,
    #[prop(into)] text_length: RwSignal<usize>,
    #[prop(into)] is_owned: Signal<bool>,
    #[prop(into)] edit_mode_enabled: RwSignal<bool>,
    #[prop(into)] max_length: Signal<usize>,
    #[prop(into)] errors: Signal<String>,
    #[prop(into)] on_save: Callback<()>,
    #[prop(into)] node_ref: NodeRef<html::Textarea>,
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView {
    let is_owned = move || is_owned.get();
    let on_save = move || {
        on_save.run(());
    };
    let is_edit_mode_enabled = move || edit_mode_enabled.get();
    let is_text_empty = move || text.with(|v| v.is_empty());
    let is_text_full = move || !is_text_empty();

    let toggle_edit_mode = move || edit_mode_enabled.update(|v| *v = !*v);
    let on_input = move |v: HtmlTextAreaElement| text_length.set(v.value().len());
    let text = move || text.get();
    let title = move || title.get();

    let id_pre = move || format!("{}_editor_outout", id_prefix.get());
    let id_textarea = move || format!("{}_editor_textarea", id_prefix.get());
    let id_errs = move || format!("{}_editor_errors", id_prefix.get());

    let children = move || {
        if let Some(children) = &children {
            children()
        } else {
            view! {
                <pre
                    id=id_pre
                    class="whitespace-break-spaces break-all text-ellipsis overflow-hidden padding max-w-[calc(100vw-1rem)] rounded {}">
                    <Show when=is_text_full fallback={move || view!{<span class="text-base03">"No description."</span>} }>
                        { text }
                    </Show>
                </pre>
            }.into_any()
        }
    };

    view! {
        <div class="flex flex-col gap-2 md:gap-4 justify-between mt-4">
            <div class="flex justify-between">
                <h1 class="text-[1.3rem] text-base0F">{title}</h1>
                <div class="flex gap-2 items-center">

                    <Show when=is_owned >
                        <Show when=is_edit_mode_enabled>
                            <LengthCounter
                                counter_current=text_length
                                counter_max=max_length
                            />
                        </Show>
                        <EditSaveCancel
                            id=id_prefix
                            when=edit_mode_enabled
                            on_save=on_save
                            on_cancel=toggle_edit_mode
                            on_edit=toggle_edit_mode
                        />
                    </Show>

                </div>
            </div>

            <Errs id=id_errs error=errors />

            <Show when=is_edit_mode_enabled fallback=move || view!{
                { children.clone() }
            }>
                <AutoTextArea
                    id=id_textarea
                    node_ref=node_ref
                    on_input=on_input
                    class="bg-base01 text-base05 px-4 py-2 rounded"
                >
                    { text }
                </AutoTextArea>
            </Show>

        </div>
    }
}
