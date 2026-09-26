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
pub fn SVGArrowDown(#[prop(optional, into)] class: Signal<String>) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=class>
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

#[component]
pub fn SVGSearch(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] stroke: Signal<String>,
) -> impl IntoView {
    let stroke = move || {
        let stroke = stroke.get();
        if stroke.is_empty() {
            "1.5".to_string()
        } else {
            stroke
        }
    };

    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width=stroke stroke="currentColor" class=class>
         <path stroke-linecap="round" stroke-linejoin="round" d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z" />
        </svg>
    }
}

#[component]
pub fn SVGPaw(#[prop(optional, into)] class: Signal<String>) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="32" cy="36" r="9" class="fill-base03"/>
            <circle cx="25" cy="46" r="10" class="fill-base03"/>
            <circle cx="39" cy="46" r="10" class="fill-base03"/>
            <circle cx="13.5" cy="28.5" r="7.5" class="fill-base03"/>
            <circle cx="23.5" cy="15.5" r="7.5" class="fill-base01"/>
            <circle cx="41.5" cy="15.5" r="7.5" class="fill-base03"/>
            <circle cx="50.5" cy="29.5" r="7.5" class="fill-base03"/>
        </svg>
    }
}

#[component]
pub fn SVGProfile(#[prop(optional, into)] class: Signal<String>) -> impl IntoView {
    view! {
        <svg class=class width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M11.9883 12.4831C11.5225 11.8664 10.9198 11.3662 10.2278 11.022C9.53575 10.6779 8.77324 10.4991 8.00033 10.4998C7.22743 10.4991 6.46492 10.6779 5.77288 11.022C5.08084 11.3662 4.47816 11.8664 4.01233 12.4831M11.9883 12.4831C12.8973 11.6746 13.5384 10.6088 13.8278 9.4272C14.1172 8.24555 14.0405 7.00386 13.608 5.86679C13.1754 4.72972 12.4075 3.75099 11.4059 3.0604C10.4044 2.36982 9.21656 2 8 2C6.78344 2 5.59562 2.36982 4.59407 3.0604C3.59252 3.75099 2.82455 4.72972 2.39202 5.86679C1.95949 7.00386 1.88284 8.24555 2.17221 9.4272C2.46159 10.6088 3.10333 11.6746 4.01233 12.4831M11.9883 12.4831C10.891 13.4619 9.47075 14.0019 8.00033 13.9998C6.52969 14.0021 5.10983 13.4621 4.01233 12.4831M10.0003 6.4998C10.0003 7.03024 9.78962 7.53894 9.41455 7.91402C9.03948 8.28909 8.53077 8.4998 8.00033 8.4998C7.4699 8.4998 6.96119 8.28909 6.58612 7.91402C6.21105 7.53894 6.00033 7.03024 6.00033 6.4998C6.00033 5.96937 6.21105 5.46066 6.58612 5.08559C6.96119 4.71052 7.4699 4.4998 8.00033 4.4998C8.53077 4.4998 9.03948 4.71052 9.41455 5.08559C9.78962 5.46066 10.0003 5.96937 10.0003 6.4998Z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}

#[component]
pub fn SVGGear(#[prop(optional, into)] class: Signal<String>) -> impl IntoView {
    view! {
        <svg class=class width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M6.39646 2.62667C6.45646 2.26533 6.7698 2 7.13646 2H8.86513C9.2318 2 9.54513 2.26533 9.60513 2.62667L9.74713 3.48067C9.78913 3.73 9.9558 3.938 10.1771 4.06067C10.2265 4.08733 10.2751 4.116 10.3238 4.14533C10.5405 4.276 10.8038 4.31667 11.0405 4.228L11.8518 3.924C12.0181 3.86147 12.2011 3.85999 12.3684 3.91981C12.5356 3.97963 12.6762 4.09688 12.7651 4.25067L13.6291 5.74867C13.7178 5.90247 13.7491 6.08275 13.7174 6.25744C13.6856 6.43213 13.5929 6.5899 13.4558 6.70267L12.7871 7.254C12.5918 7.41467 12.4951 7.66267 12.5005 7.91533C12.5014 7.972 12.5014 8.02867 12.5005 8.08533C12.4951 8.33733 12.5918 8.58533 12.7871 8.746L13.4565 9.29733C13.7391 9.53067 13.8125 9.934 13.6298 10.2507L12.7645 11.7487C12.6757 11.9024 12.5353 12.0197 12.3681 12.0796C12.201 12.1396 12.0181 12.1383 11.8518 12.076L11.0405 11.772C10.8038 11.6833 10.5405 11.724 10.3231 11.8547C10.2748 11.8841 10.2259 11.9125 10.1765 11.94C9.9558 12.062 9.78913 12.27 9.74713 12.5193L9.60513 13.3733C9.54513 13.7353 9.2318 14 8.86513 14H7.1358C6.76913 14 6.45646 13.7347 6.3958 13.3733L6.2538 12.5193C6.21246 12.27 6.0458 12.062 5.82446 11.9393C5.77503 11.9121 5.72613 11.8838 5.6778 11.8547C5.46113 11.724 5.1978 11.6833 4.96046 11.772L4.14913 12.076C3.98295 12.1383 3.80004 12.1397 3.63293 12.0799C3.46582 12.0201 3.32534 11.903 3.23646 11.7493L2.3718 10.2513C2.28309 10.0975 2.25183 9.91725 2.28357 9.74256C2.31531 9.56787 2.40799 9.4101 2.54513 9.29733L3.21446 8.746C3.40913 8.586 3.5058 8.33733 3.50113 8.08533C3.50009 8.02867 3.50009 7.972 3.50113 7.91533C3.5058 7.662 3.40913 7.41467 3.21446 7.254L2.54513 6.70267C2.40816 6.58993 2.31558 6.43229 2.28384 6.25775C2.25211 6.08321 2.28327 5.90307 2.3718 5.74933L3.23646 4.25133C3.32525 4.09742 3.4658 3.98004 3.63307 3.92009C3.80034 3.86014 3.98346 3.86153 4.1498 3.924L4.96046 4.228C5.1978 4.31667 5.46113 4.276 5.6778 4.14533C5.7258 4.116 5.77513 4.088 5.82446 4.06C6.0458 3.938 6.21246 3.73 6.2538 3.48067L6.39646 2.62667Z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M10 8C10 8.53043 9.78929 9.03914 9.41421 9.41421C9.03914 9.78929 8.53043 10 8 10C7.46957 10 6.96086 9.78929 6.58579 9.41421C6.21071 9.03914 6 8.53043 6 8C6 7.46957 6.21071 6.96086 6.58579 6.58579C6.96086 6.21071 7.46957 6 8 6C8.53043 6 9.03914 6.21071 9.41421 6.58579C9.78929 6.96086 10 7.46957 10 8Z" stroke="#CDD6F4" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}
