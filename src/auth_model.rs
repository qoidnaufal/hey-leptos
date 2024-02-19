use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub uuid: String,
    pub user_name: String,
    pub email: String,
    pub password: String,
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
}

#[cfg(feature = "ssr")]
pub mod ssr {
    pub use super::UserData;
    pub use crate::db::ssr::Database;
    pub use async_trait::async_trait;
    pub use axum_session_auth::{Authentication, SessionSurrealPool};
    pub use leptos::{use_context, ServerFnError};
    pub use surrealdb::engine::remote::ws::Client;

    pub type AuthSession =
        axum_session_auth::AuthSession<UserData, String, SessionSurrealPool<Client>, Database>;

    pub fn db() -> Result<Database, ServerFnError> {
        use_context::<Database>().ok_or_else(|| ServerFnError::new("No database is detected!"))
    }

    pub fn auth() -> Result<AuthSession, ServerFnError> {
        use_context::<AuthSession>()
            .ok_or_else(|| ServerFnError::new("No AuthSession is detected!"))
    }

    #[async_trait]
    impl Authentication<UserData, String, Database> for UserData {
        async fn load_user(
            userid: String,
            pool: Option<&Database>,
        ) -> Result<UserData, anyhow::Error> {
            let db = pool.unwrap();

            db.get_user_by_id(&userid)
                .await
                .ok_or_else(|| anyhow::anyhow!("Cannot get user"))
        }

        fn is_authenticated(&self) -> bool {
            true
        }

        fn is_active(&self) -> bool {
            true
        }

        fn is_anonymous(&self) -> bool {
            false
        }
    }
}
