use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod chat;
mod home;
mod login;
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
            future=|| authenticate()
            children=|session| {
                if let Ok(true) = session.clone().map(|v| v.clone()) {
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
    use crate::auth_model::ssr::{auth, db};

    let auth = auth()?;
    let db = db()?;

    if auth.is_authenticated() {
        match &auth.current_user {
            Some(user) => {
                if db.get_user_by_id(&user.uuid).await.is_some() {
                    auth.login_user(user.uuid.clone());
                    auth.remember_user(true);

                    Ok(true)
                } else {
                    Err(ServerFnError::new("Invalid auth session!"))
                }
            }
            None => Err(ServerFnError::new("Invalid auth session!")),
        }
    } else {
        Err(ServerFnError::new("Auth session isn't authenticated!"))
    }
}
