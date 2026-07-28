use catsquad_client::Client;
use catsquad_client::XMLSender;
use catsquad_log::prelude::*;
use catsquad_shared::UserGetBySessionKeyErr;
use leptos::prelude::*;
use leptos_router::StaticSegment;
use leptos_router::components::*;
use leptos_router::path;

mod index;
mod login;
mod register;

use index::Index as PageIndex;
use login::Login as PageLogin;
use register::Register as PageRegister;

use crate::PageState;
use crate::hook::Spawner;

pub fn create_client() -> Client<XMLSender> {
    Client::new(XMLSender::new())
}

#[component]
pub fn App() -> impl IntoView {
    PageState::set();
    let page = PageState::get();
    let spawner = Spawner::new();

    Effect::new(move || {
        spawner.spawn(async move {
            page.update_auth().await;
        });
    });

    view! {
     <Router>
        <Routes fallback=|| "not found">
            <Route path=path!("/") view=PageIndex />
            <ProtectedRoute path=path!("/login") condition=move||page.is_logged_in().map(|v|!v) redirect_path=move||"/" view=PageLogin />
            <ProtectedRoute path=path!("/register") condition=move||page.is_logged_in().map(|v|!v) redirect_path=move||"/" view=PageRegister />
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
