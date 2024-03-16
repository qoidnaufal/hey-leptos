use crate::{
    error_template::{AppError, ErrorTemplate},
    models::user_model::User,
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

pub enum MyPath {
    Register,
    Login,
    Logout,
    Home,
    Channel(Option<String>),
}

impl std::fmt::Display for MyPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register => write!(f, "/register"),
            Self::Login => write!(f, "/login"),
            Self::Logout => write!(f, "/logout"),
            Self::Home => write!(f, "/"),
            Self::Channel(id) => match id {
                Some(id) => write!(f, "/channel/{}", id),
                None => write!(f, "/channel"),
            },
        }
    }
}

impl leptos_router::ToHref for MyPath {
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
async fn authenticate_user() -> Result<User, ServerFnError> {
    use crate::{
        models::user_model::UserData,
        state::{auth, pool},
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

    let resource = create_resource(|| (), |_| authenticate_user());

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
                        path=MyPath::Home
                        view=move || view! {
                            <Transition fallback=|| view! { <p>"Loading..."</p> }>
                                {move || resource.map(|res| match res.clone() {
                                    Ok(_) => view! { <Redirect path=MyPath::Channel(None)/> },
                                    Err(_) => view! { <home::HomePage/> }
                                })}
                            </Transition>
                        }
                    />
                    <Route path=MyPath::Channel(None) view=chat::ChatPage>
                        <Route path=":id" view=chat::Channel/>
                        <Route path="" view=|| view! {
                            <div class="h-full bg-transparent grow flex items-center justify-center">
                                <p class="font-sans text-white text-center">"TODO: create a landing page"</p>
                            </div>
                        }/>
                    </Route>
                    <Route path=MyPath::Register view=register::RegisterPage/>
                    <Route path=MyPath::Login view=login::LoginPage/>
                </Routes>
            </main>
        </Router>
    }
}
