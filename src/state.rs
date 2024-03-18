pub mod auth;
pub mod db;
pub mod rooms_manager;

#[cfg(feature = "ssr")]
pub mod ssr {
    use super::{auth::ssr::AuthSession, db::ssr::Database, rooms_manager::ssr::RoomsManager};
    use axum::extract::FromRef;
    use leptos::{use_context, LeptosOptions, ServerFnError};
    use leptos_router::RouteListing;

    #[derive(Clone, FromRef, Debug)]
    pub struct AppState {
        pub pool: Database,
        pub rooms_manager: RoomsManager,
        pub leptos_options: LeptosOptions,
        pub routes: Vec<RouteListing>,
    }

    pub fn rooms_manager() -> Result<RoomsManager, ServerFnError> {
        use_context::<RoomsManager>()
            .ok_or_else(|| ServerFnError::new("RoomsManager does not exist"))
    }

    pub fn pool() -> Result<Database, ServerFnError> {
        use_context::<Database>().ok_or_else(|| ServerFnError::new("No database is detected!"))
    }

    pub fn auth() -> Result<AuthSession, ServerFnError> {
        use_context::<AuthSession>()
            .ok_or_else(|| ServerFnError::new("No AuthSession is detected!"))
    }
}
