use leptos::{
    component, create_server_action, ev::SubmitEvent, server, view, IntoView, ServerFnError,
};
use leptos_router::{ActionForm, FromFormData};

#[server(RegisterUser)]
async fn register(user_name: String, email: String, password: String) -> Result<(), ServerFnError> {
    use crate::auth_model::ssr::db;
    use crate::auth_model::UserData;
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand_core::OsRng;
    use uuid::Uuid;

    let db = db()?;

    let salt = SaltString::generate(&mut OsRng);
    let password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| ServerFnError::<std::io::Error>::ServerError(format!("{:?}", err)))
        .map(|pw| pw.to_string())?;

    let uuid = Uuid::new_v4().as_simple().to_string();
    let new_user = UserData::new(uuid.clone(), user_name.clone(), email, password);

    match db.register_user(uuid, new_user).await {
        Ok(_) => {
            leptos_axum::redirect("/login");
            Ok(())
        }
        Err(err) => Err(ServerFnError::ServerError(format!("{:?}", err))),
    }
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register_user = create_server_action::<RegisterUser>();

    let on_submit = move |ev: SubmitEvent| {
        let data = RegisterUser::from_event(&ev);
        if data.is_err() || !data.unwrap().password.contains("@") {
            ev.prevent_default();
        }
    };

    view! {
        <div id="registerpage" class="flex-col bg-slate-800/[.65] py-2.5 px-8 rounded-xl size-96">
            <h1 class="mt-5 text-white text-center text-xl">"Register"</h1>
            <ActionForm action=register_user class="flex flex-col">
                <input
                    class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-7 text-base h-10"
                    placeholder="Your name..."
                    required
                    type="text"
                    name="user_name"
                />
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
                <button
                    type="submit"
                    on:submit=on_submit
                    class="mt-5 w-full bg-sky-500 hover:bg-green-300 rounded-lg border-0 w-fit py-1 px-1"
                >
                    "Register"
                </button>
            </ActionForm>
            <p class="text-white text-center mt-2" id="switch">
                "Already have an account?"
                <a class="text-indigo-400" href="/login">
                    " Login "
                </a>
                "now!"
            </p>
        </div>
    }
}
