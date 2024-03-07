use crate::{
    error_template::{AppError, ErrorTemplate},
    user_model::{User, UserData},
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod chat;
mod create_or_join;
mod current_user;
mod home;
mod login;
mod logout;
mod register;
mod users_list;

#[derive(Clone)]
pub struct CtxProvider {
    pub user: User,
}

impl CtxProvider {
    pub fn new(user: User) -> Self {
        Self { user }
    }
}

#[server]
async fn authenticate_user() -> Result<UserData, ServerFnError> {
    use crate::state::{auth, pool};

    let auth = auth()?;
    let pool = pool()?;

    if auth.is_authenticated() {
        let user = auth
            .current_user
            .clone()
            .ok_or_else(|| ServerFnError::new("There is no current user!"))?;

        let user_data = UserData::get_from_uuid(&user.uuid, &pool)
            .await
            .ok_or_else(|| ServerFnError::new("User does not exist"))?;

        auth.login_user(user_data.uuid.clone());
        auth.remember_user(true);

        Ok(user_data)
    } else {
        Err(ServerFnError::new("Auth session isn't authenticated!"))
    }
}

#[server]
async fn validate_path() -> Result<Option<String>, ServerFnError> {
    use crate::state::rooms_manager;
    use axum::extract::Path;

    let Path(id) = leptos_axum::extract::<Path<String>>().await?;
    logging::log!("extracted id: {}", id);

    let rooms_manager = rooms_manager()?;

    if !id.contains("validate") {
        rooms_manager
            .validate_uuid(id.clone())
            .map_err(|err| ServerFnError::new(format!("{:?}", err)))
    } else {
        Ok(None)
    }
}

#[component]
fn HomeOrChat() -> impl IntoView {
    view! {
        <Await
            future=authenticate_user
            children=|auth| {
                if let Ok(user_data) = auth.clone() {
                    let user = User::from_user_data(&user_data);
                    provide_context(CtxProvider::new(user));

                    let navigate = leptos_router::use_navigate();
                    create_effect(move |_| {
                        navigate("/channel", Default::default());
                    });

                    view! { <chat::ChatPage/> }
                } else {
                    let navigate = leptos_router::use_navigate();
                    create_effect(move |_| {
                        navigate("/", Default::default());
                    });
                    view! { <home::HomePage/> }
                }
            }
        />
    }
}

#[component]
fn ViewChannel() -> impl IntoView {
    let mut outside_errors = Errors::default();
    outside_errors.insert_with_default_key(AppError::NotFound);

    view! {
        <Await
            future=validate_path
            children=move |result| {
                let outside_errors = outside_errors.clone();
                if let Ok(Some(uuid)) = result.clone() {
                    let navigate = leptos_router::use_navigate();
                    create_effect(move |_| {
                        navigate(&uuid, Default::default());
                    });
                    view! { <chat::Channel/> }
                } else if let Ok(None) = result.clone() {
                    view! { <chat::Channel/> }
                } else {
                    view! { <ErrorTemplate outside_errors/> }
                }
            }
        />
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
                    <Route path="/" view=HomeOrChat/>
                    <Route path="/channel" view=HomeOrChat>
                        <Route path=":id" view=ViewChannel/>
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
