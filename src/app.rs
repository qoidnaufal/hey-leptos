use crate::error_template::{AppError, ErrorTemplate};
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

pub enum AppPath {
    Register,
    Login,
    Logout,
    Home,
    Channel(Option<String>),
}

impl std::fmt::Display for AppPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register => write!(f, "register"),
            Self::Login => write!(f, "login"),
            Self::Logout => write!(f, "logout"),
            Self::Home => write!(f, ""),
            Self::Channel(id) => match id {
                Some(id) => write!(f, "channel/{}", id),
                None => write!(f, "channel"),
            },
        }
    }
}

impl leptos_router::ToHref for AppPath {
    fn to_href(&self) -> Box<dyn Fn() -> String + '_> {
        match self {
            Self::Register => Box::new(|| Self::Register.to_string()),
            Self::Login => Box::new(|| Self::Login.to_string()),
            Self::Logout => Box::new(|| Self::Logout.to_string()),
            Self::Home => Box::new(|| Self::Home.to_string()),
            Self::Channel(id) => Box::new(|| Self::Channel(id.clone()).to_string()),
        }
    }
}

#[server(AuthenticateUser)]
async fn authenticate_user() -> Result<(), ServerFnError> {
    use crate::{
        models::user_model::UserData,
        state::ssr::{auth, pool},
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

        Ok(())
    } else {
        Err(ServerFnError::new("Auth session isn't authenticated!"))
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
                    <Route
                        path=AppPath::Home
                        view=move || view! {
                            <Transition fallback=|| view! { <p>"Loading..."</p> }>
                                {move || auth_resource.map(|res| match res.clone() {
                                    Ok(_) => view! { <Redirect path=AppPath::Channel(None)/> },
                                    Err(_) => view! { <home::HomePage/> }
                                })}
                            </Transition>
                        }
                    />
                    // TODO: create a guarding mechanism here
                    <Route
                        path=AppPath::Channel(None)
                        view=move || view! { <chat::ChatPage logout_action/> }
                    >
                        <Route path=":id" view=chat::Channel/>
                        <Route path="" view=|| view! {
                            <div class="h-full bg-transparent grow flex items-center justify-center">
                                <p class="font-sans text-white text-center">"TODO: create a landing page"</p>
                            </div>
                        }/>
                    </Route>
                    <Route path=AppPath::Register view=register::RegisterPage/>
                    <Route path=AppPath::Login view=move || view! { <login::LoginPage login_action/> }/>
                </Routes>
            </main>
        </Router>
    }
}
