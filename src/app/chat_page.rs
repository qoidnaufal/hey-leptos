use leptos::{component, view, IntoView};

#[component]
pub fn ChatPage() -> impl IntoView {
    // Creates a reactive value to update the button
    view! {
        <div class="size-11/12 flex flex-row mx-4 my-4 bg-slate-800/[.65] rounded-xl">
            <div class="h-full w-80 bg-slate-800/[.65] rounded-l-xl" id="left-navigation"></div>
            <div class="h-full bg-transparent grow flex flex-col" id="chat">
                <div class="grow bg-transparent px-4" id="chat-log"></div>
                <form class="bg-transparent px-4 h-32 flex flex-row items-center">
                    <input class="grow rounded-l-xl h-12 text-white pl-2 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto text-base"/>
                    <button class="border-none h-12 w-fit px-2 bg-sky-500 hover:bg-green-300 hover:text-black rounded-r-xl border-0 text-white">
                        "Send"
                    </button>
                </form>
            </div>
        </div>
    }
}
