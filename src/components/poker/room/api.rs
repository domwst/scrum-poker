if_backend! {
    use super::backend::{Game, ServerState};
}
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::if_backend;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlayerState {
    pub(super) card: Option<u64>,
    pub(super) name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlayerGameState {
    pub(super) players: Vec<PlayerState>,
    pub(super) cards: Vec<u64>,
    pub(super) self_state: PlayerState,
    pub(super) hidden: bool,
}

if_backend! {
    use leptos_axum::{extract, ResponseOptions};
    use tower_sessions::Session;
    use http::StatusCode;
    use crate::uid::get_uid;

    async fn get_game(room_id: u64) -> Result<Game, ServerFnError> {
        let state = use_context::<ServerState>().expect("ServerState to be provided");
        state
            .get_game(room_id)
            .await
            .map(Ok)
            .unwrap_or_else(|| Err(ServerFnError::ServerError("No such room".to_string())))
    }

    async fn get_session() -> Result<Session, ServerFnError> {
        extract().await.map_err(Into::into)
    }

    fn set_status(code: StatusCode) {
        expect_context::<ResponseOptions>().set_status(code);
    }
}

pub fn check_username(s: &str) -> Result<(), String> {
    if s.is_empty() {
        Err("Has to be non-empty")?;
    }
    if !s
        .chars()
        .all(|c| matches!(c, '0'..='9' | 'a'..='z' | 'A'..='Z' | '-' | '_'))
    {
        Err("Allowed characters: a-z A-Z 0-9 _-".to_owned())
    } else {
        Ok(())
    }
}

#[server(name = PlaceBet, prefix = "/api")]
pub async fn place_bet(room_id: u64, card: Option<u64>) -> Result<(), ServerFnError> {
    let session = get_session().await?;
    let uid = get_uid(&session)
        .await
        .map_err(|e| {
            tracing::error!("Failed to retrieve uid: {e}");
            ServerFnError::new("Internal server error")
        })?
        .unwrap();
    get_game(room_id)
        .await?
        .0
        .lock()
        .await
        .place_bet(uid, card)
        .await;
    Ok(())
}

#[server(name = Reveal, prefix = "/api")]
pub async fn reveal(room_id: u64) -> Result<(), ServerFnError> {
    get_game(room_id).await?.0.lock().await.reveal().await;
    Ok(())
}

#[server(name = Hide, prefix = "/api")]
pub async fn hide(room_id: u64) -> Result<(), ServerFnError> {
    get_game(room_id).await?.0.lock().await.hide().await;
    Ok(())
}

#[server(name = SetName, prefix = "/api")]
pub async fn set_name(room_id: u64, name: String) -> Result<(), ServerFnError> {
    use leptos::server_fn::error::NoCustomError;
    check_username(&name).map_err(|e| {
        set_status(StatusCode::BAD_REQUEST);
        ServerFnError::<NoCustomError>::ServerError(e)
    })?;
    let session = get_session().await?;
    let uid = get_uid(&session)
        .await
        .map_err(|e| {
            tracing::error!("Failed to retrieve uid: {e}");
            ServerFnError::new(e)
        })?
        .unwrap();
    get_game(room_id)
        .await?
        .0
        .lock()
        .await
        .set_name(uid, name)
        .await;
    Ok(())
}
