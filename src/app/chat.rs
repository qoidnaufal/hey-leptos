use crate::app::logout::LogoutButton;
use leptos::*;
use leptos_use::{use_websocket, UseWebsocketReturn};

// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// pub struct OnlineUser {
//     pub uuid: String,
//     pub user_name: String,
// }

// #[server(TrackRoom)]
// async fn track_room() -> Result<Vec<OnlineUser>, ServerFnError> {
//     use crate::state::Room;

//     let room =
//         use_context::<Room>().ok_or_else(|| ServerFnError::new("No database is detected!"))?;

//     let online = room
//         .read()
//         .expect("Unable to read the room")
//         .iter()
//         .map(|(_, user)| OnlineUser {
//             uuid: user.uuid.clone(),
//             user_name: user.user_name.clone(),
//         })
//         .collect::<Vec<_>>();

//     Ok(online)
// }

// #[component]
// fn OnlineList() -> impl IntoView {
//     let _on_load = create_server_action::<TrackRoom>();
//     let resource = create_resource(|| (), |_| async move { track_room().await });
//     view! {
//         {move || match resource.get() {
//             None => view! { <p>"Loading..."</p> }.into_view(),
//             Some(data) => view! {
//                 <ul>
//                     { data.unwrap().iter().map(move |user| view! {<li>{user.user_name.clone()}</li>}).collect_view() }
//                 </ul>
//             }.into_view()
//         }}
//     }
// }

#[component]
pub fn ChatPage() -> impl IntoView {
    let (history, set_history) = create_signal(vec![]);
    let update_history = move |message: String| set_history.update(|history| history.push(message));

    let ws_addr = "ws://localhost:4321/ws";

    let UseWebsocketReturn {
        ready_state,
        message,
        send,
        open,
        ..
    } = use_websocket(ws_addr);

    let input_ref = create_node_ref::<html::Input>();

    let send_message = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let text = input_ref
            .get()
            .expect("input element doesn't exist")
            .value();
        send(text.as_str());
        set_history.update(|history: &mut Vec<_>| history.push(format! {"[send]: {:?}", text}));
        input_ref
            .get()
            .expect("input element doesn't exist")
            .set_value("");
    };

    let status = move || ready_state().to_string();

    let open_connection = move |_: ev::Event| {
        open();
        logging::log!("connection is open");
    };

    create_effect(move |_| {
        if let Some(msg) = message.get() {
            update_history(format!("[message]: {:?}", msg));
        };
    });

    view! {
        <div on:load=open_connection class="size-11/12 flex flex-row mx-4 my-4 bg-slate-800/[.65] rounded-xl">
            <div class="h-full w-[70px] bg-slate-950/[.65] rounded-l-xl" id="main-navigation"></div>
            <div class="h-full w-[300px] bg-transparent rounded-l-xl flex flex-col">
                <div class="h-20 w-full bg-transparent">
                    <p class="text-green-400 font-sans">{ status }</p>
                    <LogoutButton/>
                </div>
                <div class="grow w-full bg-slate-800/[.65] rounded-bl-xl">
                    // <OnlineList/>
                </div>
            </div>
            <div class="h-full bg-transparent grow flex flex-col" id="chat">
                <div class="grow bg-transparent px-4" id="chat-log">
                    <For
                        each=move || history.get().into_iter().enumerate()
                        key=|(index, _)| *index
                        let:msg
                    >
                        <div class="bg-slate-800/[.65] rounded-lg w-fit px-2 my-px">
                            <p class="text-white font-sans">{ msg.1 }</p>
                        </div>
                    </For>
                </div>
                <form on:submit=send_message class="bg-transparent px-4 h-32 flex flex-row items-center">
                    <input id="input"
                        _ref=input_ref
                        class="grow rounded-l-full h-12 text-white font-sans pl-2 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto text-base"
                    />
                    <button class="border-none h-12 w-fit px-2 bg-sky-500 hover:bg-green-300 hover:text-black rounded-r-full border-0 text-white font-sans">
                        <strong>"SEND"</strong>
                    </button>
                </form>
            </div>
        </div>
    }
}
