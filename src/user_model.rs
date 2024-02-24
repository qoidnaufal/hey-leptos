use crate::db::Database;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub uuid: String,
    pub user_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredUser {
    pub uuid: String,
    pub user_name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Availability {
    Available,
    Unavailable,
}

impl UserData {
    pub fn new(uuid: String, user_name: String, email: String, password: String) -> Self {
        Self {
            uuid,
            user_name,
            email,
            password,
        }
    }

    pub async fn insert_into_db(&self, pool: Database) -> Result<(), surrealdb::Error> {
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

    pub async fn get_from_id(uuid: &str, pool: &Database) -> Option<Self> {
        match pool
            .client
            .query("SELECt * FROM type::table($table) WHERE uuid = $uuid")
            .bind(("table", "user_data"))
            .bind(("uuid", uuid))
            .await
        {
            Ok(mut maybe_user) => maybe_user
                .take::<Option<Self>>(0)
                .expect("Unable to extract the user"),
            Err(_) => None,
        }
    }
}
