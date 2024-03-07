use leptos::*;
use leptos_router::ActionForm;

#[server(UserLogout)]
async fn logout() -> Result<(), ServerFnError> {
    use crate::state::auth;

    let auth = auth()?;
    let uuid = auth.current_user.clone().expect("There's no user!").uuid;

    auth.logout_user();
    auth.cache_clear_user(uuid);
    leptos_axum::redirect("/login");

    Ok(())
}

#[component]
pub fn LogoutButton() -> impl IntoView {
    let logout = create_server_action::<UserLogout>();
    view! {
        <ActionForm action=logout>
            <button class="font-sans text-white hover:text-black border-none h-12 w-fit bg-transparent">
                "Log out"
            </button>
        </ActionForm>
    }
}
