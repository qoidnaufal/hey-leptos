#[cfg(feature = "ssr")]
use axum::{
    body::Body as AxumBody,
    extract::State,
    http::Request,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
#[cfg(feature = "ssr")]
use axum_session::{SessionConfig, SessionLayer, SessionStore};
#[cfg(feature = "ssr")]
use axum_session_auth::{AuthConfig, AuthSessionLayer, SessionSurrealPool};
#[cfg(feature = "ssr")]
use leptos::*;
#[cfg(feature = "ssr")]
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
#[cfg(feature = "ssr")]
use surrealdb::engine::remote::ws::Client as SurrealClient;

#[cfg(feature = "ssr")]
use hey_leptos::{
    app,
    auth_model::AuthSession,
    db,
    fileserv::file_and_error_handler,
    // messaging,
    // ws,
    rooms_manager,
    state,
    user_model::UserData,
};

#[cfg(feature = "ssr")]
async fn server_fn_handler(
    State(app_state): State<state::AppState>,
    auth_session: AuthSession,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pool.clone());
            provide_context(app_state.rooms_manager.clone());
        },
        request,
    )
    .await
}

#[cfg(feature = "ssr")]
async fn leptos_routes_handler(
    State(app_state): State<state::AppState>,
    auth_session: AuthSession,
    req: Request<AxumBody>,
) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pool.clone());
            provide_context(app_state.rooms_manager.clone());
        },
        app::App,
    );
    handler(req).await.into_response()
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pool = db::Database::init()
        .await
        .map_err(|err| std::io::Error::other(err))?;

    let rooms_manager = rooms_manager::ssr::RoomsManager::new();

    let conf = get_configuration(None)
        .await
        .map_err(|err| std::io::Error::other(format!("{:?}", err)))?;

    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let app_routes = generate_route_list(app::App);

    // Auth section
    let session_config = SessionConfig::default().with_table_name("user_sessions");
    let auth_config = AuthConfig::<String>::default();
    let session_store = SessionStore::<SessionSurrealPool<SurrealClient>>::new(
        Some(pool.clone().client.into()),
        session_config,
    )
    .await
    .map_err(|err| std::io::Error::other(err))?;

    // AppState
    let app_state = state::AppState {
        pool: pool.clone(),
        leptos_options: leptos_options.clone(),
        routes: app_routes.clone(),
        rooms_manager,
    };

    // Router
    let router = Router::new()
        // .route("/channel/:room_uuid", get(messaging::msg_subscriber))
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(app_routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .layer(
            AuthSessionLayer::<
                UserData,
                String,
                SessionSurrealPool<SurrealClient>,
                db::Database,
            >::new(Some(pool.clone()))
            .with_config(auth_config),
        )
        .layer(SessionLayer::new(session_store))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {}
