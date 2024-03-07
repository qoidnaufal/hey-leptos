use crate::{/*message_model::MsgData,*/ rooms_manager::SelectClient, state::AppState};
use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
};
use fred::interfaces::{EventInterface, PubsubInterface};
use futures::stream::{self, Stream};
use leptos::*;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;

#[axum::debug_handler]
pub async fn msg_subscriber(
    State(app_state): State<AppState>,
    Path(room_uuid): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (response, set_response) = create_signal(String::new());
    logging::log!("path from axum handler: {}", room_uuid);

    let rooms_manager = app_state.rooms_manager;

    rooms_manager.init(SelectClient::Subscriber).await.unwrap();

    let mut msg_stream = rooms_manager.subscriber_client.message_rx();

    let subscriber_task = tokio::spawn(async move {
        while let Ok(message) = msg_stream.recv().await {
            let resp = message.value.clone().convert::<String>().unwrap();
            set_response.set(resp.clone());
            logging::log!("message from axum handler: {:?}", resp);
        }
    });

    let resp = response.get_untracked();

    let stream = stream::repeat_with(move || Event::default().data(resp.clone()))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    let resubscribe_task = rooms_manager.subscriber_client.manage_subscriptions();

    rooms_manager
        .subscriber_client
        .subscribe(room_uuid.as_str())
        .await
        .unwrap();

    rooms_manager.quit(SelectClient::Subscriber).await.unwrap();

    let _ = subscriber_task.await;
    let _ = resubscribe_task.await;

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep alive"),
    )
}
