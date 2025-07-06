#![feature(impl_trait_in_fn_trait_return)]

use time::ext::{NumericalDuration, NumericalStdDuration};

use async_nats::jetstream;
use axum::extract::FromRef;
use leptos::prelude::*;
use leptos_axum::AxumRouteListing;
use scrum_poker::{components::poker::room::backend::ServerState, session_store::NatsSessionStore};
use tower_sessions::{Expiry, SessionManagerLayer};

#[derive(FromRef, Debug, Clone)]
struct GlobalAppState {
    leptos_options: LeptosOptions,
    routes: Vec<AxumRouteListing>,
    server_state: ServerState,
}

struct EnvVar<'a> {
    name: &'a str,
    default: &'a str,
}

impl<'a> EnvVar<'a> {
    const fn new(name: &'a str, default: &'a str) -> Self {
        Self { name, default }
    }

    fn get(&self) -> String {
        std::env::var(&self.name).unwrap_or_else(|_| {
            tracing::warn!(
                "No value provided for {}, using default value instead: {}",
                self.name,
                self.default
            );
            self.default.to_string()
        })
    }
}

#[tokio::main]
async fn main() {
    const NATS_URL: EnvVar<'static> = EnvVar::new("NATS_URL", "nats://localhost:4222");
    const SESSIONS_BUCKET: EnvVar<'static> = EnvVar::new("SESSION_BUCKET", "sessions");

    use axum::Router;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use scrum_poker::app::*;

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .with_level(true)
        .init();

    let nats_url = NATS_URL.get();
    let sessions_bucket = SESSIONS_BUCKET.get();

    let client = async_nats::connect(nats_url).await.unwrap();
    let js = jetstream::new(client);

    let bucket = js
        .create_or_update_key_value(jetstream::kv::Config {
            bucket: sessions_bucket,
            description: "".to_string(),
            max_value_size: 1024,
            history: 0,
            max_age: 2.std_days(),
            num_replicas: 1,
            ..Default::default()
        })
        .await
        .unwrap();
    let session_store = NatsSessionStore::new(bucket);
    let session_manager = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(6.hours()))
        .with_secure(true);

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
        .fallback(leptos_axum::file_and_error_handler::<GlobalAppState, _>(
            shell,
        ))
        .layer(session_manager)
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
