use crate::error::AppError;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod app_error;
mod channel;
mod channel_header;
mod chat;
mod create_or_join;
mod current_user;
mod home;
mod joined_channels;
mod login;
mod logout;
mod register;

pub enum AppPath {
    Register,
    Login,
    Logout,
    Home,
    Profile(String),
    Channel(Option<String>),
}

impl std::fmt::Display for AppPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register => write!(f, "/register"),
            Self::Login => write!(f, "/login"),
            Self::Logout => write!(f, "logout"),
            Self::Home => write!(f, "/"),
            Self::Profile(id) => write!(f, "profile/{}", id),
            Self::Channel(id) => match id {
                Some(id) => write!(f, "channel/{}", id),
                None => write!(f, "channel"),
            },
        }
    }
}

impl leptos_router::ToHref for AppPath {
    fn to_href(&self) -> Box<dyn Fn() -> String + '_> {
        Box::new(|| self.to_string())
    }
}

#[server(AuthenticateUser)]
async fn authenticate_user() -> Result<bool, ServerFnError> {
    use crate::{
        models::user_model::UserData,
        state::{auth, pool},
    };

    let auth = auth()?;
    let pool = pool()?;
    if auth.is_authenticated() {
        let user = auth
            .current_user
            .ok_or_else(|| ServerFnError::new("There is no current user!"))?;
        if UserData::get_from_uuid(&user.uuid, &pool).await.is_none() {
            return Err(ServerFnError::new("Invalid user"));
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let login_action = create_server_action::<login::UserLogin>();
    let logout_action = create_server_action::<logout::UserLogout>();
    let auth_resource = create_resource(
        move || (login_action.version().get(), logout_action.version().get()),
        |_| authenticate_user(),
    );
    let (is_auth, set_is_auth) = create_signal(false);

    view! {
        <Stylesheet id="leptos" href="/pkg/hey-leptos.css"/>
        <Title text="HEY!"/>
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <app_error::ErrorTemplate outside_errors/> }.into_view()
        }>
            <main class="absolute size-full bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500">
                <Routes>
                    <Route
                        path=AppPath::Home
                        view=move || view! {
                            <Transition fallback=|| view! { <p>"Loading..."</p> }>
                                {move || auth_resource.map(|res| match *res {
                                    Ok(val) => {
                                        if is_auth.get() != val {
                                            set_is_auth.set(val);
                                        }
                                        match val {
                                            true => view! { <Redirect path=AppPath::Channel(None)/> },
                                            false => view! { <home::HomePage/> }
                                        }
                                    },
                                    Err(_) => view! { <home::HomePage/> }
                                })}
                            </Transition>
                        }
                    />
                    <ProtectedRoute
                        path=AppPath::Channel(None)
                        redirect_path=AppPath::Home
                        condition=move || is_auth.get()
                        view=move || view! { <chat::ChatPage logout_action/> }
                    >
                        <Route path=":id" view=channel::Channel/>
                        <Route path="" view=|| view! {
                            <div class="h-full grow flex items-center justify-center">
                                <p class="font-sans text-white text-center">"TODO: create a landing page"</p>
                            </div>
                        }/>
                    </ProtectedRoute>
                    <Route path=AppPath::Register view=register::RegisterPage/>
                    <Route path=AppPath::Login view=move || view! { <login::LoginPage login_action/> }/>
                </Routes>
            </main>
        </Router>
    }
}
