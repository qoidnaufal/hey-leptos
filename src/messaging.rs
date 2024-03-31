use {
    crate::{models::message_model::MsgData, state::ssr::AppState},
    axum::extract::{Path, State},
    axum::response::sse::{Event, KeepAlive, Sse},
    futures::stream::{Stream, StreamExt},
};

pub async fn stream_message(
    Path(room_uuid): Path<String>,
    State(app_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let pool = app_state.pool;

    leptos::logging::log!("captured path for sse is: {}", &room_uuid);

    let msg_stream = MsgData::stream_msg(&room_uuid, &pool).await.unwrap();

    let msg_stream = msg_stream.map(|notif| {
        let msg = notif.map_err(|err| axum::Error::new(err))?.data;
        Event::default().json_data(msg)
    });

    Sse::new(msg_stream)
        .keep_alive(KeepAlive::default().interval(std::time::Duration::from_secs(1)))
}
