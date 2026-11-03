use crate::{Db, DbTokenUpdateConfirmErr, query::user_remove_tx::DbUserRemoveErr};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserRemoveByUsernamesErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("unauthorized")]
    Unauthorized,

    #[error(transparent)]
    Image(#[from] DbUserRemoveErr),
}

impl Db {
    pub async fn user_remove_by_usernames<TCallback>(
        &self,
        time: u64,
        callback_remove_image: TCallback,
        user_usernames: Vec<String>,
    ) -> Result<(), DbUserRemoveByUsernamesErr>
    where
        TCallback: AsyncFnMut(i64, &str) + Clone,
    {
        let pool = &self.db;

        // TODO ignore not_found errors

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("user_remove_by_token {err}"))?;

        for username in user_usernames {
            self.user_remove_tx(&mut *tx, time, callback_remove_image.clone(), username)
                .await?;
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("user_remove_by_token {err}"))?;

        Ok(())
    }
}

// TODO add test
