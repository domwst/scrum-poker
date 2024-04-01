use crate::random_nickname::gen_nickname;
use atomic_refcell::AtomicRefCell;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock};

use super::api::{PlayerGameState, PlayerState};

#[derive(Debug, Clone, Default)]
pub struct ServerState(Arc<AsyncRwLock<ServerStateInner>>);

impl ServerState {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn new_connection(&self, room_id: u64, uid: u64, ws: WebSocket) {
        self.get_or_create_game(room_id)
            .await
            .0
            .lock()
            .await
            .new_player(uid, ws.split().0)
            .await;
    }

    pub(super) async fn get_game(&self, room_id: u64) -> Option<Game> {
        self.0.read().await.get_game(room_id).await
    }

    pub(super) async fn get_or_create_game(&self, room_id: u64) -> Game {
        if let Some(game) = self.get_game(room_id).await {
            return game;
        }
        self.0.write().await.get_or_create_game(room_id).await
    }
}

#[derive(Debug, Default)]
struct ServerStateInner {
    games: HashMap<u64, Game>,
}

impl ServerStateInner {
    async fn get_game(&self, room_id: u64) -> Option<Game> {
        self.games.get(&room_id).cloned()
    }

    async fn get_or_create_game(&mut self, room_id: u64) -> Game {
        self.games.entry(room_id).or_insert_with(Game::new).clone()
    }
}

#[derive(Debug, Default, Clone)]
pub(super) struct Game(pub Arc<AsyncMutex<GameInner>>);

impl Game {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug)]
pub(super) struct Player {
    card: Option<u64>,
    receiver: AtomicRefCell<SplitSink<WebSocket, Message>>, // NOTE this is a CRUNCH
    name: String,
}

impl From<&Player> for PlayerState {
    fn from(player: &Player) -> PlayerState {
        PlayerState {
            card: player.card,
            name: player.name.clone(),
        }
    }
}

#[derive(Debug)]
pub(super) struct GameInner {
    cards: Vec<u64>,
    players: HashMap<u64, Player>,
    hidden: bool,
}

impl Default for GameInner {
    fn default() -> Self {
        Self {
            cards: vec![50, 100, 200, 300, 500, 800, 1300, 2100],
            players: Default::default(),
            hidden: true,
        }
    }
}

pub async fn ws_handler(
    State(server_state): State<ServerState>,
    Path((room_id, uid)): Path<(u64, u64)>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    tracing::debug!("Received new connection");
    ws.on_upgrade(
        move |socket| async move { server_state.new_connection(room_id, uid, socket).await },
    )
}

impl GameInner {
    pub(super) async fn new_player(&mut self, uid: u64, receiver: SplitSink<WebSocket, Message>) {
        let state = Player {
            card: None,
            receiver: AtomicRefCell::new(receiver),
            name: gen_nickname(uid),
        };
        if let Some(old) = self.players.insert(uid, state) {
            Self::drop_player(uid, old).await;
        }
        self.send_update().await;
    }

    #[allow(dead_code)]
    pub(super) async fn add_new_card(&mut self, card: u64) {
        self.cards.push(card);
        self.cards.sort_unstable();
        self.cards.dedup();
        self.send_update().await;
    }

    #[allow(dead_code)]
    pub(super) async fn remove_card(&mut self, card: u64) {
        if let Some(pos) = self.cards.iter().position(|&v| v == card) {
            self.cards.remove(pos);
            self.send_update().await;
        }
    }

    pub(super) async fn set_name(&mut self, uid: u64, name: String) {
        if let Some(player) = self.players.get_mut(&uid) {
            player.name = name;
            self.send_update().await;
        }
    }

    pub(super) async fn place_bet(&mut self, uid: u64, card: Option<u64>) {
        if let Some(player) = self.players.get_mut(&uid) {
            player.card = card;
            self.send_update().await;
        }
    }

    pub(super) async fn drop_player(uid: u64, player: Player) {
        if let Err(e) = player.receiver.borrow_mut().close().await {
            tracing::debug!("Failed to close connection to the old {uid} due to {e:?}");
        }
    }

    pub(super) async fn reveal(&mut self) {
        self.hidden = false;
        self.send_update().await;
    }

    pub(super) async fn hide(&mut self) {
        self.hidden = true;
        for state in self.players.values_mut() {
            state.card = None;
        }
        self.send_update().await;
    }

    pub(super) async fn send_update(&mut self) {
        let mut disconnected = vec![];
        loop {
            for (&self_uid, self_state) in &self.players {
                let mut player_game_state = PlayerGameState {
                    cards: self.cards.clone(),
                    players: vec![],
                    self_state: self_state.into(),
                    hidden: self.hidden,
                };
                for (&other_uid, other_state) in &self.players {
                    if other_uid == self_uid {
                        continue;
                    }
                    let mut other_state: PlayerState = other_state.into();
                    if self.hidden {
                        other_state.card = other_state.card.map(|_| 0);
                    }
                    player_game_state.players.push(other_state);
                }

                if let Err(e) = self_state
                    .receiver
                    .borrow_mut()
                    .send(Message::Text(
                        serde_json::to_string(&player_game_state).unwrap(),
                    ))
                    .await
                {
                    tracing::debug!("Failed to send info to player {self_uid} due to {e:?}");
                    disconnected.push(self_uid);
                }
            }
            if disconnected.is_empty() {
                break;
            }
            for &uid in &disconnected {
                if let Some(player) = self.players.remove(&uid) {
                    Self::drop_player(uid, player).await;
                }
            }
            disconnected.clear();
        }
    }
}
