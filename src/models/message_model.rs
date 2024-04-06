use std::cmp::Ordering;

use super::user_model::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WsPayload {
    pub op_code: u8,
    pub message: String,
}

impl WsPayload {
    pub fn new(op_code: u8, message: String) -> Self {
        Self { op_code, message }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MsgData {
    pub msg_uuid: String,
    pub msg_sender: String,
    pub channel: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

impl MsgData {
    pub fn new(
        channel: String,
        msg_sender: String,
        message: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        let msg_uuid = Uuid::new_v4().as_simple().to_string();
        Self {
            msg_uuid,
            msg_sender,
            channel,
            message,
            created_at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Default)]
pub struct MsgResponse {
    pub msg_uuid: String,
    pub msg_sender: Option<User>,
    pub channel: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

impl PartialEq for MsgResponse {
    fn eq(&self, other: &Self) -> bool {
        self.msg_uuid == other.msg_uuid
    }
}

impl Ord for MsgResponse {
    fn cmp(&self, other: &Self) -> Ordering {
        self.created_at.cmp(&other.created_at)
    }
}

impl PartialOrd for MsgResponse {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "ssr")]
use {crate::state::db::Database, futures::future::join_all};

#[cfg(feature = "ssr")]
impl MsgResponse {
    async fn from_msg_data(msg_data: &MsgData, pool: &Database) -> Self {
        let query_user = pool
            .client
            .query("SELECT * FROM type::table($table) WHERE uuid = $uuid")
            .bind(("table", "user_data"))
            .bind(("uuid", &msg_data.msg_sender))
            .await;

        let maybe_user = match query_user {
            Ok(mut maybe_user) => maybe_user.take::<Option<User>>(0).unwrap_or(None),
            Err(_) => None,
        };

        Self {
            msg_uuid: msg_data.msg_uuid.clone(),
            msg_sender: maybe_user,
            channel: msg_data.channel.clone(),
            message: msg_data.message.clone(),
            created_at: msg_data.created_at.clone(),
        }
    }

    pub async fn get_all_msg(
        room_uuid: &str,
        pool: &Database,
    ) -> Result<Vec<Self>, surrealdb::Error> {
        match pool
            .client
            .query("SELECT * FROM type::table($table) WHERE channel = $channel")
            .bind(("table", "message"))
            .bind(("channel", room_uuid))
            .await
        {
            Ok(mut query_result) => {
                let response = query_result.take::<Vec<MsgData>>(0);
                let result = if let Ok(vec_msg) = response {
                    let future_vec = vec_msg
                        .iter()
                        .map(|msg| async { Self::from_msg_data(msg, &pool).await });
                    join_all(future_vec).await
                } else {
                    Vec::new()
                };

                Ok(result)
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(feature = "ssr")]
impl MsgData {
    pub async fn into_msg_response(&self, pool: &Database) -> MsgResponse {
        MsgResponse::from_msg_data(self, pool).await
    }

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

    async fn _get_from_uuid(msg_uuid: &str, pool: &Database) -> Option<Self> {
        match pool
            .client
            .query("SELECT * FROM type::table($table) WHERE msg_uuid = $msg_uuid")
            .bind(("table", "message"))
            .bind(("msg_uuid", msg_uuid))
            .await
        {
            Ok(mut query_result) => query_result.take::<Option<Self>>(0).unwrap_or(None),
            Err(_) => None,
        }
    }

    async fn _get_all_msg(room_uuid: &str, pool: &Database) -> Result<Vec<Self>, surrealdb::Error> {
        match pool
            .client
            .query("SELECT * FROM type::table($table) WHERE channel = $channel")
            .bind(("table", "message"))
            .bind(("channel", room_uuid))
            .await
        {
            Ok(mut query_result) => Ok(query_result.take::<Vec<Self>>(0).unwrap_or(Vec::new())),
            Err(err) => Err(err),
        }
    }

    async fn _stream_msg(
        room_uuid: &str,
        pool: &Database,
    ) -> Result<surrealdb::method::QueryStream<surrealdb::Notification<Self>>, surrealdb::Error>
    {
        let mut response = pool
            .client
            .query("LIVE SELECT * FROM message WHERE channel = $channel")
            .bind(("channel", room_uuid))
            .await?;

        response.stream::<surrealdb::Notification<Self>>(0)
    }
}
