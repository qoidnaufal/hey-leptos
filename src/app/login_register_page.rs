use leptos::{component, create_server_action, server, view, Action, IntoView, ServerFnError};
use leptos_router::ActionForm;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_name: String,
    pub email: String,
    pub password: String,
}

#[server(RegisterUser)]
pub async fn register(user: User) -> Result<(), ServerFnError> {
    Ok(println!("user: {:?}", user))
}

#[server(UserLogin)]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    Ok(println!("email: {}, password: {}", email, password))
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register_user = Action::<RegisterUser, _>::server();

    view! {
        <div id="registerpage" class="flex-col bg-slate-800/[.65] py-2.5 px-8 rounded-xl size-96">
            <h1 class="mt-5 text-white text-center text-xl">Register</h1>
                <ActionForm action=register_user class="flex flex-col">
                    <input
                        class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-7 text-base h-10"
                        placeholder="Your name..."
                        required
                        type="text"
                        name="user[user_name]"
                    />
                    <input
                        class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-7 text-base h-10"
                        placeholder="Your email..."
                        required
                        type="email"
                        name="user[email]"
                    />
                    <input
                        class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-7 text-base h-10"
                        placeholder="Your password..."
                        required
                        type="password"
                        name="user[password]"
                    />
                    <button type="submit" class="mt-5 w-full bg-sky-500 hover:bg-green-300 rounded-lg border-0 w-fit py-1 px-1">
                        "Register"
                    </button>
                </ActionForm>
            <p class="text-white text-center mt-2" id="switch">
                "Already have an account?"
                <a class="text-indigo-400" href="/log">
                    " Login "
                </a>
                "now!"
            </p>
        </div>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let login = create_server_action::<UserLogin>();
    // let _value = login.value();

    view! {
        <div
            id="loginpage"
            class="flex-col content-center bg-slate-800/[.65] py-2.5 px-8 rounded-xl size-96"
        >
            <h1 class="mt-5 text-white text-center text-xl">Register</h1>
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
                <a class="text-indigo-400" href="/reg">
                    " Register "
                </a>
                "now!"
            </p>
        </div>
    }
}
