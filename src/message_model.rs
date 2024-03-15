use crate::user_model::User;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Msg {
    Text(String),
    Bytes(Vec<u8>),
    // Json { sender: String, message: Msg },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MsgData {
    pub msg_uuid: String,
    pub msg_sender: User,
    pub channel: String,
    pub message: Msg,
}

impl MsgData {
    pub fn new(channel: String, msg_sender: User, message: Msg) -> Self {
        let msg_uuid = Uuid::new_v4().as_simple().to_string();
        Self {
            msg_uuid,
            msg_sender,
            channel,
            message,
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    pub use super::MsgData;
    use crate::db::Database;

    impl MsgData {
        pub async fn insert_into_db(&self, pool: &Database) -> Result<(), surrealdb::Error> {
            match pool
                .client
                .create::<Option<Self>>(("message", self.msg_uuid.clone()))
                .content(self)
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }

        pub async fn get_from_uuid(msg_uuid: &str, pool: &Database) -> Option<Self> {
            match pool
                .client
                .query("SELECT * FROM type::table($table) WHERE msg_uuid = $msg_uuid")
                .bind(("table", "messages"))
                .bind(("msg_uuid", msg_uuid))
                .await
            {
                Ok(mut maybe_msg) => maybe_msg.take::<Option<Self>>(0).unwrap_or(None),
                Err(_) => None,
            }
        }
    }
}
