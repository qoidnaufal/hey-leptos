use crate::{
    auth_model::ssr::AuthSession,
    state::{AppState, ConStatus, Room, UserConn},
};
use axum::{
    extract::{
        ws::{rejection::WebSocketUpgradeRejection, Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use leptos::*;
use tokio::sync::mpsc;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    auth_session: AuthSession,
) -> Result<impl IntoResponse, WebSocketUpgradeRejection> {
    let db = app_state.db.clone();
    let uuid = auth_session.id;
    let user_data = db.get_user_by_id(&uuid).await.unwrap();

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
        // broadcast_msg(message, tx.clone()).await;
    }

    user_conn.status = ConStatus::Disconnected;
    user_conn.sender = None;
    room.write()
        .unwrap()
        .insert(email.clone(), user_conn.clone());
}

async fn broadcast_msg(msg: Message, email: &String, room: &Room) {
    logging::log!("{:?}", room);
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
