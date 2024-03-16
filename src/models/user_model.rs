use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Availability {
    Available,
    Unavailable,
}

// ---- avatar

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub joined_channels: Vec<(String, String)>,
    pub avatar: Avatar,
}

// ---- user to expose to the client

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct User {
    pub uuid: String,
    pub user_name: String,
}

impl User {
    pub fn from_user_data(user_data: &UserData) -> Self {
        Self {
            uuid: user_data.uuid.clone(),
            user_name: user_data.user_name.clone(),
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    pub use super::{Availability, Avatar, UserData};
    use crate::db::Database;

    impl UserData {
        pub fn new(uuid: String, user_name: String, email: String, password: String) -> Self {
            let mut initial = user_name.clone();
            initial.truncate(1);
            let avatar = Avatar::Initial { text: initial };

            Self {
                uuid,
                user_name,
                email,
                password,
                joined_channels: Vec::<(String, String)>::new(),
                avatar,
            }
        }

        pub async fn insert_into_db(&self, pool: &Database) -> Result<(), surrealdb::Error> {
            match pool
                .client
                .create::<Option<Self>>(("user_data", self.uuid.clone()))
                .content(self)
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }

        pub async fn add_channel(
            &self,
            channel: (String, String),
            pool: &Database,
        ) -> Result<(), surrealdb::Error> {
            let find_entry = pool
                .client
                .select::<Option<Self>>(("user_data", self.uuid.clone()))
                .await?;

            if let Some(mut user_data) = find_entry {
                user_data.joined_channels.push(channel);

                pool.client
                    .update::<Option<Self>>(("user_data", self.uuid.clone()))
                    .merge(user_data)
                    .await?;
            }

            Ok(())
        }

        pub async fn remove_channel(
            &self,
            channel: String,
            pool: &Database,
        ) -> Result<(), surrealdb::Error> {
            let find_entry = pool
                .client
                .select::<Option<Self>>(("user_data", self.uuid.clone()))
                .await?;

            if let Some(mut user_data) = find_entry {
                user_data
                    .joined_channels
                    .retain(|(room_uuid, _)| *room_uuid != channel);

                pool.client
                    .update::<Option<Self>>(("user_data", self.uuid.clone()))
                    .merge(user_data)
                    .await?;
            }

            Ok(())
        }

        pub async fn check_availability_by_email(email: &str, pool: &Database) -> Availability {
            match pool
                .client
                .query("SELECT * FROM type::table($table) WHERE email = $email")
                .bind(("table", "user_data"))
                .bind(("email", email))
                .await
            {
                Ok(mut maybe_user) => match maybe_user.take::<Option<Self>>(0) {
                    Ok(None) => Availability::Available,
                    _ => Availability::Unavailable,
                },
                _ => Availability::Unavailable,
            }
        }

        pub async fn get_from_email(
            email: &str,
            pool: &Database,
        ) -> Result<Option<Self>, surrealdb::Error> {
            match pool
                .client
                .query("SELECT * FROM type::table($table) WHERE email = $email")
                .bind(("table", "user_data"))
                .bind(("email", email))
                .await
            {
                Ok(mut maybe_user) => maybe_user.take::<Option<Self>>(0),
                Err(err) => Err(err),
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
}
