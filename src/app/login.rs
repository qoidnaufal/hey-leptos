use super::AppPath;
use leptos::*;
use leptos_router::{ActionForm, FromFormData, A};

type LoginAction = Action<UserLogin, Result<(), ServerFnError>>;

#[server(UserLogin)]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use super::AppPath;
    use crate::models::user_model::UserData;
    use crate::state::{auth, pool};
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let pool = pool()?;
    let auth = auth()?;

    let user = UserData::get_from_email(&email, &pool)
        .await?
        .ok_or_else(|| ServerFnError::new("User does not exist"))?;

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
    let (button_text, set_button_text) = create_signal("Login");

    let validate = move |ev: ev::SubmitEvent| {
        let data = UserLogin::from_event(&ev);
        if data.is_err() {
            ev.prevent_default();
        }

        set_button_text.set("Loading...");
    };

    view! {
        <div
            id="loginpage"
            class="block absolute m-auto left-0 right-0 top-0 bottom-0 flex flex-col bg-slate-800/[.65] py-2.5 px-8 rounded-xl size-[27rem]"
        >
            <h1 class="mt-5 text-white text-center text-xl">"Login"</h1>
            <ActionForm
                action=login_action
                on:submit=validate
                class="flex flex-col"
            >
                <label class="font-sans text-white mt-3 flex flex-col">
                    "Email:"
                    <input
                        class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-2 text-base h-10"
                        placeholder="Your email..."
                        required
                        type="email"
                        name="email"
                    />
                </label>
                <label class="font-sans text-white mt-3 flex flex-col">
                    "Password:"
                    <input
                        class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-2 text-base h-10"
                        placeholder="Your password..."
                        required
                        type="password"
                        name="password"
                    />
                </label>
                <ErrorBoundary
                    fallback=move |_| {
                        if button_text.get() == "Loading..." { set_button_text.set("Login"); }
                        view! {
                            <div class="mt-3 bg-slate-50/[.85] w-full flex items-center justify-center">
                                <p class="font-sans text-red-600">"Error: email or password doesn't match"</p>
                            </div>
                        }
                    }
                >
                    {move || login_action.value().get()}
                </ErrorBoundary>
                <button class="mt-3 w-full bg-sky-500 hover:bg-green-300 rounded-lg border-0 py-1 px-1">
                    {move || button_text.get()}
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
