use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Availability {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub uuid: String,
    pub user_name: String,
    pub email: String,
    pub password: String,
    pub joined_channels: Vec<String>,
}

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
    pub use super::{Availability, UserData};
    use crate::db::Database;

    impl UserData {
        pub fn new(uuid: String, user_name: String, email: String, password: String) -> Self {
            Self {
                uuid,
                user_name,
                email,
                password,
                joined_channels: Vec::<String>::new(),
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
            channel: String,
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
                    .retain(|room_uuid| *room_uuid != channel);

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
