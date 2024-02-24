use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod chat;
mod home;
mod login;
mod logout;
mod register;

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
                    <Route path="/" view=HomeOrChat/>
                    <Route path="/register" view=register::RegisterPage/>
                    <Route path="/login" view=login::LoginPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomeOrChat() -> impl IntoView {
    view! {
        <Await
            future=authenticate
            children=|is_auth| {
                if let Ok(true) = is_auth {
                    view! { <chat::ChatPage/> }
                } else {
                    view! { <home::HomePage/> }
                }
            }
        />
    }
}

#[server]
async fn authenticate() -> Result<bool, ServerFnError> {
    use crate::auth_model::{auth, pool};
    use crate::user_model::UserData;

    let auth = auth()?;
    let pool = pool()?;

    if auth.is_authenticated() {
        match &auth.current_user {
            Some(user) => {
                if UserData::get_from_id(&user.uuid, &pool).await.is_some() {
                    auth.login_user(user.uuid.clone());
                    auth.remember_user(true);

                    Ok(true)
                } else {
                    Err(ServerFnError::new("Invalid auth session!"))
                }
            }
            None => Err(ServerFnError::new("Auth session contains no user!")),
        }
    } else {
        Err(ServerFnError::new("Auth session isn't authenticated!"))
    }
}
