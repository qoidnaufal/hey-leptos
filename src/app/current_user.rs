use super::logout::LogoutButton;
use crate::models::user_model::Avatar;
use leptos::*;

#[server(GetAvatarAndName, "/api", "GetJson")]
async fn get_avatar_and_name() -> Result<(Avatar, String), ServerFnError> {
    use crate::models::user_model::UserData;
    use crate::state::{auth, pool};

    let auth = auth()?;
    let pool = pool()?;

    let current_user = auth
        .current_user
        .ok_or_else(|| ServerFnError::new("Auth does not contain user"))?;

    let user_data = UserData::get_from_uuid(&current_user.uuid, &pool)
        .await
        .ok_or_else(|| ServerFnError::new("User does not exist"))?;

    // std::thread::sleep(std::time::Duration::from_millis(1500));

    Ok((user_data.avatar.clone(), user_data.user_name))
}

#[component]
pub fn CurrentUser(
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

    let user_resource = create_resource(|| (), |_| get_avatar_and_name());

    view! {
        <div class="select-none bg-slate-900/[.65] w-auto h-[50px] flex flex-row pl-2 items-center rounded-tl-xl">
            <Transition
                fallback=|| view! {
                    <div class="flex justify-center items-center pb-1 h-9 w-9 bg-sky-500 rounded-full text-white hover:text-black uppercase font-sans text-2xl text-center">
                    </div>
                    <div class="font-sans text-white pl-2 grow bg-slate-300/[.65]">
                    </div>
                }
            >
                {move || user_resource.map(|result| match result.clone() {
                    Ok((avatar, user_name)) => view! {
                        <div class="flex justify-center items-center pb-1 h-9 w-9 bg-sky-500 rounded-full text-white hover:text-black hover:bg-green-300 uppercase font-sans text-2xl text-center">
                            { avatar.get_view() }
                        </div>
                        <div class="font-sans text-white pl-2 grow bg-transparent">
                            { user_name }
                        </div>
                    }.into_view(),
                    Err(_) => view! { <p>"Error loading user information"</p> }.into_view()
                })}
            </Transition>
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
