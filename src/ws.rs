use crate::{
    auth_model::AuthSession,
    state::{AppState, ConStatus, Room, UserConn},
    user_model::UserData,
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::{ErrorResponse, IntoResponse},
};
use futures::{SinkExt, StreamExt};
use leptos::*;
use tokio::sync::mpsc;

#[axum::debug_handler]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    auth_session: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let pool = app_state.pool.clone();
    let current_user = auth_session
        .current_user
        .expect("There's no current user detected from auth session!");
    let uuid = current_user.uuid;

    let user_data = UserData::get_from_id(&uuid, &pool)
        .await
        .expect("There's no user with that id");

    let user_conn = UserConn {
        user_name: user_data.user_name,
        uuid: user_data.uuid,
        status: ConStatus::Connected,
        sender: None,
    };

    match app_state
        .room
        .clone()
        .write()
        .expect("Unable to write to the lock")
        .insert(user_data.email.clone(), user_conn)
    {
        _ => Ok(ws.on_upgrade(move |socket| {
            ws_connection(socket, user_data.email, app_state.room.clone())
        })),
    }
}

async fn ws_connection(socket: WebSocket, email: String, room: Room) {
    let mut user_conn = room.write().unwrap().get_mut(&email).unwrap().clone();
    logging::log!("user: {}, is {:?}", email.clone(), user_conn.status);

    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match sender.send(msg).await {
                Ok(_) => (),
                Err(err) => eprintln!("Unable to send ws message: {}", err),
            }
        }
        sender.close().await.unwrap();
    });

    user_conn.sender = Some(tx);
    room.write()
        .unwrap()
        .insert(email.clone(), user_conn.clone())
        .unwrap();

    while let Some(Ok(message)) = receiver.next().await {
        logging::log!("message received: {:?}", message);
        broadcast_msg(message, &email, &room).await;
    }

    user_conn.status = ConStatus::Disconnected;
    user_conn.sender = None;
    room.write()
        .unwrap()
        .insert(email.clone(), user_conn.clone());
}

async fn broadcast_msg(msg: Message, email: &String, room: &Room) {
    for (other_email, user) in room.read().unwrap().iter() {
        if let (Some(tx), Message::Text(_)) = (user.sender.clone(), msg.clone()) {
            if other_email != email {
                tx.send(msg.clone())
                    .map_err(|err| logging::log!("Unable to send message: {:?}", err))
                    .unwrap();
            }
        }
    }
}
