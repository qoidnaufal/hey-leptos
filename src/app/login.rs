use super::AppPath;
use leptos::*;
use leptos_router::{ActionForm, A};

type LoginAction = Action<UserLogin, Result<(), ServerFnError>>;

#[server(UserLogin)]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use super::AppPath;
    use crate::models::user_model::UserData;
    use crate::state::ssr::{auth, pool};
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let pool = pool()?;
    let auth = auth()?;

    let user = UserData::get_from_email(&email, &pool)
        .await?
        .expect("User doesn't exist");

    let parsed_password =
        PasswordHash::new(&user.password).map_err(|err| ServerFnError::new(format!("{}", err)))?;

    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed_password)
        .is_ok()
    {
        auth.login_user(user.uuid);
        auth.remember_user(true);
        leptos_axum::redirect(&AppPath::Channel(None).to_string());

        Ok(())
    } else {
        Err(ServerFnError::new("Password does not match".to_string()))
    }
}

#[component]
pub fn LoginPage(login_action: LoginAction) -> impl IntoView {
    // let login_action = create_server_action::<UserLogin>();

    view! {
        <div
            id="loginpage"
            class="flex-col content-center bg-slate-800/[.65] py-2.5 px-8 rounded-xl size-96"
        >
            <h1 class="mt-5 text-white text-center text-xl">"Login"</h1>
            <ActionForm action=login_action class="flex flex-col">
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
                "Want to create a new account?"
                <A class="text-indigo-400" href=AppPath::Register>
                    " Register "
                </A>
                "now!"
            </p>
        </div>
    }
}
