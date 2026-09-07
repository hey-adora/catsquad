use catsquad_shared::proccess_tags;

use crate::{Db, DbPostUpdateBuilderTextErr, DbPostUpdateBuilderTextField};

impl Db {
    pub async fn post_update_tags(
        &self,
        time: u64,
        user_username: impl Into<String>,
        post_id: i64,
        new_tags: impl Into<String>,
    ) -> Result<(), DbPostUpdateBuilderTextErr> {
        let new_tags = proccess_tags(new_tags);

        self.post_update_builder_text(
            time,
            user_username,
            post_id,
            DbPostUpdateBuilderTextField::Tags,
            new_tags,
        )
        .await
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_update_tags() {
    use catsquad_log::prelude::*;
    use catsquad_shared::PostState;
    init_log();

    let db = Db::test_db(0, "test_post_update_tags").await;

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
        assert_eq!(post1.tags, " tags ");

        post1
    };

    let post1 = {
        db.post_update_tags(0, user.username.clone(), post1.id, " tags2   Tag3")
            .await
            .unwrap();
        let post1 = db
            .post_get_by_id(user.username.clone(), post1.id)
            .await
            .unwrap();
        assert_eq!(post1.tags, " tags2 tag3 ");
        post1
    };

    let result = db.post_update_tags(0, "invalid", post1.id, "tags2").await;
    assert!(matches!(
        result,
        Err(DbPostUpdateBuilderTextErr::Unauthorized)
    ));

    let result = db
        .post_update_tags(0, user2.username.clone(), post1.id, "tags2")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateBuilderTextErr::Unauthorized)
    ));

    let result = db
        .post_update_tags(0, user.username.clone(), 0, "tags2")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateBuilderTextErr::PostNotFound)
    ));
}
