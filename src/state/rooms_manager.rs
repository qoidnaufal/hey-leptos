use {
    crate::models::user_model::User,
    chrono::{DateTime, Utc},
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    thiserror::Error,
};

#[derive(Debug, Error)]
pub enum RoomsManagerError {
    #[error("Room Does Not Exist")]
    RoomDoesNotExist,
    #[cfg(feature = "ssr")]
    #[error("Room Creation Failed")]
    FromDbError(#[from] surrealdb::Error),
    #[error("User Already Inside")]
    UserAlreadyInside,
    #[error("User Does Not Exist")]
    UserDoesNotExist,
    #[error("Other: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub room_name: String,
    pub room_uuid: String,
    pub users: HashMap<String, User>,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "ssr")]
use {
    crate::{models::message_model::WsPayload, state::db::Database},
    std::sync::{Arc, RwLock},
    tokio::sync::mpsc,
    uuid::Uuid,
};

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Default)]
pub struct RoomsManager {
    pub users: Arc<RwLock<HashMap<String, Option<mpsc::UnboundedSender<WsPayload>>>>>,
}

#[cfg(feature = "ssr")]
impl Room {
    fn new(room_name: String, created_at: DateTime<Utc>) -> Self {
        let room_uuid = Uuid::new_v4().as_simple().to_string();
        let users = HashMap::<String, User>::new();

        Self {
            room_name,
            room_uuid,
            users,
            created_at,
        }
    }

    fn insert_user(&mut self, user: User) -> Result<(), RoomsManagerError> {
        if !self.users.contains_key(&user.uuid) {
            self.users.insert(user.uuid.clone(), user);
            Ok(())
        } else {
            Err(RoomsManagerError::UserAlreadyInside)
        }
    }

    fn remove_user(&mut self, user: User) -> Result<(), RoomsManagerError> {
        if self.users.contains_key(&user.uuid) {
            self.users.retain(|k, _| *k != user.uuid);
            Ok(())
        } else {
            Err(RoomsManagerError::UserDoesNotExist)
        }
    }
}

#[cfg(feature = "ssr")]
impl RoomsManager {
    pub async fn new_room(
        room_name: String,
        user: User,
        pool: &Database,
        created_at: DateTime<Utc>,
    ) -> Result<String, RoomsManagerError> {
        let mut room = Room::new(room_name, created_at);

        room.insert_user(user)?;

        pool.client
            .create::<Option<Room>>(("room_data", &room.room_uuid))
            .content(room.clone())
            .await?;

        Ok(room.room_uuid)
    }

    pub async fn join_room(
        room_uuid: &str,
        user: User,
        pool: &Database,
    ) -> Result<(), RoomsManagerError> {
        let find_entry = pool
            .client
            .select::<Option<Room>>(("room_data", room_uuid))
            .await?;

        if let Some(mut room) = find_entry {
            room.insert_user(user)?;

            pool.client
                .update::<Option<Room>>(("room_data", &room.room_uuid))
                .merge(room)
                .await?;
        }

        Ok(())
    }

    pub async fn leave_room(
        room_uuid: &str,
        user: User,
        pool: &Database,
    ) -> Result<(), RoomsManagerError> {
        let find_entry = pool
            .client
            .select::<Option<Room>>(("room_data", room_uuid))
            .await?;

        if let Some(mut room) = find_entry {
            room.remove_user(user)?;

            pool.client
                .update::<Option<Room>>(("room_data", &room.room_uuid))
                .merge(room)
                .await?;
        }

        Ok(())
    }

    pub async fn get_room_name(
        room_uuid: &str,
        pool: &Database,
    ) -> Result<String, RoomsManagerError> {
        let find_entry = pool
            .client
            .select::<Option<Room>>(("room_data", room_uuid))
            .await?;

        match find_entry {
            Some(room) => Ok(room.room_name),
            None => Err(RoomsManagerError::RoomDoesNotExist),
        }
    }

    pub async fn validate_uuid(
        room_uuid: &str,
        pool: &Database,
    ) -> Result<Room, RoomsManagerError> {
        let find_entry = pool
            .client
            .select::<Option<Room>>(("room_data", room_uuid))
            .await?;

        match find_entry {
            Some(room) => Ok(room),
            None => Err(RoomsManagerError::RoomDoesNotExist),
        }
    }
}
