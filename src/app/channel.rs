use {
    super::{
        app_error::{AppError, ErrorTemplate},
        channel_header::ChannelHeader,
    },
    crate::{
        models::{
            message_model::{MsgResponse, WsPayload},
            user_model::User,
        },
        state::rooms_manager::Room,
    },
    chrono::Local,
    leptos::*,
    leptos_use::{use_websocket, UseWebsocketReturn},
};

#[server]
async fn validate_path(path: String) -> Result<Room, ServerFnError> {
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
            .expect("Valid uuid is needed");

        RoomsManager::validate_uuid(room_uuid, &pool)
            .await
            .map_err(|err| ServerFnError::new(err))
    } else {
        Err(ServerFnError::new("Invalid path"))
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
            // vec_msg.reverse();
            Ok(vec_msg)
        }
        Err(err) => Err(ServerFnError::new(err)),
    }
}

#[component]
pub fn Channel() -> impl IntoView {
    let path = leptos_router::use_location().pathname;

    let path_resource = create_resource(move || path.get(), validate_path);

    let user_resource = expect_context::<Resource<(), Result<User, ServerFnError>>>();

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || {
                match path_resource.get().unwrap_or(Err(ServerFnError::new("Invalid path"))) {
                    Ok(room) => {
                        let room_uuid = path.get();
                        let room_uuid = room_uuid
                            .strip_prefix("/channel/")
                            .expect("Provide valid uuid!");

                        let UseWebsocketReturn { open, close, send, message_bytes, .. } = use_websocket(&format!("ws://localhost:4321/ws/{}", room_uuid));

                        let msg_resource = create_resource(move || path.get(), fetch_msg);

                        let message_input = create_node_ref::<html::Div>();

                        let publish_msg = create_server_action::<PublishMsg>();

                        let handle_keyup = move |ev: ev::KeyboardEvent| {
                            ev.prevent_default();
                            if !ev.shift_key() && ev.key() == "Enter" && !message_input.get().expect("").inner_text().trim().is_empty() {
                                let path = path.get();
                                let room_uuid = path
                                    .strip_prefix("/channel/")
                                    .expect("Provide valid uuid!")
                                    .to_string();
                                let text = message_input
                                    .get()
                                    .expect("input element doesn't exist")
                                    .inner_text()
                                    .trim()
                                    .to_string();
                                let ws_payload = WsPayload::new(1, "new message".to_string());
                                publish_msg.dispatch(PublishMsg { text, room_uuid });
                                send(&serde_json::to_string(&ws_payload).unwrap());
                                message_input.get().expect("input element doesn't exist").set_inner_text("");
                            }
                        };

                        create_effect(move |_| {
                            if let Some(bytes) = message_bytes.get() {
                                let msg = serde_json::from_slice::<WsPayload>(&bytes).unwrap();
                                match msg.op_code {
                                    11 =>  msg_resource.refetch(),
                                     n => logging::log!("not yet registered op_code: {}", n)
                                }
                            }
                        });

                        let handle_focusin = move |_: ev::FocusEvent| {
                            if let Some(node) = message_input.get() {
                                node.set_inner_text("");
                            }
                        };

                        let handle_focusout = move |_: ev::FocusEvent| {
                            if let Some(node) = message_input.get() {
                                node.set_inner_text("Type your message...");
                            }
                        };

                        let _root = create_node_ref::<html::Ol>();

                        on_cleanup(close);

                        view! {
                            <div
                                on:load=move |_| open()
                                class="h-full w-full bg-transparent flex pt flex-col overflow-y-hidden"
                                id="chat-interface"
                            >
                                <ChannelHeader channel_name=room.room_name/>
                                <ol
                                    class="flex flex-col-reverse h-[44rem] w-full bg-transparent px-4 overflow-y-scroll"
                                    id="chat-log"
                                    _ref=_root
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
                                        key=|(_, msg_response)| (msg_response.created_at, msg_response.msg_uuid.clone())
                                        children=move |(idx, _)| {
                                            let msg = create_memo(move |_| {
                                                msg_resource
                                                    .and_then(|vec| {
                                                        let len = vec.len() - 1;
                                                        vec.get(len-idx).unwrap().clone()
                                                    })
                                                    .unwrap_or(Ok(MsgResponse::default()))
                                                    .unwrap_or_default()
                                            });

                                            view! { <MessageBubble msg user_resource/> }
                                        }
                                    />
                                </ol>
                                <form
                                    class="px-4 h-32 flex flex-row items-center"
                                >
                                    <div
                                        on:keyup=handle_keyup
                                        on:focusin=handle_focusin
                                        on:focusout=handle_focusout
                                        id="input"
                                        role="textbox"
                                        aria-multiline="true"
                                        contenteditable="true"
                                        _ref=message_input
                                        class="grow rounded-md min-h-12 max-h-[120px] h-fit overflow-y-scroll text-white font-sans mb-2 px-2 py-3 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto text-base flex items-center"
                                    >"Type your message..."</div>
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

#[component]
fn MessageBubble(
    msg: Memo<MsgResponse>,
    user_resource: Resource<(), Result<User, ServerFnError>>,
) -> impl IntoView {
    let sender = move || {
        msg.get().msg_sender.unwrap().user_name
            == user_resource
                .map(|user| user.clone().unwrap_or_default().user_name)
                .unwrap_or_default()
    };

    let receiver_class = "bg-transparent flex flex-row mt-2";
    let sender_class = "bg-transparent flex flex-row-reverse mt-2";

    view! {
        <li class=move || if sender() { sender_class } else { receiver_class }>
            <div class="flex flex-shrink-0 justify-center items-center pb-1 size-9 bg-sky-500 rounded-full text-white hover:text-black hover:bg-green-300 uppercase font-sans text-2xl text-center">
                {move || msg.get().msg_sender.unwrap().avatar.get_view()}
            </div>
            <div class=move || if sender() { "flex flex-col mr-2 rounded-md bg-green-300 px-2 max-w-[500px]" } else { "flex flex-col ml-2 rounded-md bg-slate-300 px-2 max-w-[500px]" }>
                <div class=move || if sender() { "flex flex-row flex-wrap justify-end" } else { "flex flex-row content-start" }>
                    {move || if sender() {
                        view! {
                            <p class="text-right">
                                <span class="font-sans text-black/[.65] text-xs mr-2">
                                    {move || msg.get().created_at.with_timezone(&Local).format("%d/%m/%Y %H:%M").to_string()}
                                </span>
                                <span class="font-sans text-indigo-500 text-lg">
                                    {move || msg.get().msg_sender.unwrap().user_name}
                                </span>
                            </p>
                        }
                    } else {
                        view! {
                            <p>
                                <span class="font-sans text-indigo-500 text-lg">
                                    {move || msg.get().msg_sender.unwrap().user_name}
                                </span>
                                <span class="font-sans text-black/[.65] text-xs ml-2">
                                    {move || msg.get().created_at.with_timezone(&Local).format("%d/%m/%Y %H:%M").to_string()}
                                </span>
                            </p>
                        }
                    }}
                </div>
                <pre class=move || if sender() { "py-1 font-sans text-black text-right text-wrap" } else { "py-1 font-sans text-black text-wrap" }>
                    {move || msg.get().message}
                </pre>
            </div>
        </li>
    }
}
