use catsquad_log::prelude::*;
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::HtmlDivElement;
use web_sys::HtmlElement;
use web_sys::MouseEvent;

use crate::hook::EventListener;

// #[derive(Clone, Copy)]
// pub struct ZoneData<T: Send + Sync + 'static> {
//     pub is_active: RwSignal<bool>,
//     pub mouse_xy: RwSignal<(f64, f64)>,
//     pub data: RwSignal<Option<T>>,
// }

#[derive(Clone, Copy)]
pub struct ZoneData {
    pub waiting_for_drop: RwSignal<bool>,
    pub zones: RwSignal<Vec<Zone>>,
}

#[derive(Clone)]
pub struct Zone {
    pub zone_index: usize,
    pub is_active: bool,
    pub target: Element,
    pub callback: Callback<(usize, usize)>,
    // pub x: f64,
    // pub y: f64,
    // pub width: f64,
    // pub height: f64,
}

impl ZoneData {
    pub fn new() -> Self {
        Self {
            waiting_for_drop: RwSignal::new(false),
            zones: RwSignal::new(Vec::new()),
        }
    }
}

pub fn mouse_inside_box_bounds(
    mouse_x: f64,
    mouse_y: f64,
    box_x: f64,
    box_y: f64,
    box_width: f64,
    box_height: f64,
) -> bool {
    mouse_x >= box_x
        && mouse_x <= box_x + box_width
        && mouse_y >= box_y
        && mouse_y <= box_y + box_height
}

#[test]
fn test_inside_box_bounds() {
    assert!(!mouse_inside_box_bounds(0., 0., 5., 2., 10., 4.));
    assert!(!mouse_inside_box_bounds(5., 0., 5., 2., 10., 4.));
    assert!(!mouse_inside_box_bounds(5., 1., 5., 2., 10., 4.));
    assert!(mouse_inside_box_bounds(5., 2., 5., 2., 10., 4.));
    assert!(mouse_inside_box_bounds(15., 2., 5., 2., 10., 4.));
    assert!(!mouse_inside_box_bounds(16., 2., 5., 2., 10., 4.));
    assert!(mouse_inside_box_bounds(15., 6., 5., 2., 10., 4.));
    assert!(!mouse_inside_box_bounds(15., 7., 5., 2., 10., 4.));
}

// pub fn Floater(fix_leptos_please: Callback<(), AnyView>) -> impl IntoView {

