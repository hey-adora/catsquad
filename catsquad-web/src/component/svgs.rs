use leptos::prelude::*;

#[component]
pub fn SVGTrash(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=class>
          <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
        </svg>
    }
}

#[component]
pub fn SVGArrowDown(#[prop(optional, into)] class: Option<Callback<(), String>>) -> impl IntoView {
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();

    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=class_fn>
          <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
        </svg>
    }
}

// #[component]
// pub fn SVGTriangleDown(
//     #[prop(optional, into)] class: Option<Callback<(), String>>,
// ) -> impl IntoView {
//     let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();

//     view! {
//         <svg width="13" height="11" viewBox="0 0 13 11" fill="none" xmlns="http://www.w3.org/2000/svg" class=class_fn>
//             <path d="M6.62988 10.25C6.4374 10.5831 5.95713 10.5831 5.76465 10.25L0.56836 1.25C0.375933 0.916705 0.616155 0.500096 1.00098 0.499999L11.3936 0.5C11.7784 0.500098 12.0186 0.916705 11.8262 1.25L6.62988 10.25Z" fill="currentColor" stroke="currentColor"/>
//         </svg>
//     }
// }

#[component]
pub fn SVGTriangle(#[prop(optional, into)] class: Signal<String>) -> impl IntoView {
    view! {
        <svg width="12" height="11" viewBox="0 0 12 11" fill="none" xmlns="http://www.w3.org/2000/svg" class=class>
            <path d="M6.63067 9.75C6.24577 10.4167 5.28352 10.4167 4.89862 9.75L0.135483 1.5C-0.249417 0.833333 0.231708 -2.83122e-07 1.00151 -2.83122e-07L10.5278 -2.83122e-07C11.2976 -2.83122e-07 11.7787 0.833333 11.3938 1.5L6.63067 9.75Z" fill="currentColor"/>
        </svg>
    }
}

#[component]
pub fn SVGStar(
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] fill: Option<Callback<(), bool>>,
) -> impl IntoView {
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();
    let fill_fn = move || {
        let v = fill.map(|v| v.run(())).unwrap_or_default();
        if v { "currentColor" } else { "none" }
    };

    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=class_fn fill=fill_fn>
          <path stroke-linecap="round" stroke-linejoin="round" d="M11.48 3.499a.562.562 0 0 1 1.04 0l2.125 5.111a.563.563 0 0 0 .475.345l5.518.442c.499.04.701.663.321.988l-4.204 3.602a.563.563 0 0 0-.182.557l1.285 5.385a.562.562 0 0 1-.84.61l-4.725-2.885a.562.562 0 0 0-.586 0L6.982 20.54a.562.562 0 0 1-.84-.61l1.285-5.386a.562.562 0 0 0-.182-.557l-4.204-3.602a.562.562 0 0 1 .321-.988l5.518-.442a.563.563 0 0 0 .475-.345L11.48 3.5Z" />
        </svg>
    }
}

#[component]
pub fn SVGSpinner(#[prop(optional, into)] class: Option<Callback<(), String>>) -> impl IntoView {
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();
    let class = move || format!("animate-spin {}", class_fn());

    view! {
        <svg class=class xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" style="--darkreader-inline-stroke: currentColor;" data-darkreader-inline-stroke=""></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
    }
}

#[component]
pub fn SVGUpload(
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] stroke: Signal<String>,
) -> impl IntoView {
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();
    let stroke = move || {
        let stroke = stroke.get();
        if stroke.is_empty() {
            "0".to_string()
        } else {
            stroke
        }
    };

    view! {
        <svg class=class_fn width="15" height="15" viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path stroke="currentColor" stroke-width=stroke d="M6.41667 11V3.52917L4.03333 5.9125L2.75 4.58333L7.33333 0L11.9167 4.58333L10.6333 5.9125L8.25 3.52917V11H6.41667ZM1.83333 14.6667C1.32917 14.6667 0.897722 14.4873 0.539 14.1286C0.180278 13.7699 0.000611111 13.3381 0 12.8333V10.0833H1.83333V12.8333H12.8333V10.0833H14.6667V12.8333C14.6667 13.3375 14.4873 13.7692 14.1286 14.1286C13.7699 14.4879 13.3381 14.6673 12.8333 14.6667H1.83333Z" fill="currentColor"/>
        </svg>
    }
}
