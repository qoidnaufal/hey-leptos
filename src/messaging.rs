#[cfg(feature = "ssr")]
use {
    crate::{
        models::{
            message_model::WsPayload,
            user_model::{User, UserData},
        },
        state::{
            auth::AuthSession,
            db::Database,
            rooms_manager::{ChatRoom, RoomsManager},
            AppState,
        },
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
    tokio::sync::mpsc,
};

#[cfg(feature = "ssr")]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    auth_session: AuthSession,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    // when we started the connection,
    // we need to sync with db whether we have Some created room(s) or None
    let rooms_manager = app_state.rooms_manager;
    let pool = app_state.pool;
    let user = auth_session.current_user.ok_or(StatusCode::UNAUTHORIZED)?;
    let user_data = UserData::get_from_uuid(&user.uuid, &pool)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(ws.on_upgrade(|socket| handle_connection(socket, user, user_data, rooms_manager, pool)))
}

#[cfg(feature = "ssr")]
async fn handle_connection(
    ws: WebSocket,
    user: User,
    user_data: UserData,
    rooms_manager: RoomsManager,
    pool: Database,
) {
    let (mut ws_sender, mut ws_reader) = ws.split();
    let (tx1, mut rx) = mpsc::unbounded_channel::<WsPayload>();
    let tx2 = tx1.clone();
    let mut ipc_receiver = rooms_manager.ipc_sender.subscribe();

    let chatrooms = rooms_manager.chatrooms.clone();
    let user_uuid = user.uuid.clone();
    let user_id = user_uuid.clone();
    let rm = rooms_manager.clone();

    // --- Ensure on server restart, the rooms manager is synced with database
    tokio::spawn(async move {
        if !user_data.joined_channels.is_empty() {
            let iter = user_data.joined_channels.iter().map(|room_uuid| async {
                let room_data = rm.validate_uuid(room_uuid, &pool).await.unwrap();
                let chatroom = ChatRoom::from_room_data(&room_data);
                {
                    let mut users = chatroom.users.write().unwrap();
                    users.insert(user_id.clone(), Some(tx2.clone()));
                }
                let mut chatrooms = rm.chatrooms.write().unwrap();
                chatrooms.insert(chatroom.uuid.clone(), chatroom);
            });
            futures::future::join_all(iter).await;
        }
    });

    // --- Receive notification when user create or join to a new channel
    tokio::spawn(async move {
        while let Ok(chatroom) = ipc_receiver.recv().await {
            let mut users = chatroom.users.write().unwrap();
            users.insert(user_uuid.clone(), Some(tx1.clone()));
            let mut chatrooms = chatrooms.write().unwrap();
            chatrooms.insert(chatroom.uuid.clone(), chatroom.clone());
        }
    });

    // --- Send back the message to the client
    tokio::spawn(async move {
        while let Some(channel_payload) = rx.recv().await {
            let msg = serde_json::to_vec(&channel_payload).unwrap();
            let msg = Message::Binary(msg);
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
        let _ = ws_sender.close();
    });

    // --- Receive message from the client
    while let Some(Ok(ws_message)) = ws_reader.next().await {
        let channel_payload = modify_msg(ws_message);
        broadcast_msg(channel_payload, &rooms_manager).await;
    }
}

#[cfg(feature = "ssr")]
fn modify_msg(ws_message: Message) -> WsPayload {
    // on page refresh, the browser will send ws Close message
    // so i need to handle this to avoid error
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
    modified_response
}

#[cfg(feature = "ssr")]
async fn broadcast_msg(channel_payload: WsPayload, rooms_manager: &RoomsManager) {
    let chatrooms = rooms_manager.chatrooms.read().unwrap();
    if let Some(chatroom) = chatrooms.get(&channel_payload.message) {
        let users = chatroom.users.read().unwrap();
        for (_, channel) in users.iter() {
            if let Some(tx1) = channel {
                let _ = tx1.send(channel_payload.clone());
            }
        }
    }
}
