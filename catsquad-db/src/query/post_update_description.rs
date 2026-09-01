use crate::{Db, DbPostUpdateBuilderTextErr, DbPostUpdateBuilderTextField};

impl Db {
    pub async fn post_update_description(
        &self,
        time: u64,
        user_username: impl Into<String>,
        post_id: i64,
        new_description: impl Into<String>,
    ) -> Result<(), DbPostUpdateBuilderTextErr> {
        self.post_update_builder_text(
            time,
            user_username,
            post_id,
            DbPostUpdateBuilderTextField::Description,
            new_description,
        )
        .await
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_update_description() {
    use catsquad_log::init_log;

    init_log();

    let db = Db::test_db(0, "test_post_update_description").await;

    let (user, user2) = {
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

        (user, user2)
    };

    let (post1, post2) = {
        use catsquad_shared::PostState;

        let post1 = db
            .post_add(0, user.username.clone(), "title1", "description1", "tags")
            .await
            .unwrap();

        db.post_update_state(0, user.username.clone(), post1.id, PostState::Active)
            .await
            .unwrap();

        let post2 = db
            .post_add(0, user.username.clone(), "title1", "description4", "tags")
            .await
            .unwrap();

        db.post_update_state(0, user.username.clone(), post2.id, PostState::Active)
            .await
            .unwrap();

        (post1, post2)
    };

    assert_eq!(post1.description, "description1");

    // success assert
    {
        db.post_update_description(0, user.username.clone(), post1.id, "description2")
            .await
            .unwrap();
        let post1 = db
            .post_get_by_id(user.username.clone(), post1.id)
            .await
            .unwrap();
        assert_eq!(post1.description, "description2");

        // make sure it only updated single row
        let post2 = db
            .post_get_by_id(user.username.clone(), post2.id)
            .await
            .unwrap();
        assert_eq!(post2.description, "description4");
    }

    // error assert
    {
        let result = db
            .post_update_description(0, user2.username.clone(), post1.id, "description2")
            .await;
        assert!(matches!(
            result,
            Err(DbPostUpdateBuilderTextErr::Unauthorized)
        ));
    }

    // error assert
    {
        let result = db
            .post_update_description(0, user.username.clone(), 0, "description2")
            .await;
        assert!(matches!(
            result,
            Err(DbPostUpdateBuilderTextErr::PostNotFound)
        ));
    }
}
