use {
    crate::{error::ApiError, models::user_model::User},
    chrono::{DateTime, Utc},
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    uuid::Uuid,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub room_name: String,
    pub room_uuid: String,
    pub users: HashMap<String, User>,
    pub created_at: DateTime<Utc>,
}

impl Room {
    pub fn new(room_name: String, created_at: DateTime<Utc>) -> Self {
        let room_uuid = Uuid::new_v4().as_simple().to_string();
        let users = HashMap::<String, User>::new();
        Self {
            room_name,
            room_uuid,
            users,
            created_at,
        }
    }

    pub fn insert_user(&mut self, user: User) -> Result<(), ApiError> {
        if !self.users.contains_key(&user.uuid) {
            self.users.insert(user.uuid.clone(), user);
            Ok(())
        } else {
            Err(ApiError::AddChannelError)
        }
    }

    pub fn remove_user(&mut self, user: User) -> Result<(), ApiError> {
        if self.users.contains_key(&user.uuid) {
            self.users.retain(|k, _| *k != user.uuid);
            Ok(())
        } else {
            Err(ApiError::UserDoesNotExist)
        }
    }
}

#[cfg(feature = "ssr")]
use {
    crate::{models::message_model::WsPayload, state::db::Database},
    std::sync::{Arc, RwLock},
    tokio::sync::broadcast,
};

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Default)]
pub struct ChatRoom {
    pub uuid: String,
    pub name: String,
    pub channel: Option<broadcast::Sender<WsPayload>>,
}

#[cfg(feature = "ssr")]
impl ChatRoom {
    pub fn from_room_data(room: &Room) -> Self {
        let uuid = room.room_uuid.clone();
        let name = room.room_name.clone();
        let (tx, _) = broadcast::channel::<WsPayload>(100);
        let channel = Some(tx);
        Self {
            uuid,
            name,
            channel,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WsPayload> {
        self.channel
            .as_ref()
            .expect("tx must be assigned")
            .subscribe()
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Default)]
pub struct RoomsManager {
    pub channels: Arc<RwLock<HashMap<String, ChatRoom>>>,
}

#[cfg(feature = "ssr")]
impl RoomsManager {
    pub async fn new_room(
        &self,
        room_name: String,
        user: User,
        pool: &Database,
        created_at: DateTime<Utc>,
    ) -> Result<String, ApiError> {
        let mut room = Room::new(room_name, created_at);
        room.insert_user(user)?;
        pool.client
            .create::<Option<Room>>(("room_data", &room.room_uuid))
            .content(room.clone())
            .await?;
        {
            let chat_room = ChatRoom::from_room_data(&room);
            let mut channels = self.channels.write().unwrap();
            if !channels.contains_key(&chat_room.uuid) {
                channels.insert(chat_room.uuid.clone(), chat_room);
            }
        }
        Ok(room.room_uuid)
    }

    pub async fn join_room(
        &self,
        room_uuid: &str,
        user: User,
        pool: &Database,
    ) -> Result<(), ApiError> {
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
        &self,
        room_uuid: &str,
        user: User,
        pool: &Database,
    ) -> Result<(), ApiError> {
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
        &self,
        room_uuid: &str,
        pool: &Database,
    ) -> Result<String, ApiError> {
        let find_entry = pool
            .client
            .select::<Option<Room>>(("room_data", room_uuid))
            .await?;
        match find_entry {
            Some(room) => Ok(room.room_name),
            None => Err(ApiError::RoomDoesNotExist),
        }
    }

    pub async fn validate_uuid(&self, room_uuid: &str, pool: &Database) -> Result<Room, ApiError> {
        let find_entry = pool
            .client
            .select::<Option<Room>>(("room_data", room_uuid))
            .await?;
        match find_entry {
            Some(room) => Ok(room),
            None => Err(ApiError::RoomDoesNotExist),
        }
    }
}
