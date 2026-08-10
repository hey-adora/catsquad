use std::fmt::Debug;

use crate::{Errs, PageState, hook::Spawner, page::create_client};
use catsquad_client::{Client, Response, Sender};
use catsquad_shared::{UserAddErr, validate_password, validate_username};
use catsquad_web_utils::prelude::RwQuery;
use leptos::{prelude::*, task::spawn_local};
use web_sys::HtmlInputElement;

#[derive(Clone, Copy)]
pub struct RegisterFormState {
    pub err_general: RwSignal<String>,
    pub err_username: RwSignal<String>,
    pub err_invite_key: RwSignal<String>,
    pub err_password: RwSignal<String>,
}

impl RegisterFormState {
    pub fn new() -> Self {
        Self {
            err_general: RwSignal::new(String::new()),
            err_username: RwSignal::new(String::new()),
            err_invite_key: RwSignal::new(String::new()),
            err_password: RwSignal::new(String::new()),
        }
    }

    pub async fn run_register<TSender>(
        &self,
        client: &Client<TSender>,
        username: impl Into<String>,
        invite_key: impl Into<String>,
        password: impl Into<String>,
        password_confirmation: impl Into<String>,
    ) where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let username = username.into().trim().to_lowercase();
        let password = password.into();
        let password_confirmation = password_confirmation.into();

        let mut username_errs = String::new();
        let mut password_errs = String::new();

        if password != password_confirmation {
            password_errs += "passwords dont match\n";
        }

        if let Err(err) = validate_password(&password) {
            password_errs += &err;
        }

        if let Err(err) = validate_username(&username) {
            username_errs += &err;
        }

        self.err_username.set(username_errs.clone());
        self.err_password.set(password_errs.clone());

        if !password_errs.is_empty() || !username_errs.is_empty() {
            return;
        }

