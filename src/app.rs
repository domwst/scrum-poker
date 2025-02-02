use crate::components::poker::{main::frontend::PickRoom, room::frontend::PokerRoom};
use crate::error_template::{AppError, ErrorTemplate};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, MetaTags, Stylesheet, Title};
use leptos_router::{components::{Router, Routes, Route}, path};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/static/scrum-poker.css"/>
        <Link rel="icon" href="/card-club-thick.svg" />

        <Title text="Poker" />

        <Router>
            <main>
                <Routes fallback=|| {
                    let mut outside_errors = Errors::default();
                    outside_errors.insert_with_default_key(AppError::NotFound);
                    view! {
                        <ErrorTemplate outside_errors/>
                    }
                }>
                    <Route path=path!("") view=PickRoom />
                    <Route path=path!("rooms/:room_id") view=PokerRoom/>
                </Routes>
            </main>
        </Router>
    }
}
