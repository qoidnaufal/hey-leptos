use super::AppPath;
use leptos::*;
use leptos_router::{ActionForm, FromFormData, A};

#[server(RegisterUser)]
async fn register(user_name: String, email: String, password: String) -> Result<(), ServerFnError> {
    use super::AppPath;
    use crate::{models::user_model::UserData, state::ssr::pool};
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand_core::OsRng;
    use uuid::Uuid;

    let pool = pool()?;

    // std::thread::sleep(std::time::Duration::from_millis(2000));

    let salt = SaltString::generate(&mut OsRng);
    let password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| ServerFnError::new(format!("{:?}", err)))
        .map(|pw| pw.to_string())?;

    let uuid = Uuid::new_v4().as_simple().to_string();
    let new_user = UserData::new(uuid, user_name, email.clone(), password);

    match new_user.insert_into_db(&pool).await {
        Ok(_) => {
            leptos_axum::redirect(&AppPath::Login.to_string());
            Ok(())
        }
        Err(err) => Err(ServerFnError::new(format!("{:?}", err))),
    }
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register_user = create_server_action::<RegisterUser>();

    let (button_text, set_button_text) = create_signal("Register");

    let validate = move |ev: ev::SubmitEvent| {
        let data = RegisterUser::from_event(&ev);
        if data.is_err() {
            ev.prevent_default();
        }

        set_button_text.set("Loading...");
    };

    view! {
        <div id="registerpage" class="flex-col bg-slate-800/[.65] py-2.5 px-8 rounded-xl size-[27rem]">
            <h1 class="mt-5 text-white text-center text-xl">"Register"</h1>
            <ActionForm
                action=register_user
                on:submit=validate
                class="flex flex-col"
            >
                <label class="font-sans text-white flex flex-col mt-3">
                    "User Name:"
                    <input
                        class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-2 text-base h-10"
                        placeholder="Your name..."
                        required
                        type="text"
                        name="user_name"
                    />
                </label>
                <label class="font-sans text-white flex flex-col mt-3">
                    "Email:"
                    <input
                        class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-2 text-base h-10"
                        placeholder="Your email..."
                        required
                        type="email"
                        name="email"
                    />
                </label>
                <label class="font-sans text-white flex flex-col mt-3">
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
                        if button_text.get() == "Loading..." { set_button_text.set("Register"); }
                        view! {
                            <div class="mt-3 bg-slate-50/[.85] w-full flex items-center justify-center">
                                <p class="font-sans text-red-600">"Error: email has been taken"</p>
                            </div>
                        }
                    }
                >
                    {move || register_user.value().get()}
                </ErrorBoundary>
                <button
                    type="submit"
                    class="mt-3 w-full bg-sky-500 hover:bg-green-300 rounded-lg border-0 w-fit py-1 px-1"
                >
                    {move || button_text.get()}
                </button>
            </ActionForm>
            <p class="text-white text-center mt-2" id="switch">
                "Already have an account?"
                <A class="text-indigo-400" href=AppPath::Login>
                    " Login "
                </A>
                "now!"
            </p>
        </div>

    }
}
