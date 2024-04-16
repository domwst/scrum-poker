#![feature(impl_trait_in_fn_trait_return)]

use axum::{
    body::Body as AxumBody,
    extract::{FromRef, Request, State},
    response::IntoResponse,
    routing::get,
};
use leptos::*;
use leptos_axum::handle_server_fns_with_context;
use leptos_router::*;
use scrum_poker::{app::App, components::poker::room::backend::ServerState};
use std::future::Future;

#[derive(FromRef, Debug, Clone)]
struct GlobalAppState {
    leptos_options: LeptosOptions,
    routes: Vec<RouteListing>,
    server_state: ServerState,
}

async fn server_fn_handler(
    State(server_state): State<ServerState>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(server_state.clone());
        },
        request,
    )
    .await
}

fn leptos_routes_handler(
    GlobalAppState {
        leptos_options,
        routes,
        server_state,
    }: GlobalAppState,
) -> impl Fn(Request<AxumBody>) -> (impl Future<Output: IntoResponse> + Send + 'static)
       + Clone
       + Send
       + 'static {
    leptos_axum::render_route_with_context(
        leptos_options,
        routes,
        move || {
            provide_context(server_state.clone());
        },
        App,
    )
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use scrum_poker::app::*;
    use scrum_poker::components::poker::room::backend::ws_handler;
    use scrum_poker::fileserv::file_and_error_handler;

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .with_level(true)
        .init();

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let server_state = ServerState::new();
    let server_state = GlobalAppState {
        server_state,
        leptos_options,
        routes: routes.clone(),
    };

    let app = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler(server_state.clone())))
        .route("/ws/room/:room_id/:uid", get(ws_handler))
        .fallback(file_and_error_handler)
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
