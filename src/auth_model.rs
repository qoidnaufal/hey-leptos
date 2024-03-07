pub use crate::{
    db::Database,
    user_model::{User, UserData},
};
pub use async_trait::async_trait;
pub use axum_session_auth::{Authentication, SessionSurrealPool};
pub use leptos::{use_context, ServerFnError};
pub use surrealdb::engine::remote::ws::Client;

pub type AuthSession =
    axum_session_auth::AuthSession<User, String, SessionSurrealPool<Client>, Database>;

#[async_trait]
impl Authentication<User, String, Database> for User {
    async fn load_user(userid: String, pool: Option<&Database>) -> Result<User, anyhow::Error> {
        let pool = pool.expect("Pool doesn't exist!");

        let user_data = UserData::get_from_uuid(&userid, &pool)
            .await
            .ok_or_else(|| anyhow::anyhow!("User does not exist"))?;

        Ok(User::from_user_data(&user_data))
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
