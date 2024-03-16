use leptos::*;
use leptos_router::ActionForm;

#[server(UserLogout)]
pub async fn logout() -> Result<(), ServerFnError> {
    use super::MyPath;
    use crate::state::auth;

    let auth = auth()?;
    let uuid = auth.current_user.clone().expect("There's no user!").uuid;

    auth.logout_user();
    auth.cache_clear_user(uuid);
    leptos_axum::redirect(&MyPath::Login.to_string());

    Ok(())
}

#[component]
pub fn LogoutButton() -> impl IntoView {
    let logout_action = create_server_action::<UserLogout>();
    view! {
        <ActionForm action=logout_action>
            <button class="cursor-pointer font-sans text-white text-right hover:text-green-300 hover:bg-slate-600/[.75] border-none h-9 w-full bg-transparent pr-2">
                "log out"
            </button>
        </ActionForm>
    }
}
