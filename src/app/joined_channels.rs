use leptos::*;
use leptos_router::A;

#[server(FetchJoinedChannels, "/api", "GetJson")]
pub async fn fetch_joined_channels() -> Result<Vec<(String, String)>, ServerFnError> {
    use crate::{
        state::{auth, pool},
        user_model::UserData,
    };

    let auth = auth()?;
    let pool = pool()?;

    let current_user = auth
        .current_user
        .ok_or_else(|| ServerFnError::new("There is no current user!"))?;

    let user_data = UserData::get_from_uuid(&current_user.uuid, &pool)
        .await
        .ok_or_else(|| ServerFnError::new("Invalid user: Entry not found in db"))?;

    Ok(user_data.joined_channels)
}

#[component]
pub fn UserChannels(
    channels_resource: Resource<(usize, usize), Result<Vec<(String, String)>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <For
        {move || channels_resource.track()}
            each=move || {
                channels_resource
                    .get()
                    .unwrap_or_else(|| Ok(Vec::<(String, String)>::new()))
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate()
            }
            key=|(_, channel_tuple)| channel_tuple.clone()
            children=move |(idx, _)| {
                let channel = create_memo(move |_| {
                    channels_resource.and_then(|vec| {
                        vec.get(idx)
                            .unwrap()
                            .clone()
                    })
                    .unwrap_or(Ok((String::new(), String::new())))
                    .unwrap_or_default()
                });

                let room_uuid = channel.get().0;
                let path = format!("/channel/{}", room_uuid);

                view! {
                    <A href=path>
                        <div
                            class="flex cursor-pointer text-center items-center text-xl text-ellipsis overflow-hidden uppercase w-12 h-12 rounded-xl bg-sky-500 hover:bg-green-300 active:bg-green-300 mt-2 px-2">
                            { move || channel.get().1 }
                        </div>
                    </A>
                }
            }
        />
    }
}
