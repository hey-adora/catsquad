use crate::{Db, DbUserSearch, DbUserSearchBuilderErr};
use catsquad_shared::{Order, TimeRange};

impl Db {
    pub async fn user_search_users(
        &self,
        time_range_value: u64,
        limit: usize,
        offset: usize,
        time_range: TimeRange,
        order: Order,
    ) -> Result<Vec<DbUserSearch>, DbUserSearchBuilderErr> {
        let result = self
            .user_search_builder(false, time_range_value, limit, offset, time_range, order)
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
    use catsquad_log::prelude::*;

    init_log();

    let db = Db::test_db(0, "test_user_search_users").await;

    for i in 0..10 {
        let email = format!("hey{i}@hey.com");
        let username = format!("hey{i}");
        let invite = db.invite_add(0, email, 10).await.unwrap();
        let _user = db
            .user_add(i, username, "pss", invite.token.clone(), 10, 10)
            .await
            .unwrap();
    }

    let users = db
        .user_search_users(0, 10, 0, TimeRange::MoreOrEqual, Order::OneTwoThree)
        .await
        .unwrap();
    assert_eq!(users.len(), 10);
    assert_eq!(users[0].username, "hey0");
    assert_eq!(users[9].username, "hey9");

    let users = db
        .user_search_users(0, 2, 0, TimeRange::MoreOrEqual, Order::OneTwoThree)
        .await
        .unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].username, "hey0");
    assert_eq!(users[1].username, "hey1");

    let users = db
        .user_search_users(1, 2, 0, TimeRange::MoreOrEqual, Order::OneTwoThree)
        .await
        .unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].username, "hey1");
    assert_eq!(users[1].username, "hey2");

    let users = db
        .user_search_users(1, 2, 2, TimeRange::MoreOrEqual, Order::OneTwoThree)
        .await
        .unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].username, "hey3");
    assert_eq!(users[1].username, "hey4");
}
