use crate::{app::logout::LogoutButton, user_model::User};
use leptos::*;

#[component]
pub fn CurrentUser(
    user: Memo<User>,
    display_user_menu: ReadSignal<&'static str>,
    set_display_user_menu: WriteSignal<&'static str>,
) -> impl IntoView {
    let toggle_user_menu = move |_| {
        if display_user_menu.get() == "hidden" {
            set_display_user_menu
                .set("block relative flex flex-col bg-slate-900 select-none left-[120px] w-[250px]")
        } else {
            set_display_user_menu.set("hidden")
        }
    };

    view! {
        <div class="select-none bg-slate-900/[.65] w-auto h-[50px] flex flex-row pl-2 items-center rounded-tl-xl">
            <div class="flex justify-center items-center pb-1 h-9 w-9 bg-sky-500 rounded-full text-white hover:text-black uppercase font-sans text-2xl text-center">
                { move || user.get().avatar.get_view() }
            </div>
            <div class="font-sans text-white pl-2 grow bg-transparent">
                { move || user.get().user_name }
            </div>
            <button on:click=toggle_user_menu class="mr-2 text-white text-xl cursor-pointer rounded-md bg-transparent hover:bg-slate-600/[.75] w-7 h-7 border-none">
                "⚙"
            </button>
        </div>
    }
}

#[component]
pub fn UserMenu(display_user_menu: ReadSignal<&'static str>) -> impl IntoView {
    view! {
        <div class=move || display_user_menu.get()>
            <LogoutButton/>
            <LogoutButton/>
            <LogoutButton/>
        </div>
    }
}
