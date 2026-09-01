use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;
use catsquad_shared::PostState;

#[derive(Debug, thiserror::Error)]
pub enum DbPostUpdateStateErr {
    #[error("same state")]
    SameState,

    #[error("cant set draft")]
    CantSetDraft,

    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn post_update_state(
        &self,
        time: u64,
        user_username: String,
        post_id: i64,
        new_state: PostState,
    ) -> Result<(), DbPostUpdateStateErr> {
        if new_state == PostState::Draft {
            return Err(DbPostUpdateStateErr::CantSetDraft);
        }

        let pool = &self.db;

        let mut tx = pool.begin().await?;

        let query = "SELECT post_state, post_user_username FROM posts WHERE post_id = $1";

        let result = sqlx::query_as(query)
            .bind(post_id)
            .fetch_one(&mut *tx)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbPostUpdateStateErr::PostNotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostUpdateStateErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");

        let (post_state, post_user_username): (String, String) = result;

        if new_state == post_state {
            return Err(DbPostUpdateStateErr::SameState);
        }

        if user_username != post_user_username {
            return Err(DbPostUpdateStateErr::Unauthorized);
        }

        let query = "UPDATE posts SET
                            post_state = $1,
                            post_modified_at = $2
                            WHERE post_id = $3";

        debug!("about to run {query}");

        let result = sqlx::query(query)
            .bind(new_state.as_str())
            .bind(XTimestamp(time as i64))
            .bind(post_id)
            .execute(&mut *tx)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostUpdateStateErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");

        let affected = result.rows_affected();
        if affected != 1 {
            tx.rollback().await.unwrap();
            panic!(
                "something is wrong, updated wrong number of rows, expected 1, got {}, query {}, params {} {} {}",
                affected, query, new_state, time, post_id
            );
        }

        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_update_state() {
    init_log();

    let db = Db::test_db(0, "test_post_update_state").await;

    let (user1, user2) = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user1 = db
            .user_add(0, "hey", "hey", invite1.token.clone(), 10, 10)
            .await
            .unwrap();

        let invite1 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        let user2 = db
            .user_add(0, "hey2", "hey", invite1.token.clone(), 10, 10)
            .await
            .unwrap();

        (user1, user2)
    };

    let post1 = db
        .post_add(0, user1.username.clone(), "title", "description", "")
        .await
        .unwrap();
    assert_eq!(post1.state, PostState::Draft.to_string());

    // error assert
    {
        let result = db
            .post_update_state(0, user2.username.clone(), post1.id, PostState::Active)
            .await;
        assert!(matches!(result, Err(DbPostUpdateStateErr::Unauthorized)));
    }

    // error assert
    {
        let result = db
            .post_update_state(0, user1.username.clone(), post1.id, PostState::Draft)
            .await;
        assert!(matches!(result, Err(DbPostUpdateStateErr::CantSetDraft)));
    }

    // success assert
    {
        let result = db
            .post_update_state(0, user1.username.clone(), post1.id, PostState::Hidden)
            .await;
        assert!(matches!(result, Ok(_)));
    }

    // success assert
    let post1 = {
        db.post_update_state(0, user1.username.clone(), post1.id, PostState::Active)
            .await
            .unwrap();
        let post1 = db
            .post_get_by_id(user1.username.clone(), post1.id)
            .await
            .unwrap();
        assert_eq!(post1.state, PostState::Active.to_string());

        post1
    };

    let post2 = {
        let post2 = db
            .post_add(0, user1.username.clone(), "title2", "description2", "")
            .await
            .unwrap();
        db.post_update_state(0, user1.username.clone(), post2.id, PostState::Active)
            .await
            .unwrap();
        post2
    };

    // success assert
    let post1 = {
        db.post_update_state(0, user1.username.clone(), post1.id, PostState::Hidden)
            .await
            .unwrap();
        let post1 = db
            .post_get_by_id(user1.username.clone(), post1.id.clone())
            .await
            .unwrap();

        // make sure only 1 row was updated
        {
            let post2 = db
                .post_get_by_id(user1.username.clone(), post2.id)
                .await
                .unwrap();

            assert_eq!(post2.state, PostState::Active.to_string());
        }

        post1
    };

    // error assert
    {
        let result = db
            .post_update_state(0, user1.username.clone(), post1.id, PostState::Draft)
            .await;
        assert!(matches!(result, Err(DbPostUpdateStateErr::CantSetDraft)));
    }

    // error assert
    {
        let result = db
            .post_update_state(0, user1.username.clone(), post1.id, PostState::Hidden)
            .await;
        assert!(matches!(result, Err(DbPostUpdateStateErr::SameState)));
    }

    // success assert
    {
        db.post_update_state(0, user1.username.clone(), post1.id, PostState::Active)
            .await
            .unwrap();
    }
}
