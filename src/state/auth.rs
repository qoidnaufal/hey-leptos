#[cfg(feature = "ssr")]
use {
    crate::models::user_model::{User, UserData},
    crate::state::db::Database,
    async_trait::async_trait,
    axum_session_auth::{Authentication, SessionSurrealPool},
    surrealdb::engine::remote::ws::Client,
};

#[cfg(feature = "ssr")]
pub type AuthSession =
    axum_session_auth::AuthSession<User, String, SessionSurrealPool<Client>, Database>;

#[cfg(feature = "ssr")]
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
