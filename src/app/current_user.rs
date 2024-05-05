use super::logout::{LogoutAction, LogoutButton};
use crate::models::user_model::User;
use leptos::*;
use leptos_router::A;

#[server(GetAvatarAndName, "/api", "GetJson")]
pub async fn get_avatar_and_name() -> Result<User, ServerFnError> {
    use crate::state::auth;

    let auth = auth()?;
    let current_user = auth
        .current_user
        .ok_or_else(|| ServerFnError::new("Auth does not contain user"))?;
    Ok(current_user)
}

#[component]
pub fn CurrentUser(
    display_user_menu: ReadSignal<bool>,
    set_display_user_menu: WriteSignal<bool>,
    user_resource: Resource<(), Result<User, ServerFnError>>,
) -> impl IntoView {
    let toggle_user_menu = move |_| {
        if !display_user_menu.get() {
            set_display_user_menu.set(true)
        } else {
            set_display_user_menu.set(false)
        }
    };

    view! {
        <div class="select-none bg-slate-900/[.65] w-auto h-[50px] flex flex-row pl-2 items-center rounded-tl-xl">
            <Transition
                {move || user_resource.track()}
                fallback=|| view! {
                    <div class="pb-1 h-9 w-9 bg-slate-500 rounded-full">
                    </div>
                    <div class="pl-2 grow bg-slate-300/[.65]">
                    </div>
                }
            >
                {move || user_resource.map(|result| match result.clone() {
                    Ok(current_user) => view! {
                        <A href="">
                            <div class="flex justify-center items-center pb-1 h-9 w-9 bg-sky-500 rounded-full text-white hover:text-black hover:bg-green-300 uppercase font-sans text-2xl text-center">
                                { current_user.avatar.get_view() }
                            </div>
                        </A>
                        <div class="font-sans text-white pl-2 grow bg-transparent">
                            { current_user.user_name }
                        </div>
                    }.into_view(),
                    Err(err) => view! { <div class="grow pl-2 text-white font-sans bg-transparent">{ err.to_string() }</div> }.into_view()
                })}
            </Transition>
            <button
                on:click=toggle_user_menu
                class="mr-2 text-white text-xl cursor-pointer rounded-md bg-transparent hover:bg-slate-600/[.75] w-7 h-7 border-none"
            >
                "⚙"
            </button>
        </div>
    }
}

#[component]
pub fn UserMenu(display_user_menu: ReadSignal<bool>) -> impl IntoView {
    let logout_action = expect_context::<LogoutAction>();

    view! {
        <Show when=move || display_user_menu.get()>
            <div class="block relative flex flex-col bg-slate-900 select-none left-[120px] w-[250px]">
                <LogoutButton logout_action/>
                <LogoutButton logout_action/>
                <LogoutButton logout_action/>
            </div>
        </Show>
    }
}
