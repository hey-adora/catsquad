use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{RoleRes, RoleValue};
use leptos::prelude::*;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct RolesState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub err_general: RwSignal<String>,
    pub roles: RwSignal<Vec<RoleRes>>,

    pub check_all: RwSignal<bool>,
    pub checked_roles: RwSignal<Vec<i64>>,

    pub client: StoredValue<Client<TSender>, LocalStorage>,
}

impl<TSender> Copy for RolesState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
}

impl<TSender> RolesState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            err_general: RwSignal::new(String::new()),
            roles: RwSignal::new(Vec::new()),
            check_all: RwSignal::new(false),
            checked_roles: RwSignal::new(Vec::new()),
            client: StoredValue::new_local(client),
        }
    }

    pub async fn init(&self) {
        let client = self.client.get_value();

        self.err_general.update(|v| v.clear());

        let result = client.role_get_all().send().await.into_json().await;
        let roles = match result {
            Ok(v) => v,
            Err(err) => {
                error!("{}", err);
                self.err_general.set(err.to_string());
                return;
            }
        };

        self.roles.set(roles);
    }

    pub async fn add_role(&self, name: impl Into<String>, role_values: Vec<RoleValue>) {
        let name = name.into();
        let client = self.client.get_value();

        self.err_general.update(|v| v.clear());

        let result = client
            .role_add(name, role_values)
            .send()
            .await
            .into_json()
            .await;

        let role = match result {
            Ok(v) => v,
            Err(err) => {
                error!("{}", err);
                self.err_general.set(err.to_string());
                return;
            }
        };

        self.roles.update(|v| v.push(role));
    }

    pub async fn remove_role(&self, role_id: i64) {
        let client = self.client.get_value();

        self.err_general.update(|v| v.clear());

        let result = client.role_remove(role_id).send().await.into_json().await;

        match result {
            Ok(_) => (),
            Err(err) => {
                error!("{}", err);
                self.err_general.set(err.to_string());
                return;
            }
        };

        self.roles.update(|v| {
            let Some(pos) = v.iter().position(|v| v.id == role_id) else {
                return;
            };
            v.remove(pos);
        });
    }

    pub async fn update_role(&self, role_id: i64) {
        // let client = self.client.get_value();

        // self.err_general.update(|v| v.clear());

        // let result = client.role_remove(role_id).send().await.into_json().await;

        // match result {
        //     Ok(_) => (),
        //     Err(err) => {
        //         error!("{}", err);
        //         self.err_general.set(err.to_string());
        //         return;
        //     }
        // };

        // self.roles.update(|v| {
        //     let Some(pos) = v.iter().position(|v| v.id == role_id) else {
        //         return;
        //     };
        //     v.remove(pos);
        // });
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_roles_state() {
    use catsquad_api::{TestServer, auth::create_auth_cookie_str};
    use catsquad_shared::uuid_to_str;
    use http::header;

    init_log();

    let _owner = crate::init_owner();
    let server = TestServer::new(0, "test_roles_state").await;

    let create_user = async |i: usize| {
        server.state.set_time(i as u64);
        let (user, session) = server
            .user_add_full(
                format!("prime{i}"),
                format!("prime{i}@heyadora.com"),
                "235j4t49ngerigrog#IOTNOnfo",
            )
            .await;

        (user, session)
    };

    let (_user0, token0) = create_user(0).await;
    server.admin("prime0").await;
    server
        .inject_header(header::COOKIE, create_auth_cookie_str(uuid_to_str(token0)))
        .await;

    let roles_state = RolesState::new(server.client.clone());

    roles_state.init().await;

    assert_eq!(roles_state.roles.get_untracked().len(), 1);
    assert_eq!(roles_state.roles.get_untracked()[0].name, "admin");
    assert!(roles_state.err_general.get_untracked().is_empty());

    roles_state.add_role("mod", vec![]).await;

    assert_eq!(roles_state.roles.get_untracked().len(), 2);
    assert_eq!(roles_state.roles.get_untracked()[0].name, "admin");
    assert_eq!(roles_state.roles.get_untracked()[1].name, "mod");
    assert!(roles_state.err_general.get_untracked().is_empty());

    roles_state
        .add_role("mod2", vec![RoleValue::PermissionRoleAdd])
        .await;

    assert_eq!(roles_state.roles.get_untracked().len(), 3);
    assert_eq!(roles_state.roles.get_untracked()[0].name, "admin");
    assert_eq!(roles_state.roles.get_untracked()[1].name, "mod");
    assert_eq!(roles_state.roles.get_untracked()[2].name, "mod2");
    assert_eq!(roles_state.roles.get_untracked()[2].values.len(), 1);
    assert_eq!(
        roles_state.roles.get_untracked()[2].values[0],
        RoleValue::PermissionRoleAdd.as_str()
    );
    assert!(roles_state.err_general.get_untracked().is_empty());

    // roles_state.add_role("mod", role_values)

    // server
    //     .role_add("admin", vec![RoleValue::PermissionRoleAdd], token0)
    // .await;

    //
}
