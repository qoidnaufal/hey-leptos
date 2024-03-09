use leptos::*;

#[component]
pub fn JoinedChannels(
    channels_resource: Resource<(usize, usize), Result<Vec<String>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <For
        {move || channels_resource.track()}
            each=move || {
                channels_resource
                    .get()
                    .unwrap_or_else(|| Ok(Vec::new()))
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate()
            }
            key=|(_, room_uuid)| room_uuid.clone()
            children=move |(idx, _)| {
                let room_uuid = create_memo(move |_| {
                    channels_resource.and_then(|data| {
                        data.get(idx)
                            .unwrap()
                            .clone()
                    })
                    .unwrap_or(Ok(String::new()))
                    .unwrap_or_default()
                });
                let navigate = leptos_router::use_navigate();
                let view_channel = move |_| {
                    navigate(&room_uuid.get(), Default::default());
                };
                view! {
                    <div on:click=view_channel class="cursor-pointer text-xl text-center w-12 h-12 rounded-xl border-none bg-sky-500 mt-2">"🏠"</div>
                }
            }
        />
    }
}
