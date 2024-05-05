#[cfg(feature = "ssr")]
use {
    crate::{
        models::{message_model::WsPayload, user_model::User},
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
    tokio::sync::mpsc,
};

#[cfg(feature = "ssr")]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    auth_session: AuthSession,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let rooms_manager = app_state.rooms_manager;
    let user = auth_session
        .current_user
        .expect("AuthSession should contains user information");
    Ok(ws.on_upgrade(|socket| handle_connection(socket, user, rooms_manager)))
}

#[cfg(feature = "ssr")]
async fn handle_connection(ws: WebSocket, user: User, rooms_manager: RoomsManager) {
    let (mut ws_sender, mut ws_reader) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsPayload>();
    let mut ipc_receiver = rooms_manager.ipc_sender.subscribe();

    let chatrooms = rooms_manager.chatrooms.clone();
    let user_uuid = user.uuid.clone();

    // --- Receive notification when user create or join to a new channel
    tokio::spawn(async move {
        while let Ok(chatroom) = ipc_receiver.recv().await {
            let mut users = chatroom.users.write().unwrap();
            users.insert(user_uuid.clone(), Some(tx.clone()));
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
        // let _ = ws_sender.close();
    });

    // --- Receive message from the client
    while let Some(Ok(ws_message)) = ws_reader.next().await {
        let channel_payload = modify_msg(ws_message);
        broadcast_msg(channel_payload, &rooms_manager).await;
    }
}

#[cfg(feature = "ssr")]
fn modify_msg(ws_message: Message) -> WsPayload {
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
    let chatroom = chatrooms.get(&channel_payload.message).unwrap(); // unwrap here is dangerous
    let users = chatroom.users.read().unwrap();
    for (_, channel) in users.iter() {
        if let Some(tx) = channel {
            let _ = tx.send(channel_payload.clone());
        }
    }
}
