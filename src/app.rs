use crate::{
    error_template::{AppError, ErrorTemplate},
    user_model::User,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod chat;
mod create_or_join;
mod current_user;
mod home;
mod joined_channels;
mod login;
mod logout;
mod register;
mod users_list;

#[server(AuthenticateUser)]
async fn authenticate_user() -> Result<User, ServerFnError> {
    use crate::{
        state::{auth, pool},
        user_model::UserData,
    };

    let auth = auth()?;
    let pool = pool()?;

    if auth.is_authenticated() {
        let user = auth
            .current_user
            .clone()
            .ok_or_else(|| ServerFnError::new("There is no current user!"))?;

        if UserData::get_from_uuid(&user.uuid, &pool).await.is_none() {
            return Err(ServerFnError::new("Invalid user"));
        }

        auth.login_user(user.uuid.clone());
        auth.remember_user(true);

        Ok(user)
    } else {
        Err(ServerFnError::new("Auth session isn't authenticated!"))
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/hey-leptos.css"/>
        <Title text="HEY!"/>
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main class="grid h-screen place-items-center bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500">
                <Routes>
                    <Route path="/" view=home::HomePage/>
                    <Route path="/channel" view=chat::ChatPage>
                        <Route path=":id" view=chat::Channel/>
                        <Route path="" view=|| view! {
                            <div class="h-full bg-transparent grow flex items-center justify-center">
                                <p class="font-sans text-white text-center">"TODO: create a landing page"</p>
                            </div>
                        }/>
                    </Route>
                    <Route path="/register" view=register::RegisterPage/>
                    <Route path="/login" view=login::LoginPage/>
                </Routes>
            </main>
        </Router>
    }
}
