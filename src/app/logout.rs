use leptos::*;
use leptos_router::ActionForm;

pub type LogoutAction = Action<UserLogout, Result<(), ServerFnError>>;

#[server(UserLogout)]
pub async fn logout() -> Result<(), ServerFnError> {
    use super::AppPath;
    use crate::state::ssr::auth;

    let auth = auth()?;
    // let uuid = auth.current_user.clone().expect("There's no user!").uuid;

    auth.logout_user();
    // auth.cache_clear_user(uuid);
    leptos_axum::redirect(&AppPath::Login.to_string());

    Ok(())
}

#[component]
pub fn LogoutButton(logout_action: LogoutAction) -> impl IntoView {
    // let logout_action = create_server_action::<UserLogout>();
    view! {
        <ActionForm action=logout_action>
            <button class="cursor-pointer font-sans text-white text-right hover:text-green-300 hover:bg-slate-600/[.75] border-none h-9 w-full bg-transparent pr-2">
                "log out"
            </button>
        </ActionForm>
    }
}
