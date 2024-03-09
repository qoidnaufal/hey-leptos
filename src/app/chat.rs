use crate::app::{
    create_or_join::{
        create_new_room, join_room, CreateOrJoinRoomButton, CreateRoomPayload, JoinRoomPayload,
        PopUpRoomForm,
    },
    current_user::{CurrentUser, UserMenu},
    fetch_user_channels,
    joined_channels::JoinedChannels,
    CtxProvider,
};
use crate::message_model::{Msg, MsgData};
use leptos::*;
use leptos_router::Outlet;

#[server]
async fn publish_msg(msg_data: MsgData) -> Result<(), ServerFnError> {
    use crate::rooms_manager::SelectClient;
    use crate::state::rooms_manager;

    let room_uuid = msg_data.channel.clone();

    let rooms_manager = rooms_manager()?;

    let message =
        serde_json::to_string(&msg_data).map_err(|err| ServerFnError::new(format!("{:?}", err)))?;

    rooms_manager
        .init(SelectClient::Publisher)
        .await
        .map_err(|err| ServerFnError::new(format!("{:?}", err)))?;

    let msg = Msg::Text(message);

    logging::log!("message published: {:?}\n", msg);

    rooms_manager
        .publish_msg(room_uuid, msg)
        .await
        .map_err(|err| ServerFnError::new(format!("{:?}", err)))?;

    Ok(())
}

#[component]
pub fn Channel() -> impl IntoView {
    let path = leptos_router::use_location().pathname;
    let mut room_uuid = path.get_untracked();
    room_uuid.remove(0);

    // ----

    let user = create_memo(move |_| expect_context::<CtxProvider>().user);

    // ----

    let message_ref = create_node_ref::<html::Input>();

    let publish_msg = create_action(|msg_data: &MsgData| publish_msg(msg_data.clone()));

    let send = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let room_uuid = room_uuid.clone();

        let text = message_ref
            .get()
            .expect("input element doesn't exist")
            .value();

        let msg = Msg::Text(text);
        let msg_data = MsgData::new(room_uuid, user.get().clone(), msg);

        publish_msg.dispatch(msg_data);
        message_ref
            .get()
            .expect("input element doesn't exist")
            .set_value("");
    };

    view! {
        <div class="h-full bg-transparent grow flex flex-col" id="chat-interface">
            <div class="grow bg-transparent px-4" id="chat-log">
            </div>
            <form on:submit=send class="bg-transparent px-4 h-32 flex flex-row items-center">
                <input
                    id="input"
                    name="message"
                    _ref=message_ref
                    class="grow rounded-l-full h-12 text-white font-sans pl-2 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto text-base"
                />
                <button class="border-none h-12 w-fit px-2 bg-sky-500 hover:bg-green-300 hover:text-black rounded-r-full border-0 text-white font-sans">
                    "send"
                </button>
            </form>
        </div>
    }
}

#[component]
pub fn ChatPage() -> impl IntoView {
    let (display_room_form, set_display_room_form) = create_signal("hidden");
    let (display_user_menu, set_display_user_menu) = create_signal("hidden");

    let user = create_memo(move |_| expect_context::<CtxProvider>().user);
    let user_name = user.get_untracked().user_name;

    // ---- handle channels fetching

    let create_room_action =
        create_action(move |payload: &CreateRoomPayload| create_new_room(payload.clone()));
    let join_room_action =
        create_action(move |payload: &JoinRoomPayload| join_room(payload.clone()));

    let channels_resource = create_resource(
        move || {
            (
                create_room_action.version().get(),
                join_room_action.version().get(),
            )
        },
        |_| fetch_user_channels(),
    );

    view! {
        <div class="size-11/12 flex flex-row mx-4 my-4 bg-slate-800/[.65] rounded-xl">
            <div id="left-navigation" class="flex flex-col items-center h-full w-[70px] bg-slate-950/[.65] rounded-l-xl pb-2">
                <div id="channel-list" class="flex flex-col grow bg-transparent">
                    <JoinedChannels channels_resource/>
                </div>
                <CreateOrJoinRoomButton display_room_form set_display_room_form/>
                <PopUpRoomForm display_room_form create_room_action join_room_action/>
            </div>
            <div id="inner-navigation" class="h-full w-[300px] bg-transparent rounded-l-xl flex flex-col">
                <div class="h-[50px] w-full bg-transparent">
                    <CurrentUser user_name display_user_menu set_display_user_menu/>
                    <UserMenu display_user_menu/>
                </div>
                <div id="sub-channel" class="grow w-full bg-slate-800/[.65] rounded-bl-xl"></div>
            </div>
            <Outlet/>
        </div>
    }
}
