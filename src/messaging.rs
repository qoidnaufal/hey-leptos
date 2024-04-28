#[cfg(feature = "ssr")]
use {
    crate::{
        models::{
            message_model::WsPayload,
            user_model::{User, UserData},
        },
        state::{auth::AuthSession, db::Database, rooms_manager::RoomsManager, AppState},
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
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    },
    tokio::sync::broadcast,
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
    let (mut sink, mut stream) = ws.split();
    let (tx, _) = broadcast::channel::<WsPayload>(100);
    let mut rx = tx.subscribe();
    {
        let channels = rooms_manager.channels.write().unwrap();
        channels
            .iter()
            .for_each(|(room_uuid, room)| println!("{}: {:?}", room_uuid, room));
    }
    // spawn for every connected user or spawn for each channel?
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json_data = serde_json::to_vec(&msg).unwrap();
            let response = Message::Binary(json_data);
            if sink.send(response).await.is_err() {
                break;
            }
        }
        sink.close().await.unwrap()
    });
    while let Some(Ok(ws_message)) = stream.next().await {
        let recv_payload = match ws_message {
            Message::Text(text) => {
                serde_json::from_str::<WsPayload>(&text).unwrap_or_else(|_| WsPayload::default())
            }
            _ => WsPayload::default(),
        };
        let channel_payload = match recv_payload.op_code {
            1 => WsPayload::new(11, recv_payload.message.clone()),
            _ => WsPayload::default(),
        };
        broadcast_msg(channel_payload, &tx).await;
    }
}

#[cfg(feature = "ssr")]
async fn broadcast_msg(channel_payload: WsPayload, tx: &broadcast::Sender<WsPayload>) {
    tx.send(channel_payload).unwrap();
}
