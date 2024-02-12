use axum::{
    body::Body as AxumBody,
    extract::{FromRef, Path, State},
    http::Request,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use hey_leptos::app;
use hey_leptos::fileserv::file_and_error_handler;
use leptos::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
use leptos_router::RouteListing;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub type DummyState = Arc<RwLock<HashMap<String, app::login_register_page::User>>>;

#[derive(Clone, FromRef, Debug)]
pub struct AppState {
    pub db: DummyState,
    pub leptos_options: LeptosOptions,
    pub routes: Vec<RouteListing>,
}

async fn server_fn_handler(
    State(app_state): State<AppState>,
    path: Path<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    logging::log!("{:?}", path);

    handle_server_fns_with_context(
        move || {
            // provide_context(auth_session.clone());
            provide_context(app_state.db.clone());
        },
        request,
    )
    .await
}

async fn leptos_routes_handler(
    State(app_state): State<AppState>,
    req: Request<AxumBody>,
) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || {
            // provide_context(auth_session.clone());
            provide_context(app_state.db.clone());
        },
        app::App,
    );
    handler(req).await.into_response()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let db = DummyState::default();

    let conf = get_configuration(None)
        .await
        .map_err(|err| std::io::Error::other(format!("{:?}", err)))?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let app_routes = generate_route_list(app::App);

    let app_state = AppState {
        db: db.clone(),
        leptos_options: leptos_options.clone(),
        routes: app_routes.clone(),
    };

    // build our application with a route
    let router = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(app_routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}

// Setting get_configuration(None) means we'll be using cargo-leptos's env values
// For deployment these variables are:
// <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
// Alternately a file can be specified such as Some("Cargo.toml")
// The file would need to be included with the executable when moved to deployment
