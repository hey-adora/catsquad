use crate::state::AppState;
use catsquad_db::DbUserGetByUsernameErr;
use catsquad_log::prelude::*;
use catsquad_shared::{ADMIN_ROLE_NAME, RoleValue, role_extract_permission_role_add};

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum PromoteAdminErr {
    #[error("option not set")]
    OptionNotSet,

    #[error("user not found")]
    UserNotFound,

    #[error("internal server err")]
    InternalServer,
}

/// returns true if admin is promoted succesfully or admin_username in conf not set
pub async fn promote_admin(state: AppState) -> Result<i64, PromoteAdminErr> {
    let inner = async || {
        let admin_username = state.get_admin_username().await;
        if admin_username.is_empty() {
            return Err(PromoteAdminErr::OptionNotSet);
        }

        // get/create admin role
        let admin_role_id = {
            let result = state.db.role_get_all().await;
            let roles = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("sometng exploded in promote_admin {err}");
                    return Err(PromoteAdminErr::InternalServer);
                }
            };

            let admin_role_exists = roles.iter().find(|v| v.name == ADMIN_ROLE_NAME);
            let admin_role_id = if let Some(admin_role) = admin_role_exists {
                // check if contains correct permissions
                let contains_role_add = role_extract_permission_role_add(&admin_role.values);
                if !contains_role_add {
                    let result = state
                        .db
                        .role_update_value(0, admin_role.id, vec![RoleValue::PermissionRoleAdd])
                        .await;
                    match result {
                        Ok(v) => v,
                        Err(err) => {
                            error!("sometng exploded in promote_admin {err}");
                            return Err(PromoteAdminErr::InternalServer);
                        }
                    };
                }

                admin_role.id
            } else {
                // create new admin role
                let result = state
                    .db
                    .role_add(0, ADMIN_ROLE_NAME, vec![RoleValue::PermissionRoleAdd])
                    .await;
                let admin_role = match result {
                    Ok(v) => v,
                    Err(err) => {
                        error!("sometng exploded in promote_admin {err}");
                        return Err(PromoteAdminErr::InternalServer);
                    }
                };

                admin_role.id
            };

            admin_role_id
        };

        // promote user if not admin already
        {
            let result = state.db.user_get_by_username("", &admin_username).await;
            let admin_user = match result {
                Ok(v) => v,
                Err(DbUserGetByUsernameErr::NotFound) => return Err(PromoteAdminErr::UserNotFound),
                Err(DbUserGetByUsernameErr::Db(err)) => {
                    error!("something exploded in promote_admin {err}");
                    return Err(PromoteAdminErr::InternalServer);
                }
            };

            let is_admin = admin_user.roles.contains(&admin_role_id);
            if !is_admin {
                let result = state
                    .db
                    .user_update_role_add(0, admin_username, ADMIN_ROLE_NAME)
                    .await;
                match result {
                    Ok(v) => v,
                    Err(err) => {
                        error!("sometng exploded in promote_admin {err}");
                        return Err(PromoteAdminErr::UserNotFound);
                    }
                };
            }

            Ok(admin_role_id)
        }
    };

    let result = inner().await;

    let mut keep_checking = state.keep_check_if_admin_promoted.write().unwrap();
    match result {
        Err(PromoteAdminErr::UserNotFound) => {
            *keep_checking = true;
        }
        Err(PromoteAdminErr::InternalServer) | Err(PromoteAdminErr::OptionNotSet) | Ok(_) => {
            *keep_checking = false;
        }
    };

    result
}

#[cfg(test)]
#[tokio::test]
async fn test_promote_admin_empty() {
    init_log();
    let server = crate::TestServer::new(0, "test_promote_admin_empty").await;

    server.state.set_admin_username("hey").await;

    // assert - make sure it doesnt promote random people
    {
        let invite = server
            .state
            .db
            .invite_add(0, "hey2@heyadora.com", 10)
            .await
            .unwrap();
        let user1 = server
            .user_add("hey2", invite.token, "EN)#r0rhw0efnh3h0r4")
            .await
            .unwrap();

        assert_eq!(user1.roles_ids.len(), 0);
    }

    // assert success
    {
        let keep_checking = server.state.get_keep_checking_if_admin_promoted();
        assert!(keep_checking);

        let invite = server
            .state
            .db
            .invite_add(0, "hey@heyadora.com", 10)
            .await
            .unwrap();
        let user1 = server
            .user_add("hey", invite.token, "EN)#r0rhw0efnh3h0r4")
            .await
            .unwrap();

        let admin_role = server
            .state
            .db
            .role_get_all()
            .await
            .unwrap()
            .into_iter()
            .find(|v| v.name == ADMIN_ROLE_NAME)
            .unwrap();

        let keep_checking = server.state.get_keep_checking_if_admin_promoted();

        assert!(!keep_checking);
        assert_eq!(user1.roles_ids.len(), 1);
        assert_eq!(
            admin_role.values,
            vec![RoleValue::PermissionRoleAdd.to_string()]
        );
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_promote_admin_update_role() {
    init_log();
    let server = crate::TestServer::new(0, "test_promote_admin_update_role").await;

    server.state.set_admin_username("hey").await;

    // assert success
    {
        server
            .state
            .db
            .role_add(0, ADMIN_ROLE_NAME, Vec::<String>::new())
            .await
            .unwrap();

        let keep_checking = server.state.get_keep_checking_if_admin_promoted();
        assert!(keep_checking);

        let invite = server
            .state
            .db
            .invite_add(0, "hey@heyadora.com", 10)
            .await
            .unwrap();
        let user1 = server
            .user_add("hey", invite.token, "EN)#r0rhw0efnh3h0r4")
            .await
            .unwrap();

        let admin_role = server
            .state
            .db
            .role_get_all()
            .await
            .unwrap()
            .into_iter()
            .find(|v| v.name == ADMIN_ROLE_NAME)
            .unwrap();

        let keep_checking = server.state.get_keep_checking_if_admin_promoted();

        assert!(!keep_checking);
        assert_eq!(user1.roles_ids.len(), 1);
        assert_eq!(
            admin_role.values,
            vec![RoleValue::PermissionRoleAdd.to_string()]
        );
    }
}
