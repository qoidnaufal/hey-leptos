use leptos::*;

#[component]
pub fn ChannelHeader(channel_name: String) -> impl IntoView {
    view! {
        <div class="select-none shrink-0 bg-slate-800/[.65] w-auto h-[50px] flex flex-row pl-2 items-center rounded-tr-xl">
            <p class="font-sans tracking-wider text-white">"Current channel: "{channel_name}</p>
        </div>
    }
}
