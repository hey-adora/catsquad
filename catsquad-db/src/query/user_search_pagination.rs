use crate::{Db, DbUserSearchBuilderErr};
use catsquad_shared::{Order, SessionTokenKind, TimeRange, UserSearchFields};

impl Db {
    pub async fn user_search_pagination(
        &self,
        auth_session_token: SessionTokenKind,
        time_range_value: u64,
        limit: usize,
        offset: usize,
        time_range: TimeRange,
        order: Order,
        fields: UserSearchFields,
        query: impl Into<String>,
    ) -> Result<u32, DbUserSearchBuilderErr> {
        let result = self
            .user_search_builder(
                auth_session_token,
                true,
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
            Ok(crate::DbUserSearchBuilder::PageCount(pages)) => Ok(pages),
            Ok(crate::DbUserSearchBuilder::Users(_)) => {
                unreachable!("invalid quary, should only return pages")
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_search_pagination() {
    use catsquad_log::prelude::*;
    use catsquad_shared::RoleValue;

    use crate::user_add_with_roles;

    init_log();

    let db = Db::test_db(0, "test_user_search_pagination").await;

    let create_user = async |i: usize| {
        let username = format!("hey{i}");
        let email = format!("hey{i}@heyadora.com");
        user_add_with_roles(
            &db,
            0,
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
        let pages = db
            .user_search_pagination(
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
        assert_eq!(pages, 10);

        let pages = db
            .user_search_pagination(
                SessionTokenKind::Root,
                0,
                5,
                2,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "",
            )
            .await
            .unwrap();
        assert_eq!(pages, 10);
    }

    // assert error unauthorized
    {
        let search = async |token: SessionTokenKind| {
            db.user_search_pagination(
                token,
                0,
                5,
                2,
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
