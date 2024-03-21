use crate::models::user_model::User;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

pub enum PubSubClient {
    All,
    Publisher,
    Subscriber,
}

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
    #[cfg(feature = "ssr")]
    #[error("Msg Send Error: {0}")]
    FromRedisError(#[from] fred::error::RedisError),
    #[error("Other: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub room_name: String,
    pub room_uuid: String,
    pub users: HashSet<User>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use super::{PubSubClient, Room, RoomsManagerError};
    use crate::{
        models::{message_model::Msg, user_model::User},
        state::db::ssr::Database,
    };
    use fred::{
        clients::{RedisClient, SubscriberClient},
        error::RedisError,
        interfaces::{ClientLike, EventInterface, PubsubInterface, RedisResult},
        types::Builder,
    };
    use leptos::logging;
    use std::collections::HashSet;
    use tokio::task::JoinHandle;
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub struct RoomsManager {
        pub publisher_client: RedisClient,
        pub subscriber_client: SubscriberClient,
    }

    impl Room {
        fn new(room_name: String) -> Self {
            let room_uuid = Uuid::new_v4().as_simple().to_string();
            let users = HashSet::<User>::new();

            Self {
                room_name,
                room_uuid,
                users,
            }
        }

        fn insert_user(&mut self, user: User) -> Result<(), RoomsManagerError> {
            if !self.users.insert(user) {
                return Err(RoomsManagerError::UserAlreadyInside);
            }

            Ok(())
        }

        fn _remove_user(&mut self, user: User) -> Result<(), RoomsManagerError> {
            if self.users.contains(&user) {
                self.users.retain(|k| *k != user);
                Ok(())
            } else {
                Err(RoomsManagerError::UserDoesNotExist)
            }
        }
    }

    impl RoomsManager {
        pub fn new() -> Result<Self, RoomsManagerError> {
            let publisher_client = RedisClient::default();
            let subscriber_client = Builder::default_centralized().build_subscriber_client()?;

            Ok(Self {
                publisher_client,
                subscriber_client,
            })
        }

        pub async fn init(&self, client: PubSubClient) -> Result<(), RoomsManagerError> {
            match client {
                PubSubClient::Publisher => {
                    self.publisher_client.init().await?;
                }
                PubSubClient::Subscriber => {
                    self.subscriber_client.init().await?;
                }
                PubSubClient::All => {
                    self.publisher_client.init().await?;
                    self.subscriber_client.init().await?;
                }
            }

            Ok(())
        }

        pub async fn quit(&self, client: PubSubClient) -> Result<(), RoomsManagerError> {
            match client {
                PubSubClient::Publisher => {
                    self.publisher_client.quit().await?;
                }
                PubSubClient::Subscriber => {
                    self.subscriber_client.quit().await?;
                }
                PubSubClient::All => {
                    self.publisher_client.quit().await?;
                    self.subscriber_client.quit().await?;
                }
            }

            Ok(())
        }

        pub async fn new_room(
            room_name: String,
            user: User,
            pool: &Database,
        ) -> Result<String, RoomsManagerError> {
            // let mut rooms = self.rooms.write().unwrap();
            let mut room = Room::new(room_name);

            room.insert_user(user)?;

            pool.client
                .create::<Option<Room>>(("room_data", &room.room_uuid))
                .content(room.clone())
                .await?;

            // rooms.insert(room.room_uuid.clone(), room.clone());

            Ok(room.room_uuid)
        }

        pub async fn join_room(
            room_uuid: &str,
            user: User,
            pool: &Database,
        ) -> Result<(), RoomsManagerError> {
            // let mut rooms = self.rooms.write().unwrap();
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

        pub async fn get_room_name(
            room_uuid: &str,
            pool: &Database,
        ) -> Result<String, RoomsManagerError> {
            // let rooms = self.rooms.read().unwrap();
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
            room_uuid: String,
            pool: &Database,
        ) -> Result<(), RoomsManagerError> {
            // let rooms = self.rooms.read().unwrap();
            let find_entry = pool
                .client
                .select::<Option<Room>>(("room_data", room_uuid))
                .await?;

            match find_entry {
                Some(_) => Ok(()),
                None => Err(RoomsManagerError::RoomDoesNotExist),
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
