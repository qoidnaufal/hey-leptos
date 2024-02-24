use crate::{db::Database, user_model::FilteredUser};
// use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Msg {
    uuid: String,
    sender: FilteredUser,
    message: String,
}

impl Msg {
    pub async fn new(uuid: String, sender: FilteredUser, message: String) -> Self {
        Self {
            uuid,
            sender,
            message,
        }
    }

    pub async fn insert_into_db(&self, pool: Database) -> Result<(), surrealdb::Error> {
        match pool
            .client
            .create::<Option<Self>>(("message", self.uuid.clone()))
            .content(self)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}
