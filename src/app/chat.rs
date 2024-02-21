use crate::app::logout::LogoutButton;
use leptos::*;
use leptos_use::{use_websocket, UseWebsocketReturn};

#[component]
pub fn ChatPage() -> impl IntoView {
    let (history, set_history) = create_signal(vec![]);
    let update_history = move |message: String| set_history.update(|history| history.push(message));

    // let (url, set_url) = create_signal(String::new());
    // let update_url = move |ws_addr: String| set_url.update(|url| url.push_str(&ws_addr));

    // create_effect(move |_| {
    //     let ws_addr = format!("ws://{}/ws", window().location().host().unwrap());
    //     update_url(ws_addr);
    // });

    // logging::log!("{:?}", url.get_untracked());

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
                <div class="grow w-full bg-slate-800/[.65] rounded-bl-xl"></div>
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
