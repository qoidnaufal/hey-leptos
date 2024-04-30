#[cfg(feature = "ssr")]
use {
    crate::{
        models::{message_model::WsPayload, user_model::UserData},
        state::{auth::AuthSession, rooms_manager::RoomsManager, AppState},
    },
    axum::{
        extract::{
            ws::{Message, WebSocket, WebSocketUpgrade},
            State,
        },
        http::StatusCode,
        response::IntoResponse,
    },
    futures::{SinkExt, StreamExt},
};

#[cfg(feature = "ssr")]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    auth_session: AuthSession,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let pool = app_state.pool;
    let rooms_manager = app_state.rooms_manager;
    let user = auth_session.current_user.ok_or(StatusCode::UNAUTHORIZED)?;
    let user_data = UserData::get_from_uuid(&user.uuid, &pool)
        .await
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(ws.on_upgrade(|socket| handle_connection(socket, user_data, rooms_manager)))
}

#[cfg(feature = "ssr")]
async fn handle_connection(ws: WebSocket, user_data: UserData, rooms_manager: RoomsManager) {
    let (mut server, mut from_client) = ws.split();
    {
        let mut stream_map = tokio_stream::StreamMap::new();
        rooms_manager
            .rooms
            .read()
            .unwrap()
            .iter()
            .for_each(|(_, chatroom)| {
                let mut rx = chatroom.subscribe();
                let rx = Box::pin(async_stream::stream! {
                    while let Ok(msg) = rx.recv().await { yield msg; }
                })
                    as std::pin::Pin<Box<dyn futures::Stream<Item = Message> + Send>>;
                stream_map.insert(chatroom.uuid.clone(), rx);
            });
        tokio::spawn(async move {
            while let Some((_, msg)) = stream_map.next().await {
                server
                    .send(msg)
                    .await
                    .map_err(|err| leptos::logging::log!("ERROR: {:?}", err))?;
            }
            server
                .close()
                .await
                .map_err(|err| leptos::logging::log!("ERROR: {:?}", err))?;
            Ok::<(), ()>(())
        });
    }
    while let Some(Ok(ws_message)) = from_client.next().await {
        let channel_payload = modify_msg(ws_message);
        broadcast_msg(channel_payload, &user_data, &rooms_manager).await;
    }
}

#[cfg(feature = "ssr")]
fn modify_msg(ws_message: Message) -> Message {
    let recv_payload = match ws_message {
        Message::Text(text) => {
            serde_json::from_str::<WsPayload>(&text).unwrap_or_else(|_| WsPayload::default())
        }
        _ => WsPayload::default(),
    };
    let modified_response = match recv_payload.op_code {
        1 => WsPayload::new(11, recv_payload.message.clone()),
        _ => WsPayload::default(),
    };
    let send_payload = serde_json::to_vec(&modified_response).unwrap();
    Message::Binary(send_payload)
}

#[cfg(feature = "ssr")]
async fn broadcast_msg(
    channel_payload: Message,
    user_data: &UserData,
    rooms_manager: &RoomsManager,
) {
    user_data.joined_channels.iter().for_each(|room_uuid| {
        let rooms = rooms_manager.rooms.read().unwrap();
        let chatroom = rooms
            .get(room_uuid)
            .expect("Need to provide valid room uuid");
        if let Some(tx) = &chatroom.sender {
            tx.send(channel_payload.clone()).unwrap();
        }
    })
}
