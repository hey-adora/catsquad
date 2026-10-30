use crate::{Db, DbUserSearch, DbUserSearchBuilderErr};
use catsquad_shared::{Order, TimeRange};

impl Db {
    pub async fn user_search_pagination(
        &self,
        time_range_value: u64,
        limit: usize,
        offset: usize,
        time_range: TimeRange,
        order: Order,
    ) -> Result<u32, DbUserSearchBuilderErr> {
        let result = self
            .user_search_builder(true, time_range_value, limit, offset, time_range, order)
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

    init_log();

    let db = Db::test_db(0, "test_user_search_pagination").await;

    for i in 0..10 {
        let email = format!("hey{i}@hey.com");
        let username = format!("hey{i}");
        let invite = db.invite_add(0, email, 10).await.unwrap();
        let _user = db
            .user_add(i, username, "pss", invite.token.clone(), 10, 10)
            .await
            .unwrap();
    }

    let pages = db
        .user_search_pagination(0, 10, 0, TimeRange::MoreOrEqual, Order::OneTwoThree)
        .await
        .unwrap();
    assert_eq!(pages, 10);

    let pages = db
        .user_search_pagination(0, 10, 2, TimeRange::MoreOrEqual, Order::OneTwoThree)
        .await
        .unwrap();
    assert_eq!(pages, 8);
}
