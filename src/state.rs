use crate::db::Database;
use axum::extract::{ws::Message, FromRef};
use leptos::LeptosOptions;
use leptos_router::RouteListing;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, FromRef, Debug)]
pub struct AppState {
    pub pool: Database,
    pub room: Room,
    pub leptos_options: LeptosOptions,
    pub routes: Vec<RouteListing>,
}

// key is email
pub type Room = Arc<RwLock<HashMap<String, UserConn>>>;

#[derive(Clone, Default, Debug)]
pub struct UserConn {
    pub user_name: String,
    pub uuid: String,
    pub status: ConStatus,
    pub sender: Option<UnboundedSender<Message>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum ConStatus {
    Connected,
    #[default]
    Disconnected,
}
