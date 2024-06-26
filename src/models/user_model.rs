use chrono::{DateTime, Utc};
use leptos::*;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Avatar {
    Image { bytes: Vec<u8> },
    Initial { text: String },
}

impl Default for Avatar {
    fn default() -> Self {
        Self::Initial {
            text: "".to_string(),
        }
    }
}

impl Avatar {
    pub fn get_view(&self) -> impl IntoView {
        match self {
            Self::Initial { text: t } => t.clone().into_view(),
            Self::Image { bytes: b } => b.clone().into_view(),
        }
    }
}

// ---- user data as main data to store in db

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub uuid: String,
    pub user_name: String,
    pub email: String,
    pub password: String,
    pub joined_channels: Vec<String>,
    pub avatar: Avatar,
    pub created_at: DateTime<Utc>,
}

// ---- user to expose to the client

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Default)]
pub struct User {
    pub uuid: String,
    pub user_name: String,
    pub avatar: Avatar,
}

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl User {
    pub fn from_user_data(user_data: &UserData) -> Self {
        Self {
            uuid: user_data.uuid.clone(),
            user_name: user_data.user_name.clone(),
            avatar: user_data.avatar.clone(),
        }
    }
}

#[cfg(feature = "ssr")]
use crate::{error::ServerError, state::db::Database};

#[cfg(feature = "ssr")]
impl UserData {
    pub fn new(
        uuid: String,
        user_name: String,
        email: String,
        password: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        let mut initial = user_name.clone();
        initial.truncate(1);
        let avatar = Avatar::Initial { text: initial };

        Self {
            uuid,
            user_name,
            email,
            password,
            joined_channels: Vec::<String>::new(),
            avatar,
            created_at,
        }
    }

    pub async fn insert_into_db(&self, pool: &Database) -> Result<(), ServerError> {
        if pool
            .client
            .query("SELECT * FROM user_data WHERE email = $email")
            .bind(("email", &self.email))
            .await?
            .take::<Option<Self>>(0)
            .unwrap_or(None)
            .is_some()
        {
            return Err(ServerError::EmailTaken);
        }

        match pool
            .client
            .create::<Option<Self>>(("user_data", &self.uuid))
            .content(self)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(ServerError::DatabaseError(err)),
        }
    }

    pub async fn add_channel(&self, room_uuid: String, pool: &Database) -> Result<(), ServerError> {
        let find_entry = pool
            .client
            .select::<Option<Self>>(("user_data", &self.uuid))
            .await?;

        if let Some(user_data) = find_entry {
            if user_data.joined_channels.contains(&room_uuid) {
                return Err(ServerError::AddChannelError);
            }

            pool.client
                .update::<Option<Self>>(("user_data", &self.uuid))
                .patch(surrealdb::opt::PatchOp::add("/joined_channels", room_uuid))
                .await?;
        }
        Ok(())
    }

    pub async fn remove_channel(
        &self,
        channel: String,
        pool: &Database,
    ) -> Result<(), ServerError> {
        let find_entry = pool
            .client
            .select::<Option<Self>>(("user_data", &self.uuid))
            .await?;

        if let Some(mut user_data) = find_entry {
            if !user_data.joined_channels.contains(&channel) {
                return Err(ServerError::RemoveChannelError);
            }
            user_data
                .joined_channels
                .retain(|room_uuid| *room_uuid != channel);

            pool.client
                .update::<Option<Self>>(("user_data", &self.uuid))
                .merge(user_data)
                .await?;
        }

        Ok(())
    }

    pub async fn get_from_email(email: &str, pool: &Database) -> Result<Option<Self>, ServerError> {
        match pool
            .client
            .query("SELECT * FROM type::table($table) WHERE email = $email")
            .bind(("table", "user_data"))
            .bind(("email", email))
            .await
        {
            Ok(mut maybe_user) => maybe_user
                .take::<Option<Self>>(0)
                .map_err(|err| ServerError::DatabaseError(err)),
            Err(err) => Err(ServerError::DatabaseError(err)),
        }
    }

    pub async fn get_from_uuid(uuid: &str, pool: &Database) -> Option<Self> {
        match pool
            .client
            .query("SELECT * FROM type::table($table) WHERE uuid = $uuid")
            .bind(("table", "user_data"))
            .bind(("uuid", uuid))
            .await
        {
            Ok(mut maybe_user) => maybe_user.take::<Option<Self>>(0).unwrap_or(None),
            Err(_) => None,
        }
    }
}
