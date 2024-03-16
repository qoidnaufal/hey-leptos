use super::{
    create_or_join::{CreateNewRoom, CreateOrJoinRoomButton, JoinRoom, PopUpRoomForm},
    current_user::{CurrentUser, UserMenu},
    joined_channels::{fetch_joined_channels, UserChannels},
    ErrorTemplate,
};
use crate::error_template::AppError;
use leptos::*;
use leptos_router::Outlet;

#[server]
async fn validate_path(path: String) -> Result<(), ServerFnError> {
    use crate::state::rooms_manager;

    let rooms_manager = rooms_manager()?;

    let uuid = path
        .strip_prefix("/channel/")
        .expect("Valid uuid is needed")
        .to_string();

    rooms_manager
        .validate_uuid(uuid)
        .map_err(|err| ServerFnError::new(format!("{:?}", err)))
}

#[server(PublishMsg)]
async fn publish_msg(text: String, room_uuid: String) -> Result<(), ServerFnError> {
    use crate::models::message_model::{Msg, MsgData};
    use crate::rooms_manager::SelectClient;
    use crate::state::{auth, pool, rooms_manager};

    let auth = auth()?;
    let pool = pool()?;
    let rooms_manager = rooms_manager()?;

    let user = auth
        .current_user
        .ok_or_else(|| ServerFnError::new("Auth does not contain user"))?;

    rooms_manager
        .init(SelectClient::Publisher)
        .await
        .map_err(|err| ServerFnError::new(format!("{:?}", err)))?;

    let msg = Msg::Text(text);
    let msg_data = MsgData::new(room_uuid.clone(), user, msg.clone());

    logging::log!("msg published to channel {}: {:?}\n", &room_uuid, msg);

    match rooms_manager
        .publish_msg(room_uuid, msg)
        .await
        .map_err(|err| ServerFnError::new(format!("{:?}", err)))
    {
        Ok(_) => msg_data
            .insert_into_db(&pool)
            .await
            .map_err(|err| ServerFnError::new(format!("{:?}", err))),
        Err(err) => Err(ServerFnError::new(format!("{:?}", err))),
    }
}

#[component]
pub fn Channel() -> impl IntoView {
    let path = leptos_router::use_location().pathname;

    let path_resource = create_resource(move || path.get(), |path| validate_path(path));

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
                                class="h-full bg-transparent grow flex flex-col"
                                id="chat-interface"
                            >
                                <div class="grow bg-transparent px-4" id="chat-log"></div>
                                <form
                                    on:submit=send
                                    class="bg-transparent px-4 h-32 flex flex-row items-center"
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
#[component]
pub fn ChatPage() -> impl IntoView {
    let (display_room_form, set_display_room_form) = create_signal("hidden");
    let (display_user_menu, set_display_user_menu) = create_signal("hidden");

    // ---- handle channels fetching

    let create_room_action = create_server_action::<CreateNewRoom>();
    let join_room_action = create_server_action::<JoinRoom>();

    let channels_resource = create_local_resource(
        move || {
            (
                create_room_action.version().get(),
                join_room_action.version().get(),
            )
        },
        |_| fetch_joined_channels(),
    );

    view! {
        <div class="size-11/12 flex flex-row mx-4 my-4 bg-slate-800/[.65] rounded-xl">
            <div
                id="outer-navigation-container"
                class="flex flex-col w-[370px] h-full rounded-l-xl bg-transparent"
            >
                <div
                    id="current-user-container"
                    class="h-[50px] w-full rounded-tl-xl bg-transparent"
                >
                    <CurrentUser display_user_menu set_display_user_menu/>
                    <UserMenu display_user_menu/>
                </div>
                <div
                    id="inner-navigation"
                    class="flex flex-row bg-transparent rounded-bl-xl w-[370px] h-full"
                >
                    <div
                        id="channels-navigation"
                        class="flex flex-col items-center h-full w-[70px] bg-slate-950/[.65] rounded-bl-xl pb-2"
                    >
                        <div id="channel-list" class="flex flex-col grow bg-transparent">
                            <UserChannels channels_resource/>
                        </div>
                        <CreateOrJoinRoomButton display_room_form set_display_room_form/>
                        <PopUpRoomForm display_room_form create_room_action join_room_action/>
                    </div>
                    <div
                        id="sub-channel-navigation"
                        class="h-full w-[300px] bg-transparent rounded-l-xl flex flex-col"
                    >
                        <div
                            id="sub-channels"
                            class="grow w-full bg-slate-800/[.65] rounded-bl-xl"
                        ></div>
                    </div>
                </div>
            </div>
            <Outlet/>
        </div>
    }
}
