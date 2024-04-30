use {
    crate::{error::ApiError, models::user_model::User},
    chrono::{DateTime, Utc},
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    uuid::Uuid,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomData {
    pub room_name: String,
    pub room_uuid: String,
    pub users: HashMap<String, User>,
    pub created_at: DateTime<Utc>,
}

impl RoomData {
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
    crate::state::db::Database,
    axum::extract::ws::Message,
    std::sync::{Arc, RwLock},
    tokio::sync::broadcast,
};

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Default)]
pub struct ChatRoom {
    pub uuid: String,
    pub name: String,
    pub sender: Option<broadcast::Sender<Message>>,
}

#[cfg(feature = "ssr")]
impl ChatRoom {
    pub fn from_room_data(room_data: &RoomData) -> Self {
        let uuid = room_data.room_uuid.clone();
        let name = room_data.room_name.clone();
        let (tx, _) = broadcast::channel::<Message>(100);
        let sender = Some(tx);
        Self { uuid, name, sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.sender
            .as_ref()
            .expect("tx must be assigned")
            .subscribe()
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Default)]
pub struct RoomsManager {
    pub rooms: Arc<RwLock<HashMap<String, ChatRoom>>>,
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
        let mut room_data = RoomData::new(room_name, created_at);
        room_data.insert_user(user)?;
        pool.client
            .create::<Option<RoomData>>(("room_data", &room_data.room_uuid))
            .content(room_data.clone())
            .await?;
        {
            let chatroom = ChatRoom::from_room_data(&room_data);
            let mut rooms = self.rooms.write().unwrap();
            if !rooms.contains_key(&chatroom.uuid) {
                rooms.insert(chatroom.uuid.clone(), chatroom);
            }
        }
        Ok(room_data.room_uuid)
    }

    pub async fn join_room(
        &self,
        room_uuid: &str,
        user: User,
        pool: &Database,
    ) -> Result<(), ApiError> {
        let find_entry = pool
            .client
            .select::<Option<RoomData>>(("room_data", room_uuid))
            .await?;
        if let Some(mut room_data) = find_entry {
            room_data.insert_user(user)?;
            pool.client
                .update::<Option<RoomData>>(("room_data", &room_data.room_uuid))
                .merge(room_data)
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
            .select::<Option<RoomData>>(("room_data", room_uuid))
            .await?;
        if let Some(mut room_data) = find_entry {
            room_data.remove_user(user)?;
            pool.client
                .update::<Option<RoomData>>(("room_data", &room_data.room_uuid))
                .merge(room_data)
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
            .select::<Option<RoomData>>(("room_data", room_uuid))
            .await?;
        match find_entry {
            Some(room_data) => Ok(room_data.room_name),
            None => Err(ApiError::RoomDoesNotExist),
        }
    }

    pub async fn validate_uuid(
        &self,
        room_uuid: &str,
        pool: &Database,
    ) -> Result<RoomData, ApiError> {
        let find_entry = pool
            .client
            .select::<Option<RoomData>>(("room_data", room_uuid))
            .await?;
        match find_entry {
            Some(room_data) => Ok(room_data),
            None => Err(ApiError::RoomDoesNotExist),
        }
    }
}