        let result = client
            .user_add(username, invite_key, password)
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(v) => {}
            Err(UserAddErr::InvalidInput { username, password }) => {
                if let Some(err) = username {
                    self.err_username.set(err);
                }

                if let Some(err) = password {
                    self.err_password.set(err);
                }
            }
            Err(UserAddErr::UsernameIsTaken) => {
                self.err_username.set("username is taken".to_string())
            }
            Err(UserAddErr::EmailIsTaken) => self.err_invite_key.set("email is taken".to_string()),
            Err(UserAddErr::InviteNotFound) => {
                self.err_invite_key.set("invite not found".to_string())
            }
            Err(UserAddErr::InviteAlreadyUsed) => {
                self.err_invite_key.set("already used".to_string())
            }
            Err(UserAddErr::InviteExpired) => self.err_invite_key.set("invite expired".to_string()),
            Err(UserAddErr::BadRequest(err)) => self.err_general.set(err),
            Err(UserAddErr::InternalServer) => {
                self.err_general.set("internal server err".to_string())
            }
        }

        //
    }

    pub fn has_no_err(&self) -> bool {
        let has_err_general = self.err_general.with_untracked(|v| v.is_empty());
        let has_err_username = self.err_username.with_untracked(|v| v.is_empty());
        let has_err_invite_key = self.err_invite_key.with_untracked(|v| v.is_empty());
        let has_err_password = self.err_password.with_untracked(|v| v.is_empty());
        has_err_general && has_err_username && has_err_invite_key && has_err_password
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_register_form_state() {
    use catsquad_api::id_to_string;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new().await;
    let client = &server.client;

    let result = client
        .invite_add("prime@heyadora.com")
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    let invite_key = server
        .state
        .db
        .invite_get_all()
        .await
        .unwrap()
        .into_iter()
        .find(|v| !v.used && v.email == "prime@heyadora.com")
        .unwrap()
        .id;
    let invite_key = id_to_string(invite_key);

    let state = RegisterFormState::new();
    state
        .run_register(
            client,
            "he",
            invite_key,
            "1234*455677889dfgdf",
            "1234*455677889GDdfgdf",
        )
        .await;
    assert_eq!(state.err_general.get_untracked(), "");
    assert_eq!(state.err_invite_key.get_untracked(), "");
    assert!(!state.err_username.get_untracked().is_empty());
    assert!(!state.err_password.get_untracked().is_empty());
    // assert!(state.has_err());

    let state = RegisterFormState::new();
    state
        .run_register(
            client,
            "hey",
            "invalid",
            "1234*455677889GDdfgdf",
            "1234*3455677889GDdfgdf",
        )
        .await;
    assert_eq!(state.err_general.get_untracked(), "");
    assert_eq!(state.err_invite_key.get_untracked(), "");
    assert_eq!(state.err_username.get_untracked(), "");
    assert!(!state.err_password.get_untracked().is_empty());

    // state.err_general.get().is_empty()
    //
}

pub fn invite_key_to_email(
    mut fn_invite_key: impl FnMut() -> String + 'static,
    errs: RwSignal<String>,
) -> RwSignal<String> {
    let email = RwSignal::new(String::new());

    Effect::new(move || {
        let client = create_client();
        let invite_key = fn_invite_key();

        spawn_local(async move {
            let result = client
                .invite_get_by_key(invite_key)
                .send()
                .await
                .into_json()
                .await;
            match result {
                Ok(res) => {
                    email.set(res.email);
                }
                Err(err) => {
                    errs.set(err.to_string());
                }
            }
        });
        // spawner.spawn(||);

        // invi
    });

    email
}

#[component]
pub fn RegisterForm() -> impl IntoView {
    let page = PageState::get();
    let reg = RegisterFormState::new();
    let spawner = Spawner::new();
    let input_username = NodeRef::new();
    let input_invite_key = RwQuery::<String>::new("token");
    let email = invite_key_to_email(
        move || input_invite_key.get().unwrap_or_default(),
        reg.err_invite_key,
    );
    // let email = Memo::new()
    let input_password = NodeRef::new();
    let input_password_confirmation = NodeRef::new();
    let on_register = move |e: web_sys::SubmitEvent| {
        e.prevent_default();

        let (Some(username), Some(invite_key), Some(password), Some(password_confirmation)) = (
            input_username
                .get_untracked()
                .map(|v: HtmlInputElement| v.value()),
            input_invite_key.get_untracked(),
            input_password
                .get_untracked()
                .map(|v: HtmlInputElement| v.value()),
            input_password_confirmation
                .get_untracked()
                .map(|v: HtmlInputElement| v.value()),
        ) else {
            return;
        };

        let client = create_client();

        spawner.spawn(async move {
            reg.run_register(
                &client,
                username,
                invite_key,
                password,
                password_confirmation,
            )
            .await;
            if !reg.has_no_err() {
                return;
            }
            page.update_auth().await;
        });
    };

    let email_placeholder = move || {
        if reg.err_invite_key.with(|v| v.is_empty()) {
            "loading..."
        } else {
            "error"
        }
    };

    view! {
        <form method="POST" action="" on:submit=on_register class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full ")>
            <h1 class="text-[1.5rem]  text-center my-[4rem]">"FINISH REGISTRATION"</h1>
            <Errs error=move||reg.err_general.get()/>
            <div class="flex flex-col justify-center gap-[3rem]">
                <div class="flex flex-col gap-0">
                    <label for="username" class="text-[1.2rem] ">"Username"</label>
                    <Errs error=move||reg.err_username.get()/>
                    <input placeholder="Alice" id="username" node_ref=input_username type="text" class="border-b-2 border-base05 w-full mt-1 " />
                </div>
                <div class="fex flex-col gap-0">
                    <label for="email_reg" class="text-[1.2rem] ">"Email"</label>
                    <Errs error=move||reg.err_invite_key.get()/>
                    <input value=move|| email.get() readonly placeholder=email_placeholder id="email_reg" type="text" class="border-b-2 border-base05 w-full mt-1 " />
                </div>
                <div class="flex flex-col gap-0">
                    <label for="password" class="text-[1.2rem] ">"Password"</label>
                    <Errs error=move||reg.err_password.get()/>
                    <input id="password" node_ref=input_password type="password" class="border-b-2 border-base05 w-full mt-1 " />
                </div>
                <div class="flex flex-col gap-0">
                    <label for="password_confirmation" class="text-[1.3rem] ">"Password Confirmation"</label>
                    <input id="password_confirmation" node_ref=input_password_confirmation type="password" class="border-b-2 border-base05 w-full mt-1 " />
                </div>
            </div>
            <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                <input type="submit" value="Register" class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
            </div>
        </form>
    }
}
