use crate::{
    auth_model::AuthSession,
    message_model::{Msg, MsgData, MsgSender},
    rooms_manager::{RoomUser, RoomsManager},
    state::AppState,
    user_model::UserData,
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::{ErrorResponse, IntoResponse},
};
use futures::{SinkExt, StreamExt};
use leptos::*;
use tokio::sync::mpsc;

#[axum::debug_handler]
pub async fn ws_handler(
    Path(room_id): Path<String>,
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    auth_session: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    // maybe i also need path here to extract the room
    let current_user = auth_session
        .current_user
        .expect("There's no current user detected from auth session!");
    let uuid = current_user.uuid;

    let user_data = UserData::get_from_uuid(&uuid, &app_state.pool)
        .await
        .expect("There's no user with that id");

    let room_user = RoomUser::from_user_data(&user_data);

    // here i just need to supply the appstate (or maybe the room) & RoomUser to my ws_connection

    match app_state
        .rooms_manager
        .clone()
        .write()
        .expect("Unable to write to the lock")
        .insert(room_id, user_conn)
    {
        _ => {
            Ok(ws.on_upgrade(move |socket| {
                ws_connection(socket, user_data.email, app_state.clone())
            }))
        }
    }
}

async fn ws_connection(socket: WebSocket, email: String, app_state: AppState) {
    let room = app_state.room.clone();
    let mut user_conn = room.write().unwrap().get_mut(&email).unwrap().clone();
    logging::log!("user: {}, is connected", email.clone());

    let (mut ws_sender, mut ws_reader) = socket.split();

    // here i'll need to change it into RoomsManager instead, where i provide the tx & rx
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    tokio::spawn(async move {
        // channeled msg is received here, and resent by ws to the clients
        while let Some(msg) = rx.recv().await {
            match ws_sender.send(msg).await {
                Ok(_) => (),
                Err(err) => eprintln!("Unable to send ws message: {}", err),
            }
        }
        ws_sender.close().await.unwrap();
    });

    user_conn.sender = Some(tx);
    room.write()
        .unwrap()
        .insert(email.clone(), user_conn.clone())
        .unwrap();

    // ws is receiving messages from the clients here
    while let Some(Ok(msg)) = ws_reader.next().await {
        logging::log!("message received: {:?}", msg);

        // im trying to store the message into db once the message is received by ws here
        // store_into_db(&msg, &email, &app_state).await;

        channel_msg_to_sender(msg, &email, &room).await;
    }

    user_conn.sender = None;
    room.write()
        .unwrap()
        .insert(email.clone(), user_conn.clone());
}

async fn channel_msg_to_sender(msg: Message, _email: &String, room: &Room) {
    for (_, user) in room.read().unwrap().iter() {
        if let (Some(tx), Message::Text(_)) = (user.sender.clone(), msg.clone()) {
            tx.send(msg.clone())
                .map_err(|err| logging::log!("Unable to send message: {:?}", err))
                .unwrap();
        }
    }
}

async fn _store_into_db(msg: &Message, email: &String, app_state: &AppState) {
    let extract_message = match msg {
        Message::Text(ref string) => Msg::Text(string.clone()),
        Message::Binary(ref vec) => Msg::Bytes(vec.clone()),
        _ => Msg::Other,
    };

    if let Msg::Other = extract_message {
        ()
    } else {
        let user_data = UserData::get_from_email(&email, &app_state.pool)
            .await
            .unwrap()
            .unwrap();
        let message_sender = MsgSender::from_user_data(user_data);
        let msg_data = MsgData::new(message_sender, extract_message);

        msg_data.insert_into_db(&app_state.pool).await.unwrap();
    }
}
