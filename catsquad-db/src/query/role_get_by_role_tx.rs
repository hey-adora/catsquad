use crate::Db;
use catsquad_log::prelude::*;
use sqlx::PgConnection;

#[derive(Debug, thiserror::Error)]
pub enum DbRoleGetTxErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn role_get_by_role_ids_tx(
        &self,
        roles_ids: &[i64],
        tx: &mut PgConnection,
    ) -> Result<Vec<String>, DbRoleGetTxErr> {
        // get the role
        {
            let query = "SELECT
                                    role_id,
                                    role_values
                                FROM unnest($1)
                                INNER JOIN roles ON unnest=role_id
                                ";

            let result = sqlx::query_as(query)
                .bind(roles_ids)
                .fetch_all(&mut *tx)
                .await;

            debug!("{query}\n{result:#?}");

            let roles: Vec<(i64, Vec<String>)> = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbRoleGetTxErr::Db(err));
                }
            };

            let mut roles_values = Vec::new();

            for role_id_a in roles_ids {
                let Some((_, values)) = roles
                    .iter()
                    .find(|(role_id_b, _)| *role_id_a == *role_id_b)
                    .cloned()
                else {
                    error!(
                        "role missing for user, expected role ids {:?}, received: {:#?}",
                        roles_ids, roles
                    );
                    break;
                };

                roles_values.extend(values);
            }

            Ok(roles_values)
        }
    }
}
