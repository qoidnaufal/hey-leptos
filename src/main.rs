#[cfg(feature = "ssr")]
use {
    axum::{
        body::Body as AxumBody,
        extract::State,
        http::Request,
        response::{IntoResponse, Response},
        routing::get,
        Router,
    },
    axum_session::{SessionConfig, SessionLayer, SessionStore},
    axum_session_auth::{AuthConfig, AuthSessionLayer, SessionSurrealPool},
    hey_leptos::{
        app, fileserv, messaging,
        models::user_model,
        state::{self, auth, db, rooms_manager},
    },
    leptos::*,
    leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes},
    surrealdb::engine::remote::ws::Client as SurrealClient,
};

#[cfg(feature = "ssr")]
async fn server_fn_handler(
    State(app_state): State<state::AppState>,
    auth_session: auth::AuthSession,
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
    auth_session: auth::AuthSession,
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
    let rooms_manager = rooms_manager::RoomsManager::init();
    let conf = get_configuration(None)
        .await
        .map_err(|err| std::io::Error::other(err))?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let app_routes = generate_route_list(app::App);

    // --- Auth
    let session_config = SessionConfig::default().with_table_name("user_sessions");
    let auth_config = AuthConfig::<String>::default();
    let session_store = SessionStore::<SessionSurrealPool<SurrealClient>>::new(
        Some(pool.clone().client.into()),
        session_config,
    )
    .await
    .map_err(|err| std::io::Error::other(err))?;

    // --- AppState
    let app_state = state::AppState {
        pool: pool.clone(),
        leptos_options: leptos_options.clone(),
        routes: app_routes.clone(),
        rooms_manager: rooms_manager.clone(),
    };

    // --- Router
    let router = Router::new()
        // .route("/ws:id", get(messaging::ws_handler))
        .route("/ws", get(messaging::ws_handler))
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(app_routes, get(leptos_routes_handler))
        .fallback(fileserv::file_and_error_handler)
        .layer(
            AuthSessionLayer::<
                user_model::User,
                String,
                SessionSurrealPool<SurrealClient>,
                db::Database,
            >::new(Some(pool.clone()))
            .with_config(auth_config),
        )
        .layer(SessionLayer::new(session_store))
        .with_state(app_state);

    // --- Serve
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {}
