use catsquad_client::Client;
use catsquad_client::XMLSender;
use catsquad_log::prelude::*;
use catsquad_web_utils::interval;
use catsquad_web_utils::time::time_now_micro;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use std::time::Duration;

pub mod index;
pub mod login;
pub mod post;
pub mod profile;
pub mod register;
pub mod settings;
pub mod upload;

use index::Index as PageIndex;
use login::Login as PageLogin;
use post::Post as PagePost;
use profile::Profile as PageProfile;
use register::Register as PageRegister;
use settings::Settings as PageSettings;
use upload::Upload as PageUpload;

use crate::ModalConfirmDto;
use crate::PageState;
use crate::hook::Spawner;
use crate::page::settings::user_pfp_state::UserPfpState;

pub fn create_client() -> Client<XMLSender> {
    Client::new(XMLSender::new())
}

#[component]
pub fn App() -> impl IntoView {
    PageState::set(create_client());
    ModalConfirmDto::set();

    let page = PageState::get();
    let spawner_page = Spawner::new();
    let pfp_state =
        UserPfpState::new_with_state(create_client(), page.acc_pfp_hash, page.acc_pfp_proccesed);
    let spawner_pfp = Spawner::new();

    Effect::new(move || {
        spawner_page.spawn(async move {
            page.update_auth().await;
        });
    });

    interval::new(
        move |_handle| {
            let time = time_now_micro();
            page.time.set(time);
            if !pfp_state.is_pfp_proccesed() {
                spawner_pfp.spawn(async move {
                    trace!("checking if pfp is proccesed");
                    pfp_state.update_pfp_proccesed_status().await;
                });
            }
        },
        Duration::from_secs(1),
    )
    .unwrap();

    // interval::new(
    //     move |_handle| {
    //         let time = time_now_micro();
    //         page.time.set(time);
    //     },
    //     Duration::from_secs(1),
    // )
    // .unwrap();

    view! {
     <Router>
        <Routes fallback=|| "not found">
            <Route path=path!("/") view=PageIndex />
            <Route path=path!("/p/:post") view=PagePost />
            <Route path=path!("/u/:username") view=PageProfile />
            <ProtectedRoute path=path!("/login") condition=move||page.is_logged_in().map(|v|!v) redirect_path=move||"/" view=PageLogin />
            <ProtectedRoute path=path!("/register") condition=move||page.is_logged_in().map(|v|!v) redirect_path=move||"/" view=PageRegister />
            <ProtectedRoute path=path!("/upload") condition=move||page.is_logged_in().map(|v|v) redirect_path=move||"/login" view=PageUpload />
            <ProtectedRoute path=path!("/settings") condition=move||page.is_logged_in().map(|v|v) redirect_path=move||"/login" view=PageSettings />
        </Routes>
      </Router>
    }
}
// <Route path=path!("/u/:username/:post")   view=post::Page />
// <Route path=path!("/u/:username") view=profile::Page />
// <ProtectedRoute path=path!("/settings") condition=move||global_state.is_logged_in() redirect_path view=settings::Page />
// <ProtectedRoute path=path!("/upload") condition=move||global_state.is_logged_in() redirect_path view=upload::Page />
// <ProtectedRoute path=path!("/login") condition=move||global_state.is_logged_in().map(|v| !v) redirect_path view=login::Page />
// <ProtectedRoute path=path!("/register") condition=move||global_state.is_logged_in().map(|v| !v) redirect_path view=register::Page />
