use leptos::*;
use leptos_router::A;

#[component]
pub fn JoinedChannels(
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
            key=|(_, room_uuid)| room_uuid.clone()
            children=move |(idx, _)| {
                let channel = create_memo(move |_| {
                    channels_resource.and_then(|data| {
                        data.get(idx)
                            .unwrap()
                            .clone()
                    })
                    .unwrap_or(Ok((String::new(), String::new())))
                    .unwrap_or_default()
                });

                view! {
                    <A href=move || channel.get().0>
                        <button class="cursor-pointer text-center text-xl text-ellipsis overflow-hidden uppercase w-12 h-12 rounded-xl border-none bg-sky-500 hover:bg-green-300 active:bg-green-300 mt-2 px-2">
                            { move || channel.get().1 }
                        </button>
                    </A>
                }
            }
        />
    }
}
