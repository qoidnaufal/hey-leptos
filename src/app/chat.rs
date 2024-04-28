use super::{
    create_or_join::{CreateNewRoom, CreateOrJoinRoomButton, JoinRoom, PopUpRoomForm},
    current_user::{get_avatar_and_name, CurrentUser, UserMenu},
    joined_channels::{fetch_joined_channels, JoinedChannels},
    logout::LogoutAction,
};
use leptos::*;
use leptos_router::Outlet;
use leptos_use::{use_websocket, UseWebsocketReturn};
use std::rc::Rc;

#[derive(Clone)]
pub struct WebsocketCtx {
    pub send: Rc<dyn Fn(&str)>,
    pub message_bytes: Signal<Option<Vec<u8>>>,
}

impl WebsocketCtx {
    fn new(send: Rc<dyn Fn(&str)>, message_bytes: Signal<Option<Vec<u8>>>) -> Self {
        Self {
            send,
            message_bytes,
        }
    }
}

#[component]
pub fn ChatPage(logout_action: LogoutAction) -> impl IntoView {
    let (display_room_form, set_display_room_form) = create_signal(false);
    let (display_user_menu, set_display_user_menu) = create_signal(false);
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
    let user_resource = create_resource(|| (), |_| get_avatar_and_name());
    let UseWebsocketReturn {
        send,
        message_bytes,
        ..
    } = use_websocket("ws://localhost:4321/ws");
    provide_context(logout_action);
    provide_context(user_resource);
    provide_context(WebsocketCtx::new(Rc::new(send), message_bytes));

    view! {
        <div class="block absolute m-auto left-0 right-0 top-0 bottom-0 w-[91.6667%] h-[91.6667%] max-h-[91.6667%] max-w-[91.6667%] flex flex-row bg-slate-800/[.65] rounded-xl">
            <div
                id="outer-navigation-container"
                class="flex flex-col w-[370px] h-full rounded-l-xl bg-transparent"
            >
                <div
                    id="current-user-container"
                    class="h-[50px] w-full rounded-tl-xl bg-transparent"
                >
                    <CurrentUser display_user_menu set_display_user_menu user_resource/>
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
                            <JoinedChannels channels_resource/>
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
