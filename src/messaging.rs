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
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    },
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
        Ok(_) => Ok(ws.on_upgrade(move |socket| {
            handle_ws_connection(socket, rooms_manager, current_user, room_uuid.clone())
        })),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

#[cfg(feature = "ssr")]
async fn handle_ws_connection(
    ws: WebSocket,
    rooms_manager: RoomsManager,
    current_user: User,
    room_uuid: String,
) {
    if let Ok(mut channel_list) = rooms_manager.channels.write() {
        if !channel_list.contains_key(&room_uuid) {
            let users = Arc::new(RwLock::new(HashMap::new()));
            channel_list.insert(room_uuid.clone(), users);
        }
    }

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

    if let Ok(channel_list) = rooms_manager.channels.write() {
        if let Some(channel_users) = channel_list.get(&room_uuid) {
            if let Ok(mut users) = channel_users.write() {
                users.insert(current_user.uuid.clone(), Some(tx));
            }
        }
    }

    while let Some(Ok(ws_message)) = stream.next().await {
        let recv_payload = match ws_message {
            Message::Text(text) => {
                serde_json::from_str::<WsPayload>(&text).unwrap_or_else(|_| WsPayload::default())
            }
            _ => WsPayload::default(),
        };
        let send_payload = match recv_payload.op_code {
            1 => WsPayload::new(
                11,
                "todo: instruction about which msg to fetch next".to_string(),
            ),
            _ => WsPayload::default(),
        };
        broadcast_msg(send_payload, &rooms_manager, room_uuid.clone()).await;
    }

    if let Ok(channel_list) = rooms_manager.channels.write() {
        if let Some(channel_users) = channel_list.get(&room_uuid) {
            if let Ok(mut users) = channel_users.write() {
                users.retain(|id, _| id.clone() != current_user.uuid.clone());
            }
        }
    }
}

#[cfg(feature = "ssr")]
async fn broadcast_msg(message: WsPayload, rooms_manager: &RoomsManager, room_uuid: String) {
    if let Ok(channel_list) = rooms_manager.channels.read() {
        if let Some(channel_users) = channel_list.get(&room_uuid) {
            if let Ok(users) = channel_users.read() {
                users.iter().for_each(|(_, some_tx)| {
                    if let Some(tx) = some_tx {
                        if tx.send(message.clone()).is_err() {
                            leptos::logging::log!("unable to broadcast the message");
                        }
                    }
                });
            }
        }
    }
}
