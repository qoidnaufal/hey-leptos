use leptos::*;
use leptos_router::A;
use serde::{Deserialize, Serialize};

type ChannelsResource = Resource<(usize, usize), Result<Vec<JoinedChannel>, ServerFnError>>;

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JoinedChannel {
    uuid: String,
    name: String,
}

#[cfg(feature = "ssr")]
impl JoinedChannel {
    fn new(uuid: String, name: String) -> Self {
        Self { uuid, name }
    }
}

// test

#[server(FetchJoinedChannels, "/api", "GetJson")]
pub async fn fetch_joined_channels() -> Result<Vec<JoinedChannel>, ServerFnError> {
    use crate::{
        error::ApiError,
        models::user_model::UserData,
        state::{auth, pool, rooms_manager},
    };
    use futures::future::join_all;

    let auth = auth()?;
    let pool = pool()?;
    let rooms_manager = rooms_manager()?;
    let current_user = auth
        .current_user
        .ok_or_else(|| ServerFnError::new("There is no current user!"))?;
    let user_data = UserData::get_from_uuid(&current_user.uuid, &pool)
        .await
        .ok_or_else(|| ServerFnError::new("Invalid user: Entry not found in db"))?;
    let joined_channels = user_data
        .joined_channels
        .iter()
        .map(|room_uuid| async {
            let room_name = rooms_manager.get_room_name(room_uuid, &pool).await?;
            Ok::<JoinedChannel, ApiError>(JoinedChannel::new(room_uuid.clone(), room_name))
        })
        .map(|res| async { res.await.unwrap_or_default() });
    let joined_channels = join_all(joined_channels).await;
    Ok(joined_channels)
}

#[component]
pub fn JoinedChannels(channels_resource: ChannelsResource) -> impl IntoView {
    view! {
        <For
        {move || channels_resource.track()}
            each=move || {
                channels_resource
                    .get()
                    .unwrap_or_else(|| Ok(Vec::<JoinedChannel>::new()))
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate()
            }
            key=|(_, joined_channel)| joined_channel.uuid.clone()
            children=move |(idx, _)| {
                let channel = create_memo(move |_| {
                    channels_resource.and_then(|result| {
                        result.get(idx).unwrap().clone()
                    })
                    .unwrap_or(Ok(JoinedChannel::default()))
                    .unwrap_or_default()
                });

                let room_uuid = channel.get().uuid;
                let path = leptos_router::use_location().pathname;

                let active = move |id| {move ||
                    if path.get().contains(&id) {
                        "text-xl text-ellipsis overflow-hidden uppercase w-12 h-12 rounded-xl bg-green-300 font-bold border-4 border-sky-500 border-solid mt-2 px-2"
                    } else {
                        "text-xl text-white text-ellipsis overflow-hidden uppercase w-12 h-12 rounded-xl bg-sky-500 hover:bg-green-300 border-none mt-2 px-2"
                    }};

                view! {
                    <A href=channel.get().uuid>
                        <button class={active(room_uuid)}>
                            { move || channel.get().name }
                        </button>
                    </A>
                }
            }
        />
    }
}
