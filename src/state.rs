pub mod auth;
pub mod db;
pub mod rooms_manager;

#[cfg(feature = "ssr")]
use {
    self::rooms_manager::RoomsManager,
    auth::AuthSession,
    axum::extract::FromRef,
    db::Database,
    leptos::{use_context, LeptosOptions, ServerFnError},
    leptos_router::RouteListing,
};

#[cfg(feature = "ssr")]
#[derive(Clone, FromRef, Debug)]
pub struct AppState {
    pub pool: Database,
    pub leptos_options: LeptosOptions,
    pub routes: Vec<RouteListing>,
    pub rooms_manager: RoomsManager,
}

#[cfg(feature = "ssr")]
pub fn pool() -> Result<Database, ServerFnError> {
    use_context::<Database>().ok_or_else(|| ServerFnError::new("No database is detected!"))
}

#[cfg(feature = "ssr")]
pub fn auth() -> Result<AuthSession, ServerFnError> {
    use_context::<AuthSession>().ok_or_else(|| ServerFnError::new("No AuthSession is detected!"))
}
