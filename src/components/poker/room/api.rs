use leptos::{prelude::*, server_fn::BoxedStream};
use serde::{Deserialize, Serialize};
use server_fn::{Websocket, codec::JsonEncoding};

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
    use super::backend::{Game, ServerState};
    use crate::uid::{get_or_create_uid, get_uid};

    use leptos_axum::{extract, ResponseOptions};
    use tower_sessions::Session;
    use http::StatusCode;
    use futures::{StreamExt, stream};
    use tokio::select;
    use tracing::{info, error, warn};
    use std::sync::Arc;
    use atomic_refcell::AtomicRefCell;

    async fn get_game(room_id: u64) -> Result<Game, ServerError> {
        let state = use_context::<ServerState>().expect("ServerState to be provided");
        state
            .get_game(room_id)
            .await
            .ok_or_else(|| ServerError::new_custom("No such room"))
    }

    async fn get_session() -> Result<Session, ServerError> {
        extract().await.map_err(Into::into)
    }

    fn set_status(code: StatusCode) {
        expect_context::<ResponseOptions>().set_status(code);
    }

    async fn get_uid_server(session: &Session) -> Result<u128, ServerError> {
        get_uid(session)
            .await
            .map_err(|e| {
                error!("Failed to retrieve uid: {e}");
                ServerError::new_custom("Internal server error")
            })?.ok_or_else(|| {
                warn!("Connection without uid");
                set_status(StatusCode::UNAUTHORIZED);
                ServerError::new_custom("Unauthorized")
            })
    }

    async fn get_or_create_uid_server(session: &Session) -> Result<u128, ServerError> {
        get_or_create_uid(session).await.map_err(|e| {
            error!("Failed to get uid: {e}");
            ServerError::new_custom("Internal server error")
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerError {
    ServerFnError(ServerFnErrorErr),
    Custom(String),
}

impl ServerError {
    pub fn new_custom(s: impl Into<String>) -> Self {
        Self::Custom(s.into())
    }
}

impl From<ServerFnErrorErr> for ServerError {
    fn from(value: ServerFnErrorErr) -> Self {
        Self::ServerFnError(value)
    }
}

impl FromServerFnError for ServerError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        value.into()
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum UserStreamRequest {
    SetRoom(u64),
}

#[server(protocol = Websocket<JsonEncoding, JsonEncoding>, prefix = "/api")]
pub async fn subscribe_to_room(
    inp: BoxedStream<UserStreamRequest, ServerError>,
) -> Result<BoxedStream<PlayerGameState, ServerError>, ServerError> {
    let mut inp = inp;
    let session = get_session().await?;
    let uid = get_or_create_uid_server(&session).await?;

    let (tx, rx) = tokio::sync::watch::channel(PlayerGameState::default());
    let rx = Arc::new(AtomicRefCell::new(rx));

    let state = use_context::<ServerState>().expect("ServerState to be provided");

    tokio::spawn(async move {
        let (_tx, mut rx) = tokio::sync::mpsc::channel(1);

        loop {
            select! {
                cmd = inp.next() => {
                    let cmd = match cmd {
                        Some(Ok(v)) => v,
                        Some(Err(e)) => {
                            info!("User disconnected: {e:?}");
                            break
                        },
                        None => break,
                    };
                    let room = match cmd {
                        UserStreamRequest::SetRoom(room) => room,
                    };

                    let game = state.get_or_create_game(room).await;

                    rx = game.0.lock().await.new_player(uid).await;
                }
                state = rx.recv() => {
                    let state = match state {
                        Some(v) => v,
                        None => break,
                    };
                    if tx.send(state).is_err() {
                        break;
                    }
                }
            }
        }
    });

    Ok(stream::iter(0..)
        .filter_map(move |_| {
            let rx = Arc::clone(&rx);
            async move {
                let mut rx = rx.borrow_mut();
                if rx.changed().await.is_err() {
                    return None;
                }
                let v = PlayerGameState::clone(&rx.borrow_and_update());
                Some(Ok(v))
            }
        })
        .into())
}

#[server(name = PlaceBet, prefix = "/api")]
pub async fn place_bet(room_id: u64, card: Option<u64>) -> Result<(), ServerError> {
    let session = get_session().await?;
    let uid = get_uid_server(&session).await?;
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
pub async fn reveal(room_id: u64) -> Result<(), ServerError> {
    get_game(room_id).await?.0.lock().await.reveal().await;
    Ok(())
}

#[server(name = Hide, prefix = "/api")]
pub async fn hide(room_id: u64) -> Result<(), ServerError> {
    get_game(room_id).await?.0.lock().await.hide().await;
    Ok(())
}

#[server(name = SetName, prefix = "/api")]
pub async fn set_name(room_id: u64, name: String) -> Result<(), ServerError> {
    check_username(&name).map_err(|e| {
        set_status(StatusCode::BAD_REQUEST);
        ServerError::Custom(e)
    })?;
    let session = get_session().await?;
    let uid = get_uid_server(&session).await?;
    get_game(room_id)
        .await?
        .0
        .lock()
        .await
        .set_name(uid, name)
        .await;
    Ok(())
}
