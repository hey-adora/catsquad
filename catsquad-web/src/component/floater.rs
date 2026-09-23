use catsquad_log::prelude::*;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::tachys::html::node_ref::node_ref;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::HtmlDivElement;
use web_sys::HtmlElement;
use web_sys::MouseEvent;

use crate::hook::EventListener;

#[derive(Clone, Copy)]
pub struct ZoneData {
    pub floater_index: RwSignal<Option<usize>>,
    pub zones: RwSignal<Vec<Zone>>,
}

#[derive(Clone, Debug)]
pub struct Zone {
    pub zone_index: usize,
    pub is_selected: bool,
    pub target: Element,
    pub callback: Callback<(usize, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoneStage {
    None,
    Active,
    Selected,
}

impl ZoneData {
    pub fn new() -> Self {
        Self {
            floater_index: RwSignal::new(None),
            zones: RwSignal::new(Vec::new()),
        }
    }
}

pub fn calc_zone_stages(
    floater_index: Option<usize>,
    zone_index: usize,
    is_selected: bool,
) -> ZoneStage {
    match (floater_index, is_selected) {
        (Some(float_index), false) if float_index != zone_index => ZoneStage::Active,
        (Some(_), true) => ZoneStage::Selected,
        _ => ZoneStage::None,
    }
}

#[cfg(test)]
#[test]
fn test_calc_zone_stages() {
    assert_eq!(
        vec![
            calc_zone_stages(Some(2), 0, false),
            // floater 0
            calc_zone_stages(Some(2), 1, false),
            // floater 1
            calc_zone_stages(Some(2), 2, false),
            // floater 2 - active
            calc_zone_stages(Some(2), 3, false),
            // floater 3
            // calc_zone_stages(Some(2), 4, false),
        ],
        vec![
            ZoneStage::Active,
            // floater 0
            ZoneStage::Active,
            // floater 1
            ZoneStage::None,
            // floater 2 - active
            ZoneStage::Active,
            // floater 3
            // ZoneStage::Active,
        ]
    );

    // use catsquad_api::auth::create_auth_cookie_str;
    // use catsquad_shared::uuid_to_str;
    // use http::header;

    // catsquad_log::init_log();
    // let _owner = crate::init_owner();

    // let server = catsquad_api::TestServer::new(0, "test_which_zone_is_active").await;

    // let (_user1, session1) = server
    //     .user_add_full(
    //         "prime1",
    //         "prime1@heyadora.com",
    //         "235j4t49ngerigrog#IOTNOnfo",
    //     )
    //     .await;

    // server
    //     .inject_header(
    //         header::COOKIE,
    //         create_auth_cookie_str(uuid_to_str(session1)),
    //     )
    //     .await;

    // upload.init creates new post draft
    // let upload = UploadState::new(0);
    // upload.init(&server.client).await;

    // (server, owner, upload)
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

#[component]
pub fn Floater(
    zones: ZoneData,
    #[prop(into)] floater_index: Signal<usize>,
    #[prop(optional, into)] disabled: Signal<bool>,
    children: ChildrenFn,
    // view_fn: Callback<(), AnyView>,
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
        let Some(z_iter) = zones.zones.try_get_untracked() else {
            return;
        };
        let z_iter = z_iter.into_iter().enumerate();
        for (i, z) in z_iter {
            let is_active = z.is_selected;
            let new_is_active = check_if_mouse_inside_zone(mouse_x, mouse_y, z.target);
            if is_active != new_is_active {
                trace!("11111111 update_zones");
                zones.zones.try_update(|v| v[i].is_selected = new_is_active);
            }
        }
    };

    let run_on_drop = move |mouse_x: f64, mouse_y: f64| {
        let Some(z_iter) = zones.zones.try_get_untracked() else {
            return;
        };
        let z_iter = z_iter.into_iter().enumerate();
        for (i, z) in z_iter {
            if check_if_mouse_inside_zone(mouse_x, mouse_y, z.target) {
                let Some(floater_index) = floater_index.try_get_untracked() else {
                    return;
                };
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
    };

    let dom_mousemove = EventListener::new(ev::mousemove, move |inner, e: MouseEvent| {
        trace!("dom_mousemove");
        let moved_10_px = click_xy.with_untracked(|(click_x, click_y)| {
            let Some((float_x, float_y)) = float_xy.try_get_untracked() else {
                return false;
            };
            (float_x - *click_x).abs() > 10. || (*click_y - float_y).abs() > 10.
        });
        let floater_index = floater_index.get();
        if !is_floating.try_get_untracked().unwrap_or_default() && moved_10_px {
            is_floating.set(true);
            zones.floater_index.set(Some(floater_index));
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
        zones.floater_index.set(None);
        inner.remove();
    });

    let dom_mouseleave = EventListener::new(ev::mouseleave, move |inner, e: MouseEvent| {
        trace!("dom_mouseleave");
        dom_mousemove.remove();
        is_floating.set(false);
        zones.floater_index.set(None);
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
        trace!("float_mousedown {:#?}", zones.zones.try_get_untracked());
        if disabled.try_get_untracked().unwrap_or_default() {
            return;
        }
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
    let style_left = move || {
        let (Some((float_x, _)), Some((offset_x, _))) = (float_xy.try_get(), offset_xy.try_get())
        else {
            return "0px".to_string();
        };

        format!("{}px", float_x - offset_x)
    };
    let style_top = move || {
        let (Some((_, float_y)), Some((_, offset_y))) = (float_xy.try_get(), offset_xy.try_get())
        else {
            return "0px".to_string();
        };

        format!("{}px", float_y - offset_y)
    };
    let style_position = move || {
        if is_floating.try_get().unwrap_or_default() {
            "absolute"
        } else {
            "inherit"
        }
    };
    let style_z = move || {
        if is_floating.try_get().unwrap_or_default() {
            "100"
        } else {
            "auto"
        }
    };
    // let view_fn = move || view_fn.run(());
    let when_is_floating = move || is_floating.try_get().unwrap_or_default();

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
            { children() }
        </div>
        <Show when=when_is_floating>
            <div class="size-[8rem] border-2 border-base05 rounded-xl "></div>
        </Show>
    }
}

#[component]
pub fn Zone(
    #[prop(into)] zone_index: Signal<usize>,
    zones: ZoneData,
    #[prop(into)] on_drop: Callback<(usize, usize)>,
) -> impl IntoView {
    let target = NodeRef::<html::Div>::new();

    Effect::new(move || {
        let Some(target): Option<HtmlDivElement> = target.get() else {
            return;
        };
        let zone_index = zone_index.get();
        trace!("zone_effect {zone_index}");
        zones.zones.update(|v| {
            let target: Element = target.into();

            if let Some(pos) = v.iter().position(|v| v.zone_index == zone_index) {
                v.remove(pos);
            };

            v.push(Zone {
                zone_index,
                target,
                is_selected: false,
                callback: on_drop,
            });
        });
    });

    on_cleanup(move || {
        let zone_index = zone_index.get();
        trace!("on_cleanup {zone_index}");
        zones.zones.update(|v| {
            if let Some(pos) = v.iter().position(|v| v.zone_index == zone_index) {
                v.remove(pos);
            };
        });
    });

    let zone_stage = move || -> ZoneStage {
        let zone_index = zone_index.get();
        trace!("zone_stage {zone_index}");
        let floater_index = zones.floater_index.get();
        let is_selected = zones.zones.with(|v| {
            v.iter()
                .find(|v| v.zone_index == zone_index)
                .map(|v| v.is_selected)
                .unwrap_or_default()
        });
        calc_zone_stages(floater_index, zone_index, is_selected)
        // match (
        //     zones.floater_index.get(),
        //     zones.zones.with(|v| {
        //         v.iter()
        //             .find(|v| v.zone_index == zone_index)
        //             .map(|v| v.is_selected)
        //             .unwrap_or_default()
        //     }),
        // ) {
        //     (Some(floater_index), false) if floater_index != zone_index =>
        //     // if floater_index != zone_index.saturating_sub(1)
        //     //     && floater_index != zone_index + 1
        //     //     && floater_index != zone_index =>
        //     {
        //         ZoneStage::Active
        //     }
        //     (Some(_), true) => ZoneStage::Selected,
        //     _ => ZoneStage::None,
        // }
    };

    let class_fn = move || match zone_stage() {
        ZoneStage::None => "hidden",
        ZoneStage::Active => "h-[8rem] w-[2rem] bg-base08",
        ZoneStage::Selected => "h-[8rem] w-[2rem] bg-base0B",
    };

    let when_to_show = move || zone_stage() != ZoneStage::None;

    view! {
        <Show when=when_to_show>
          <div node_ref=target class=class_fn></div>
        </Show>
    }
}

// let id = move || id.try_get();
// let class_fn = move || match stage {
//     ZoneStage::None => "hidden",
//     ZoneStage::Waiting => "h-[8rem] w-[2rem] bg-base08",
//     ZoneStage::Hovering => "h-[8rem] w-[2rem] bg-base0B",
// };
// let when_to_show = move || stage != ZoneStage::None;

// // <Show when=when_to_show>
// //     <div id=id class=class_fn></div>
// // </Show>
// view! {
//     <div id=id class=class_fn></div>
// }
// .into_any()
