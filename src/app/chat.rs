use crate::app::{
    create_or_join::{CreateOrJoinRoomButton, PopUpRoomForm},
    current_user::CurrentUser,
    logout::LogoutButton,
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
    let user = expect_context::<CtxProvider>().user;

    let (path, set_path) = create_signal(String::new());

    create_render_effect(move |_| {
        let mut path_name = window().location().pathname().unwrap();
        if !path_name.is_empty() {
            path_name.remove(0);
        }
        set_path.set(path_name);
    });

    let room_uuid = path.get_untracked();

    let message_ref = create_node_ref::<html::Input>();

    // ----

    let publish_msg = create_action(|msg_data: &MsgData| publish_msg(msg_data.clone()));

    let send = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let room_uuid = room_uuid.clone();

        let text = message_ref
            .get()
            .expect("input element doesn't exist")
            .value();

        let msg = Msg::Text(text);
        let msg_data = MsgData::new(room_uuid, user.clone(), msg);

        publish_msg.dispatch(msg_data);
        message_ref
            .get()
            .expect("input element doesn't exist")
            .set_value("");
    };

    view! {
        <div class="h-full bg-transparent grow flex flex-col" id="chat-interface">
            <div class="grow bg-transparent px-4" id="chat-log">
                // <Suspense fallback=move || view! { <p class="text-white font-sans">"loading..."</p> }>
                //     {move || {
                //         subscribe_msg.get().map(|msg| view! { <p>{ msg }</p> })
                //     }}
                // </Suspense>
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
    let (display, set_display) = create_signal("hidden");

    let user = create_memo(move |_| expect_context::<CtxProvider>().user);
    let user_name = user.get_untracked().user_name;

    view! {
        <div class="size-11/12 flex flex-row mx-4 my-4 bg-slate-800/[.65] rounded-xl">
            <PopUpRoomForm display=display/>
            <div class="flex flex-col items-center h-full w-[70px] bg-slate-950/[.65] rounded-l-xl pb-2" id="main-channel-navigation">
                <div class="grow bg-transparent py-2">
                    <CurrentUser name=user_name/>
                </div>
                <CreateOrJoinRoomButton read_sig=display write_sig=set_display/>
            </div>
            <div class="h-full w-[300px] bg-transparent rounded-l-xl flex flex-col" id="inner-channel-navigation">
                <div class="h-20 w-full bg-transparent">
                    <LogoutButton/>
                </div>
                <div class="grow w-full bg-slate-800/[.65] rounded-bl-xl"></div>
            </div>
            <Outlet/>
        </div>
    }
}
