use leptos::{component, create_server_action, server, view, IntoView, ServerFnError};
use leptos_router::ActionForm;

#[server(UserLogin)]
async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use crate::auth_model::ssr::{auth, db};
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use leptos::logging;

    let db = db()?;
    let auth = auth()?;

    let user = db
        .get_user_by_email(&email)
        .await?
        .ok_or_else(|| ServerFnError::new("User does not exist".to_string()))?;

    let parsed_password =
        PasswordHash::new(&user.password).map_err(|err| ServerFnError::new(format!("{}", err)))?;

    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed_password)
        .is_ok()
    {
        auth.login_user(user.uuid);
        auth.remember_user(true);
        leptos_axum::redirect("/");

        Ok(())
    } else {
        logging::log!("User auth failed: Mismatched email & password.");
        Err(ServerFnError::new("Password does not match".to_string()))
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let login = create_server_action::<UserLogin>();

    view! {
        <div
            id="loginpage"
            class="flex-col content-center bg-slate-800/[.65] py-2.5 px-8 rounded-xl size-96"
        >
            <h1 class="mt-5 text-white text-center text-xl">"Login"</h1>
            <ActionForm action=login class="flex flex-col">
                <input
                    class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-7 text-base h-10"
                    placeholder="Your email..."
                    required
                    type="email"
                    name="email"
                />
                <input
                    class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-7 text-base h-10"
                    placeholder="Your password..."
                    required
                    type="password"
                    name="password"
                />
                <button class="mt-5 w-full bg-sky-500 hover:bg-green-300 rounded-lg border-0 w-fit py-1 px-1">
                    "Login"
                </button>
            </ActionForm>
            <p class="text-white text-center mt-2" id="switch">
                "Don't have an account?"
                <a class="text-indigo-400" href="/register">
                    " Register "
                </a>
                "now!"
            </p>
        </div>
    }
}
