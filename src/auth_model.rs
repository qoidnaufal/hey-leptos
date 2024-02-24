pub use crate::{db::Database, user_model::UserData};
pub use async_trait::async_trait;
pub use axum_session_auth::{Authentication, SessionSurrealPool};
pub use leptos::{use_context, ServerFnError};
pub use surrealdb::engine::remote::ws::Client;

pub type AuthSession =
    axum_session_auth::AuthSession<UserData, String, SessionSurrealPool<Client>, Database>;

pub fn pool() -> Result<Database, ServerFnError> {
    use_context::<Database>().ok_or_else(|| ServerFnError::new("No database is detected!"))
}

pub fn auth() -> Result<AuthSession, ServerFnError> {
    use_context::<AuthSession>().ok_or_else(|| ServerFnError::new("No AuthSession is detected!"))
}

#[async_trait]
impl Authentication<UserData, String, Database> for UserData {
    async fn load_user(userid: String, pool: Option<&Database>) -> Result<UserData, anyhow::Error> {
        let pool = pool.expect("Pool doesn't exist!");

        UserData::get_from_id(&userid, &pool)
            .await
            .ok_or_else(|| anyhow::anyhow!("Can't get the user!"))
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
