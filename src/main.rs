#![feature(impl_trait_in_fn_trait_return)]

use axum::{extract::FromRef, routing::get};
use leptos::prelude::*;
use leptos_axum::AxumRouteListing;
use scrum_poker::components::poker::room::backend::ServerState;

#[derive(FromRef, Debug, Clone)]
struct GlobalAppState {
    leptos_options: LeptosOptions,
    routes: Vec<AxumRouteListing>,
    server_state: ServerState,
}

#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use scrum_poker::app::*;
    use scrum_poker::components::poker::room::backend::ws_handler;

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .with_level(true)
        .init();

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).unwrap();
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
        .leptos_routes_with_context(
            &server_state,
            routes,
            {
                let server_state = server_state.server_state.clone();
                move || {
                    provide_context(server_state.clone());
                }
            },
            {
                let leptos_options = server_state.leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .route("/ws/room/:room_id/:uid", get(ws_handler))
        .fallback(leptos_axum::file_and_error_handler::<GlobalAppState, _>(
            shell,
        ))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
