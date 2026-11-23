use crate::{Db, DbUserSearch, DbUserSearchBuilderErr};
use catsquad_shared::{Order, SessionTokenKind, TimeRange, UserSearchFields};

impl Db {
    pub async fn user_search_users(
        &self,
        auth_session_token: SessionTokenKind,
        time_range_value: u64,
        limit: usize,
        offset: usize,
        time_range: TimeRange,
        order: Order,
        fields: UserSearchFields,
        query: impl Into<String>,
    ) -> Result<Vec<DbUserSearch>, DbUserSearchBuilderErr> {
        let result = self
            .user_search_builder(
                auth_session_token,
                false,
                time_range_value,
                limit,
                offset,
                time_range,
                order,
                fields,
                query,
            )
            .await;

        match result {
            Ok(crate::DbUserSearchBuilder::Users(users)) => Ok(users),
            Ok(crate::DbUserSearchBuilder::PageCount(_)) => {
                unreachable!("invalid quary, should only return users")
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_search_users() {
    use crate::user_add_with_roles;
    use catsquad_log::prelude::*;
    use catsquad_shared::RoleValue;

    init_log();

    let db = Db::test_db(0, "test_user_search_users").await;

    let create_user = async |i: usize| {
        let username = format!("hey{i}");
        let email = format!("hey{i}@heyadora.com");
        user_add_with_roles(
            &db,
            i as u64,
            10,
            10,
            username,
            "pss",
            email,
            vec![] as Vec<RoleValue>,
        )
        .await
    };

    let (_user0, token0) = create_user(0).await;

    for i in 1..10 {
        let _ = create_user(i).await;
    }

    // assert success
    {
        let users = db
            .user_search_users(
                SessionTokenKind::Root,
                0,
                10,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "",
            )
            .await
            .unwrap();
        assert_eq!(users.len(), 10);
        assert_eq!(users[0].username, "hey0");
        assert_eq!(users[9].username, "hey9");

        let users = db
            .user_search_users(
                SessionTokenKind::Root,
                0,
                2,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "",
            )
            .await
            .unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "hey0");
        assert_eq!(users[1].username, "hey1");

        let users = db
            .user_search_users(
                SessionTokenKind::Root,
                1,
                2,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "",
            )
            .await
            .unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "hey1");
        assert_eq!(users[1].username, "hey2");

        let users = db
            .user_search_users(
                SessionTokenKind::Root,
                1,
                2,
                2,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "",
            )
            .await
            .unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "hey3");
        assert_eq!(users[1].username, "hey4");

        let users = db
            .user_search_users(
                SessionTokenKind::Root,
                0,
                2,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "%hey3%",
            )
            .await
            .unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "hey3");

        let users = db
            .user_search_users(
                SessionTokenKind::Root,
                0,
                2,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "%hey_ %",
            )
            .await
            .unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "hey0");
        assert_eq!(users[1].username, "hey1");
    }

    // assert error unauthorized
    {
        let search = async |token: SessionTokenKind| {
            db.user_search_users(
                token,
                0,
                10,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "",
            )
            .await
        };

        let result = search(SessionTokenKind::None).await;
        assert!(matches!(
            result,
            Err(DbUserSearchBuilderErr::Unauthorized(_))
        ));

        let result = search(token0).await;
        assert!(matches!(
            result,
            Err(DbUserSearchBuilderErr::Unauthorized(_))
        ));
    }
}
