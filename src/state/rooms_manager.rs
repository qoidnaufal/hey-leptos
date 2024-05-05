use {
    crate::{error::ServerError, models::user_model::User},
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

    pub fn insert_user(&mut self, user: User) -> Result<(), ServerError> {
        if !self.users.contains_key(&user.uuid) {
            self.users.insert(user.uuid.clone(), user);
            Ok(())
        } else {
            Err(ServerError::AddChannelError)
        }
    }

    pub fn remove_user(&mut self, user: User) -> Result<(), ServerError> {
        if self.users.contains_key(&user.uuid) {
            self.users.retain(|k, _| *k != user.uuid);
            Ok(())
        } else {
            Err(ServerError::UserDoesNotExist)
        }
    }
}

#[cfg(feature = "ssr")]
use {
    crate::{models::message_model::WsPayload, state::db::Database},
    std::sync::{Arc, RwLock},
    tokio::sync::{broadcast, mpsc},
};

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Default)]
pub struct ChatRoom {
    pub uuid: String,
    pub name: String,
    pub users: Arc<RwLock<HashMap<String, Option<mpsc::UnboundedSender<WsPayload>>>>>,
}

#[cfg(feature = "ssr")]
impl ChatRoom {
    pub fn from_room_data(room_data: &RoomData) -> Self {
        let uuid = room_data.room_uuid.clone();
        let name = room_data.room_name.clone();
        let users = Arc::new(RwLock::new(HashMap::<
            String,
            Option<mpsc::UnboundedSender<WsPayload>>,
        >::new()));
        Self { uuid, name, users }
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
pub struct RoomsManager {
    pub chatrooms: Arc<RwLock<HashMap<String, ChatRoom>>>,
    pub ipc_sender: broadcast::Sender<ChatRoom>,
}

#[cfg(feature = "ssr")]
impl RoomsManager {
    pub fn init() -> Self {
        let chatrooms = Arc::new(RwLock::new(HashMap::<String, ChatRoom>::new()));
        let (ipc_sender, _) = broadcast::channel(1024);
        Self {
            chatrooms,
            ipc_sender,
        }
    }

    pub async fn new_room(
        &self,
        room_name: String,
        user: User,
        pool: &Database,
        created_at: DateTime<Utc>,
    ) -> Result<String, ServerError> {
        let mut room_data = RoomData::new(room_name, created_at);
        room_data.insert_user(user)?;
        let room_uuid = room_data.room_uuid.clone();
        {
            let chatroom = ChatRoom::from_room_data(&room_data);
            self.ipc_sender
                .send(chatroom)
                .map_err(|_| ServerError::IPCFailed)?;
        }
        pool.client
            .create::<Option<RoomData>>(("room_data", &room_data.room_uuid))
            .content(room_data)
            .await?;
        Ok(room_uuid)
    }

    pub async fn join_room(
        &self,
        room_uuid: &str,
        user: User,
        pool: &Database,
    ) -> Result<(), ServerError> {
        let find_entry = pool
            .client
            .select::<Option<RoomData>>(("room_data", room_uuid))
            .await?;
        if let Some(mut room_data) = find_entry {
            room_data.insert_user(user)?;
            pool.client
                .update::<Option<RoomData>>(("room_data", &room_data.room_uuid))
                .merge(room_data.clone())
                .await?;
            {
                let chatrooms = self.chatrooms.read().unwrap();
                let chatroom = chatrooms.get(&room_data.room_uuid).unwrap();
                self.ipc_sender
                    .send(chatroom.clone())
                    .map_err(|_| ServerError::IPCFailed)?;
            }
        }
        Ok(())
    }

    pub async fn leave_room(
        &self,
        room_uuid: &str,
        user: User,
        pool: &Database,
    ) -> Result<(), ServerError> {
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
    ) -> Result<String, ServerError> {
        let find_entry = pool
            .client
            .select::<Option<RoomData>>(("room_data", room_uuid))
            .await?;
        match find_entry {
            Some(room_data) => Ok(room_data.room_name),
            None => Err(ServerError::RoomDoesNotExist),
        }
    }

    pub async fn validate_uuid(
        &self,
        room_uuid: &str,
        pool: &Database,
    ) -> Result<RoomData, ServerError> {
        let find_entry = pool
            .client
            .select::<Option<RoomData>>(("room_data", room_uuid))
            .await?;
        match find_entry {
            Some(room_data) => Ok(room_data),
            None => Err(ServerError::RoomDoesNotExist),
        }
    }
}
