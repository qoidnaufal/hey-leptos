use super::app_error::{AppError, ErrorTemplate};
use crate::models::message_model::MsgResponse;
use chrono::Local;
use leptos::*;

#[server]
async fn validate_path(path: String) -> Result<(), ServerFnError> {
    use crate::state::{auth, pool, rooms_manager::RoomsManager};

    let auth = auth()?;
    let pool = pool()?;

    if !auth.is_authenticated() {
        return Err(ServerFnError::new(
            "User isn't authenticated to see the channel",
        ));
    }

    if path.starts_with("/channel/") {
        let room_uuid = path
            .strip_prefix("/channel/")
            .expect("Valid uuid is needed")
            .to_string();

        RoomsManager::validate_uuid(room_uuid, &pool)
            .await
            .map_err(|err| ServerFnError::new(format!("{:?}", err)))
    } else {
        Ok(())
    }
}

#[server(PublishMsg)]
async fn publish_msg(text: String, room_uuid: String) -> Result<(), ServerFnError> {
    use crate::models::message_model::MsgData;
    use crate::state::{auth, pool};
    use chrono::Utc;

    let auth = auth()?;
    let pool = pool()?;

    let user = auth
        .current_user
        .ok_or_else(|| ServerFnError::new("Auth does not contain user"))?;

    let created_at = Utc::now();

    logging::log!(
        "[{}] > on channel: {}:\n                                 > {} published: {:?}\n",
        &created_at,
        &room_uuid,
        &user.user_name,
        &text
    );

    let msg_data = MsgData::new(room_uuid.clone(), user.uuid, text, created_at);

    msg_data
        .insert_into_db(&pool)
        .await
        .map_err(|err| ServerFnError::new(err))
}

#[server(FetchMsg, "/api", "GetJson")]
async fn fetch_msg(room_uuid: String) -> Result<Vec<MsgResponse>, ServerFnError> {
    use crate::models::message_model::MsgResponse;
    use crate::state::pool;

    let pool = pool()?;

    let room_uuid = room_uuid
        .strip_prefix("/channel/")
        .ok_or_else(|| ServerFnError::new("Invalid path"))?
        .to_string();

    match MsgResponse::get_all_msg(&room_uuid, &pool).await {
        Ok(mut vec_msg) => {
            vec_msg.sort();
            Ok(vec_msg)
        }
        Err(err) => Err(ServerFnError::new(err)),
    }
}

#[component]
pub fn Channel() -> impl IntoView {
    let path = leptos_router::use_location().pathname;

    let path_resource = create_resource(move || path.get(), validate_path);

    let msg_resource = create_resource(move || path.get(), fetch_msg);

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || {
                match path_resource.get().unwrap_or(Ok(())) {
                    Ok(_) => {
                        let message_ref = create_node_ref::<html::Input>();
                        let publish_msg = create_server_action::<PublishMsg>();
                        let send = move |ev: ev::SubmitEvent| {
                            ev.prevent_default();
                            let path = path.get();
                            let room_uuid = path
                                .strip_prefix("/channel/")
                                .expect("Provide valid uuid!")
                                .to_string();
                            let text = message_ref
                                .get()
                                .expect("input element doesn't exist")
                                .value();
                            publish_msg.dispatch(PublishMsg { text, room_uuid });
                            message_ref.get().expect("input element doesn't exist").set_value("");
                        };

                        view! {
                            <div
                                class="h-full w-full bg-transparent flex pt-2 flex-col overflow-y-hidden"
                                id="chat-interface"
                            >
                                <div
                                    class="flex flex-col h-[44rem] w-full bg-transparent px-4 overflow-y-scroll"
                                    id="chat-log"
                                >
                                    <For
                                        {move || msg_resource.track()}
                                        each=move || {
                                            msg_resource
                                                .get()
                                                .unwrap_or_else(|| Ok(Vec::new()))
                                                .unwrap_or_default()
                                                .into_iter()
                                                .enumerate()
                                        }
                                        key=|(_, msg_response)| (msg_response.created_at.clone(), msg_response.msg_uuid.clone())
                                        children=move |(idx, _)| {
                                            let msg = create_memo(move |_| {
                                                msg_resource
                                                    .and_then(|vec| vec.get(idx).unwrap().clone())
                                                    .unwrap_or(Ok(MsgResponse::default()))
                                                    .unwrap_or_default()
                                            });

                                            view! {
                                                <div class="bg-transparent flex flex-row mt-2">
                                                    <div class="flex flex-shrink-0 justify-center items-center pb-1 size-9 bg-sky-500 rounded-full text-white hover:text-black hover:bg-green-300 uppercase font-sans text-2xl text-center">
                                                        {move || msg.get().msg_sender.unwrap().avatar.get_view()}
                                                    </div>
                                                    <div class="flex flex-col ml-2 rounded-md bg-slate-300 px-2">
                                                        <div class="flex flex-row content-start">
                                                            <p>
                                                                <span class="font-sans text-indigo-500 text-lg">
                                                                    {move || msg.get().msg_sender.unwrap().user_name}
                                                                </span>
                                                                <span class="font-sans text-black/[.65] text-xs ml-2">
                                                                    {move || msg.get().created_at.with_timezone(&Local).format("%d/%m/%Y %H:%M").to_string()}
                                                                </span>
                                                            </p>
                                                        </div>
                                                        <p class="py-1 font-sans text-black text-wrap">
                                                            {move || msg.get().message}
                                                        </p>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                                <form
                                    on:submit=send
                                    class="px-4 h-32 flex flex-row items-center"
                                >
                                    <input
                                        id="input"
                                        name="message"
                                        _ref=message_ref
                                        placeholder="Type your message..."
                                        class="grow rounded-md h-12 text-white font-sans pl-2 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto text-base"
                                    />
                                </form>
                            </div>
                        }
                            .into_view()
                    }
                    Err(_) => {
                        let mut outside_errors = Errors::default();
                        outside_errors.insert_with_default_key(AppError::RoomDoesNotExist);
                        view! {
                            <div class="flex h-full bg-transparent grow items-center justify-center">
                                <ErrorTemplate outside_errors/>
                            </div>
                        }.into_view()
                    }
                }
            }}

        </Transition>
    }
}
