#[cfg(feature = "ssr")]
use {
    crate::{
        models::{message_model::WsPayload, user_model::User},
        state::{auth::AuthSession, rooms_manager::RoomsManager, AppState},
    },
    axum::{
        extract::{
            ws::{Message, WebSocket, WebSocketUpgrade},
            Path, State,
        },
        http::StatusCode,
        response::IntoResponse,
    },
    futures::{SinkExt, StreamExt},
    tokio::sync::mpsc,
};

#[cfg(feature = "ssr")]
pub async fn ws_handler(
    Path(room_uuid): Path<String>,
    ws: WebSocketUpgrade,
    auth_session: AuthSession,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let pool = app_state.pool;
    let rooms_manager = app_state.rooms_manager;
    let current_user = auth_session
        .current_user
        .ok_or_else(|| StatusCode::UNAUTHORIZED)?;

    match RoomsManager::validate_uuid(&room_uuid, &pool).await {
        Ok(_) => {
            Ok(ws.on_upgrade(|socket| handle_ws_connection(socket, rooms_manager, current_user)))
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

#[cfg(feature = "ssr")]
async fn handle_ws_connection(ws: WebSocket, rooms_manager: RoomsManager, current_user: User) {
    let (mut sink, mut stream) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsPayload>();

    tokio::spawn(async move {
        while let Some(msg_data) = rx.recv().await {
            let json_data = serde_json::to_vec(&msg_data).unwrap();
            let response = Message::Binary(json_data);
            if sink.send(response).await.is_err() {
                break;
            }
        }
        sink.close().await.unwrap_or(())
    });

    if let Ok(mut channel) = rooms_manager.users.write() {
        channel.insert(current_user.uuid.clone(), Some(tx));
    }

    while let Some(Ok(ws_message)) = stream.next().await {
        let ws_payload =
            match ws_message {
                Message::Text(text) => serde_json::from_str::<WsPayload>(&text)
                    .unwrap_or_else(|_| WsPayload::default()),
                Message::Binary(bytes) => serde_json::from_slice::<WsPayload>(&bytes)
                    .unwrap_or_else(|_| WsPayload::default()),
                _ => WsPayload::default(),
            };
        let ws_payload = match (ws_payload.op_code, ws_payload.message.as_str()) {
            (1, "send") => WsPayload::new(10, "fetch".to_string()),
            _ => WsPayload::default(),
        };
        broadcast_msg(ws_payload, &rooms_manager).await;
    }

    if let Ok(mut channel) = rooms_manager.users.write() {
        channel.insert(current_user.uuid.clone(), None);
    }
}

#[cfg(feature = "ssr")]
async fn broadcast_msg(message: WsPayload, rooms_manager: &RoomsManager) {
    if let Ok(channel) = rooms_manager.users.read() {
        channel.iter().for_each(|(_, some_tx)| {
            if let Some(tx) = some_tx {
                if tx.send(message.clone()).is_err() {
                    leptos::logging::log!("unable to broadcast the message");
                }
            }
        });
    }
}
