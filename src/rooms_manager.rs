use crate::user_model::User;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub enum SelectClient {
    All,
    Publisher,
    Subscriber,
}

#[derive(Debug)]
pub enum RoomError {
    RoomDoesNotExist,
    RoomCreationFailed,
    UserAlreadyInside,
    UserDoesNotExist,
    MsgSendError(String),
    MsgRecvError(String),
    Other(String),
}

#[derive(Debug, Clone)]
pub struct Room {
    pub room_name: String,
    pub room_uuid: String,
    pub users: Arc<RwLock<HashMap<String, User>>>, // k: user's uuid
}

#[cfg(feature = "ssr")]
pub mod ssr {
    pub use super::{Room, RoomError, SelectClient};
    use crate::{message_model::Msg, user_model::User};
    use fred::{
        clients::{RedisClient, SubscriberClient},
        error::RedisError,
        interfaces::{ClientLike, EventInterface, PubsubInterface, RedisResult},
        types::Builder,
    };
    use leptos::logging;
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };
    use tokio::task::JoinHandle;
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub struct RoomsManager {
        pub publisher_client: RedisClient,
        pub subscriber_client: SubscriberClient,
        pub rooms: Arc<RwLock<HashMap<String, Room>>>, // k: room's uuid
    }

    impl Room {
        fn new(room_name: String) -> Self {
            let room_uuid = Uuid::new_v4().as_simple().to_string();
            let users = Arc::new(RwLock::new(HashMap::<String, User>::new()));

            Self {
                room_name,
                room_uuid,
                users,
            }
        }

        pub fn insert_user(&self, user: User) -> Result<(), RoomError> {
            let mut users = self.users.write().unwrap();

            if !users.contains_key(&user.uuid) {
                users.insert(user.uuid.clone(), user);
                Ok(())
            } else {
                Err(RoomError::UserAlreadyInside)
            }
        }

        pub fn remove_user(&self, user: User) -> Result<(), RoomError> {
            let mut users = self.users.write().unwrap();

            if users.contains_key(&user.uuid) {
                users.retain(|k, _| *k != user.uuid);
                Ok(())
            } else {
                Err(RoomError::UserDoesNotExist)
            }
        }
    }

    impl RoomsManager {
        pub fn new() -> Self {
            let publisher_client = RedisClient::default();
            let subscriber_client = Builder::default_centralized()
                .build_subscriber_client()
                .map_err(|err| logging::log!("{:?}", err))
                .unwrap();
            let rooms = Arc::new(RwLock::new(HashMap::<String, Room>::new()));

            Self {
                publisher_client,
                subscriber_client,
                rooms,
            }
        }

        pub async fn init(&self, which: SelectClient) -> Result<(), RedisError> {
            match which {
                SelectClient::Publisher => {
                    self.publisher_client.init().await?;
                }
                SelectClient::Subscriber => {
                    self.subscriber_client.init().await?;
                }
                SelectClient::All => {
                    self.publisher_client.init().await?;
                    self.subscriber_client.init().await?;
                }
            }

            Ok(())
        }

        pub async fn quit(&self, which: SelectClient) -> Result<(), RedisError> {
            match which {
                SelectClient::Publisher => {
                    self.publisher_client.quit().await?;
                }
                SelectClient::Subscriber => {
                    self.subscriber_client.quit().await?;
                }
                SelectClient::All => {
                    self.publisher_client.quit().await?;
                    self.subscriber_client.quit().await?;
                }
            }

            Ok(())
        }

        pub fn new_room(&self, room_name: String, user: User) -> Result<String, RoomError> {
            let mut rooms = self.rooms.write().unwrap();
            let room = Room::new(room_name);

            room.insert_user(user)?;

            rooms.insert(room.room_uuid.clone(), room.clone());

            Ok(room.room_uuid)
        }

        pub fn join_room(&self, room_uuid: String, user: User) -> Result<(), RoomError> {
            let rooms = self.rooms.write().unwrap();

            match rooms.get(&room_uuid) {
                Some(room) => room.insert_user(user),
                None => Err(RoomError::RoomDoesNotExist),
            }
        }

        pub fn validate_uuid(&self, room_uuid: String) -> Result<Option<String>, RoomError> {
            logging::log!("Validating path: {}", room_uuid);
            let rooms = self.rooms.read().unwrap();

            match rooms.get(&room_uuid) {
                Some(_) => Ok(Some(room_uuid)),
                None => Err(RoomError::RoomDoesNotExist),
            }
        }

        pub async fn publish_msg(&self, room_uuid: String, msg: Msg) -> Result<usize, RedisError> {
            match msg {
                Msg::Text(text) => self.publisher_client.publish(&room_uuid, text).await,
                Msg::Bytes(bytes) => self.publisher_client.publish(&room_uuid, bytes).await,
            }
        }

        pub async fn subscribe_msg(&self) -> JoinHandle<RedisResult<()>> {
            self.subscriber_client.on_message(|msg| {
                // TODO: create a function to send back the message
                logging::log!("{}: {:?}", msg.channel, msg.value);

                Ok(())
            })
        }
    }
}
