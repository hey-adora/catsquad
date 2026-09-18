use catsquad_log::prelude::*;
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use web_sys::MouseEvent;

use crate::hook::EventListener;
// pub fn Floater(fix_leptos_please: Callback<(), AnyView>) -> impl IntoView {

// #[component(transparent)]
// pub fn Floater(children: Children) -> impl IntoView {
#[component]
pub fn Floater(fix_leptos_please: Callback<(), AnyView>) -> impl IntoView {
    let float_elm = NodeRef::new();
    let is_floating = RwSignal::new(false);
    let offset_xy = RwSignal::new((0.0, 0.0));
    let click_xy = RwSignal::new((0.0, 0.0));
    let float_xy = RwSignal::new((0.0, 0.0));

    let update_float = move |e: MouseEvent| {
        let x = e.client_x();
        let y = e.client_y();
        float_xy.set((x as f64, y as f64));
    };

    let dom_mousemove = EventListener::new(ev::mousemove, move |inner, e: MouseEvent| {
        trace!("dom_mousemove");
        if !is_floating.get_untracked()
            && click_xy.with_untracked(|(click_x, click_y)| {
                let (float_x, float_y) = float_xy.get_untracked();
                (float_x - *click_x).abs() > 10. || (*click_y - float_y).abs() > 10.
            })
        {
            is_floating.set(true);
        }
        update_float(e);
    });

    let dom_mouseup = EventListener::new(ev::mouseup, move |inner, e: MouseEvent| {
        trace!("dom_mouseup");
        dom_mousemove.remove();
        is_floating.set(false);
        inner.remove();
    });

    let dom_mouseleave = EventListener::new(ev::mouseleave, move |inner, e: MouseEvent| {
        trace!("dom_mouseleave");
        dom_mousemove.remove();
        is_floating.set(false);
        inner.remove();
    });

    let set_float = move |e: MouseEvent| {
        trace!("set_float");
        if dom_mousemove.is_set() {
            trace!("already set");
            return;
        }
        trace!("setting document event listener");
        let doc = document().document_element().unwrap();
        dom_mousemove.add(doc.clone());
        dom_mouseup.add(doc.clone());
        dom_mouseleave.add(doc);
        update_float(e);
    };

    let float_mousedown = move |e: MouseEvent| {
        let Some(target): Option<HtmlElement> = e.target().map(|v| v.unchecked_into()) else {
            return;
        };
        let rect = target.get_bounding_client_rect();
        let mouse_x = e.client_x() as f64;
        let mouse_y = e.client_y() as f64;
        let target_x = rect.x();
        let target_y = rect.y();
        let offset = (mouse_x - target_x, mouse_y - target_y);
        trace!(
            "{offset:?} = target_x({target_x}), mouse_x({mouse_x}), target_y({target_y}), mouse_y({mouse_y})"
        );
        offset_xy.set(offset);

        e.prevent_default();
        click_xy.set((mouse_x, mouse_y));
        set_float(e);
    };
    let style_left = move || format!("{}px", float_xy.get().0 - offset_xy.get().0);
    let style_top = move || format!("{}px", float_xy.get().1 - offset_xy.get().1);
    let style_position = move || {
        if is_floating.get() {
            "absolute"
        } else {
            "inherit"
        }
    };
    let style_z = move || {
        if is_floating.get() { "100" } else { "auto" }
    };
    let children = move || fix_leptos_please.run(());

    view! {
        <div
            class=" "
            style:left=style_left
            style:top=style_top
            style:position=style_position
            style:z-index=style_z
            node_ref=float_elm
            on:mousedown=float_mousedown
            >
            {children}
        </div>
    }
}