// #[component(transparent)]
// pub fn Floater(children: Children) -> impl IntoView {
#[component]
pub fn Floater(
    #[prop(into)] zones: ZoneData,
    #[prop(into)] floater_index: Signal<usize>,
    // #[prop(into)] on_drop: Callback<()>,
    fix_leptos_please: Callback<(), AnyView>,
) -> impl IntoView {
    let float_elm = NodeRef::new();
    let is_floating = RwSignal::new(false);
    let offset_xy = RwSignal::new((0.0, 0.0));
    let click_xy = RwSignal::new((0.0, 0.0));
    let float_xy = RwSignal::new((0.0, 0.0));

    let check_if_mouse_inside_zone = move |mouse_x: f64, mouse_y: f64, target: Element| -> bool {
        let rect = target.get_bounding_client_rect();
        let box_x = rect.x();
        let box_y = rect.y();
        let box_width = rect.width();
        let box_height = rect.height();
        mouse_inside_box_bounds(mouse_x, mouse_y, box_x, box_y, box_width, box_height)
    };

    let update_zones = move |mouse_x: f64, mouse_y: f64| {
        let z_iter = zones.zones.get_untracked().into_iter().enumerate();
        for (i, z) in z_iter {
            let is_active = z.is_active;
            let new_is_active = check_if_mouse_inside_zone(mouse_x, mouse_y, z.target);
            if is_active != new_is_active {
                zones.zones.update(|v| v[i].is_active = new_is_active);
            }
        }
    };

    let run_on_drop = move |mouse_x: f64, mouse_y: f64| {
        let z_iter = zones.zones.get_untracked().into_iter().enumerate();
        for (i, z) in z_iter {
            if check_if_mouse_inside_zone(mouse_x, mouse_y, z.target) {
                let floater_index = floater_index.get();
                zones
                    .zones
                    .with_untracked(|v| v[i].callback.run((z.zone_index, floater_index)));
                return;
            }
        }
    };

    let update_float = move |e: MouseEvent| {
        let mouse_x = e.client_x() as f64;
        let mouse_y = e.client_y() as f64;
        float_xy.set((mouse_x, mouse_y));
        update_zones(mouse_x, mouse_y);
        // let Some((zone, new_is_active)) = check_if_in_zone(mouse_x, mouse_y) else {
        //     return;
        // };
        // trace!("update_float 1 {} == {}", zone.is_active, new_is_active);
        // if zone.is_active == new_is_active {
        //     return;
        // }
        // trace!("update_float 2");
        // let zone_index = zone.index;
        // zones.zones.update(|v| {
        //     if let Some(pos) = v.iter().position(|v| v.index == zone_index) {
        //         v[pos].is_active = new_is_active;
        //     }
        // });
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
            zones.waiting_for_drop.set(true);
            // zone.index.set(index.get_untracked());
        }
        update_float(e);
    });

    let dom_mouseup = EventListener::new(ev::mouseup, move |inner, e: MouseEvent| {
        trace!("dom_mouseup");
        let mouse_x = e.client_x() as f64;
        let mouse_y = e.client_y() as f64;
        run_on_drop(mouse_x, mouse_y);
        dom_mousemove.remove();
        is_floating.set(false);
        zones.waiting_for_drop.set(false);
        inner.remove();
    });

    let dom_mouseleave = EventListener::new(ev::mouseleave, move |inner, e: MouseEvent| {
        trace!("dom_mouseleave");
        dom_mousemove.remove();
        is_floating.set(false);
        zones.waiting_for_drop.set(false);
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
    let when_is_floating = move || is_floating.get();

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
        <Show when=when_is_floating>
            <div class="size-[8rem] border-2 border-base05 rounded-xl "></div>

        </Show>
    }
}

#[component]
pub fn Zone(
    #[prop(into)] zone_index: Signal<usize>,
    #[prop(into)] zones: ZoneData,
    #[prop(into)] on_drop: Callback<(usize, usize)>,
    // #[prop(into)] view_zone: Callback<>,
) -> impl IntoView {
    let target = NodeRef::new();

    let class_glow = move || {
        let zone_index = zone_index.get();
        let glow = match (
            zones.waiting_for_drop.get(),
            zones.zones.with(|v| {
                v.iter()
                    .find(|v| v.zone_index == zone_index)
                    .map(|v| v.is_active)
                    .unwrap_or_default()
            }),
        ) {
            (true, false) => "h-[8rem] w-[2rem] bg-base08",
            (true, true) => "h-[8rem] w-[2rem] bg-base0B",
            _ => "",
        };
        format!(" rounded {glow}",)
    };

    Effect::new(move || {
        let zone_index = zone_index.get_untracked();
        zones.zones.update(|v| {
            let Some(target): Option<HtmlDivElement> = target.get_untracked() else {
                return;
            };
            let target: Element = target.into();

            if let Some(pos) = v.iter().position(|v| v.zone_index == zone_index) {
                v.remove(pos);
            };

            v.push(Zone {
                zone_index,
                target,
                is_active: false,
                callback: on_drop,
            });
        });
    });

    on_cleanup(move || {
        let zone_index = zone_index.get_untracked();
        zones.zones.update(|v| {
            if let Some(pos) = v.iter().position(|v| v.zone_index == zone_index) {
                v.remove(pos);
            };
        });
    });

    view! {
        <div node_ref=target class=class_glow>

        </div>
    }
}
