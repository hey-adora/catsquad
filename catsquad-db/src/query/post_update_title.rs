use crate::{Db, DbPostUpdateBuilderTextErr, DbPostUpdateBuilderTextField};

impl Db {
    pub async fn post_update_title(
        &self,
        time: u64,
        user_username: impl Into<String>,
        post_id: i64,
        new_title: impl Into<String>,
    ) -> Result<(), DbPostUpdateBuilderTextErr> {
        self.post_update_builder_text(
            time,
            user_username,
            post_id,
            DbPostUpdateBuilderTextField::Title,
            new_title,
        )
        .await
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_update_title() {
    use catsquad_log::prelude::*;
    use catsquad_shared::PostState;
    init_log();

    let db = Db::test_db(0, "test_post_update_title").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.token, 10, 10)
        .await
        .unwrap();

    let invite1 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey", invite1.token, 10, 10)
        .await
        .unwrap();

    let post1 = {
        let post1 = db
            .post_add(0, user.username.clone(), "title1", "description1", "tags")
            .await
            .unwrap();
        db.post_update_state(0, user.username.clone(), post1.id, PostState::Active)
            .await
            .unwrap();
        assert_eq!(post1.title, "title1");
        post1
    };

    let post1 = {
        db.post_update_title(0, user.username.clone(), post1.id, "title2")
            .await
            .unwrap();
        let post1 = db
            .post_get_by_id(user.username.clone(), post1.id)
            .await
            .unwrap();
        assert_eq!(post1.title, "title2");
        post1
    };

    let result = db.post_update_title(0, "invalid", post1.id, "title2").await;
    assert!(matches!(
        result,
        Err(DbPostUpdateBuilderTextErr::Unauthorized)
    ));

    let result = db
        .post_update_title(0, user2.username.clone(), post1.id.clone(), "title2")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateBuilderTextErr::Unauthorized)
    ));

    let result = db
        .post_update_title(0, user.username.clone(), 0, "title2")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateBuilderTextErr::PostNotFound)
    ));
}
