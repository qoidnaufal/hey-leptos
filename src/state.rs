use crate::db;
use axum::extract::FromRef;
use leptos::LeptosOptions;
use leptos_router::RouteListing;

#[derive(Clone, FromRef, Debug)]
pub struct AppState {
    pub db: db::ssr::Database,
    pub leptos_options: LeptosOptions,
    pub routes: Vec<RouteListing>,
}
