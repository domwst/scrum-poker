use super::api::{check_username, hide, place_bet, reveal, set_name, PlayerGameState, PlayerState};
use crate::{
    error_template::{AppError, ErrorTemplate},
    if_frontend,
};
use getrandom::getrandom;
use leptos::{
    component, create_action, create_memo, create_signal, event_target_value,
    leptos_dom::logging::console_log, spawn_local, view, Errors, IntoView, Params, SignalGet,
    SignalGetUntracked, SignalSet, SignalWith,
};
use leptos_router::{use_params, Params};
use std::{cmp::Reverse, iter};

fn game_state_updates(
    room_id: u64,
    uid: u64,
) -> impl SignalGet<Value = PlayerGameState> + SignalWith<Value = PlayerGameState> + Copy {
    // TODO: use create_signal_from_stream
    let (state, set_state) = create_signal(PlayerGameState::default());
    if_frontend! {
        use futures::StreamExt;
        use gloo_net::websocket::{futures::WebSocket, Message::Text};
        use gloo_utils::window;

        let protocol = if window().location().protocol().unwrap() == "https:" { "wss" } else { "ws" };
        let origin = window().location().host().unwrap();

        let conn = WebSocket::open(&format!("{protocol}://{origin}/ws/room/{room_id}/{uid}"))
            .expect("failed to open ws");

        let mut recv = conn.split().1;
        spawn_local(async move {
            while let Some(msg) = recv.next().await {
                match msg {
                    Ok(msg) => {
                        if let Text(msg) = msg {
                            let v = serde_json::from_str(&msg);
                            if let Ok(v) = v {
                                set_state.set(v);
                            } else {
                                console_log(&format!("Error parsing message: {v:?}"));
                            }
                        } else {
                            console_log(&format!("Unexpected message type: {msg:?}"));
                        }
                    }
                    Err(e) => {
                        console_log(&format!("Error receiving msg: {e:?}"));
                        break;
                    }
                }
            }
        });
    }
    state
}

fn get_random_u64() -> Result<u64, getrandom::Error> {
    let mut res = [0u8; 8];
    getrandom(&mut res)?;
    Ok(u64::from_ne_bytes(res))
}

#[component]
fn CardThick() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-5 h-5">
            <path d="m15.5,0h-7C5.467,0,3,2.467,3,5.5v13c0,3.033,2.467,5.5,5.5,5.5h7c3.033,0,5.5-2.467,5.5-5.5V5.5c0-3.033-2.467-5.5-5.5-5.5Zm2.5,18.5c0,1.378-1.122,2.5-2.5,2.5h-7c-1.378,0-2.5-1.122-2.5-2.5V5.5c0-1.378,1.122-2.5,2.5-2.5h7c1.378,0,2.5,1.122,2.5,2.5v13Zm-2.044-5.072c-.151.727-.733,1.335-1.454,1.511-.741.181-1.436-.053-1.904-.515.197.66.454,1.284.802,1.825.209.325-.046.752-.432.752h-1.935c-.386,0-.641-.427-.432-.752.348-.541.606-1.165.802-1.825-.469.462-1.163.696-1.904.515-.721-.176-1.303-.784-1.454-1.511-.268-1.292.711-2.428,1.956-2.428.101,0,.197.015.293.03-.182-.302-.293-.651-.293-1.03,0-1.105.895-2,2-2s2,.895,2,2c0,.378-.111.728-.293,1.03.097-.014.193-.03.293-.03,1.244,0,2.223,1.136,1.956,2.428Z"/>
        </svg>
    }
}

#[component]
fn NameChange(current_name: String, creds: (u64, u64)) -> impl IntoView {
    let (uid, room_id) = creds;
    let (new_name, set_new_name) = create_signal(current_name);
    let set_name = create_action(move |name: &String| {
        let name = name.clone();
        async move {
            if let Err(e) = set_name(room_id, uid, name.clone()).await {
                console_log(&format!("Received error response {e:?}"));
            }
        }
    });
    let nameError = create_memo(move |_| new_name.with(|s| check_username(s)));
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if nameError.get().is_ok() {
            set_name.dispatch(new_name.get());
        }
    };
    let on_input = move |ev| {
        set_new_name(event_target_value(&ev));
    };

    let input_classes = move |is_error: bool| {
        let default = "input input-bordered flex items-center gap-2";
        if is_error {
            format!("{default} input-error")
        } else {
            default.to_string()
        }
    };

    view! {
        <form on:submit=on_submit>
            <label class=move || input_classes(nameError.get().is_err())>
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" class="w-4 h-4 opacity-70">
                    <path d="M8 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6ZM12.735 14c.618 0 1.093-.561.872-1.139a6.002 6.002 0 0 0-11.215 0c-.22.578.254 1.139.872 1.139h9.47Z" />
                </svg>
                <input type="text" class="grow" prop:value=new_name on:input=on_input />
                { move ||
                    match nameError.get() {
                        Ok(_) => view!{}.into_view(),
                        Err(e) => view! {
                            <div class="label">
                                <span class="label-text-alt text-error">{ e }</span>
                            </div>
                        }.into_view()
                    }
                }
            </label>
        </form>
    }
}

fn convert_to_double(card: u64) -> String {
    let s = format!("{:.<2}", card as f64 / 100.);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[component]
fn CardChange<
    CardsSignal: SignalGet<Value = Vec<u64>> + Copy + 'static,
    SelfCardSignal: SignalGet<Value = Option<u64>> + Copy + 'static,
>(
    cards: CardsSignal,
    self_card: SelfCardSignal,
    creds: (u64, u64),
) -> impl IntoView {
    let (uid, room_id) = creds;

    let place_bet = create_action(move |&card: &Option<u64>| async move {
        if let Err(e) = place_bet(room_id, uid, card).await {
            console_log(&format!("Received error response {e:?}"));
        }
    });
    view! {
        <div>
        { move || {
            let default_classes = "btn mr-2";
            let active_classes = format!("{default_classes} btn-active");
            let self_card = self_card.get();
            cards.get().iter().copied().map(|v| view! {
                <button
                    on:click=move |_| place_bet.dispatch(Some(v))
                    class=if Some(v) == self_card { active_classes.clone() } else { default_classes.to_string() }
                >
                    { convert_to_double(v) }
                </button>
            }).collect::<Vec<_>>()
        }}
            <button on:click=move |_| place_bet.dispatch(None) class="btn">"X"</button>
        </div>
    }
}

#[component]
fn HideReveal<
    HiddenSignal: SignalGet<Value = bool> + Copy + 'static,
    AvgSignal: SignalGet<Value = u64> + Copy + 'static,
>(
    hidden: HiddenSignal,
    avg: AvgSignal,
    creds: (u64, u64),
) -> impl IntoView {
    let (_, room_id) = creds;
    let reveal = create_action(move |_: &()| async move {
        if let Err(e) = reveal(room_id).await {
            console_log(&format!("Received error response {e:?}"));
        }
    });

    let hide = create_action(move |_: &()| async move {
        if let Err(e) = hide(room_id).await {
            console_log(&format!("Received error response {e:?}"));
        }
    });

    view! {
        <div>
        { move || {
            if hidden.get() {
                view! {
                    <button on:click=move |_| reveal.dispatch(()) class="btn">"Reveal"</button>
                }.into_view()
            } else {
                view! {
                    <button on:click=move |_| hide.dispatch(()) class="btn">
                        "Average is " { convert_to_double(avg.get()) }
                    </button>
                }.into_view()
            }
        }}
        </div>
    }
}

#[component]
fn GameStateTable<GameStateSignal: SignalGet<Value = PlayerGameState> + Copy + 'static>(
    game_state: GameStateSignal,
) -> impl IntoView {
    view! {
        <table class="table xl:table-lg">
            <thead class="uppercase">
                <tr>
                    <th>"Player"</th>
                    <th>"Bet"</th>
                </tr>
            </thead>
            <tbody>
            { move || {
                let state = game_state.get();
                let render_player = |PlayerState { card, name }, is_self: bool| view! {
                    <tr class=if is_self { "bg-base-300" } else { "hover:bg-base-200" }>
                        <td>{ name }</td>
                        <td>
                        { match card {
                            Some(v) => if state.hidden && !is_self {
                                CardThick().into_view()
                            } else {
                                convert_to_double(v).into_view()
                            },
                            None => "".into_view(),
                        }}
                        </td>
                    </tr>
                };
                let mut players =
                    state
                        .players
                        .into_iter()
                        .map(|v| (v, false))
                        .chain(iter::once((state.self_state, true)))
                        .collect::<Vec<_>>();
                if !state.hidden {
                    players.sort_unstable_by_key(|player| Reverse(player.0.card));
                }
                players
                    .into_iter()
                    .map(|(player, is_self)| render_player(player, is_self))
                    .collect::<Vec<_>>()
            }}
            </tbody>
        </table>
    }
}

#[derive(Params, Clone, PartialEq)]
struct PokerRoomId {
    room_id: u64,
}

/// Renders the home page of your application.
#[component]
pub fn PokerRoom() -> impl IntoView {
    let room_id = match use_params::<PokerRoomId>().get_untracked() {
        Ok(r) => r.room_id,
        Err(_) => {
            let mut errors = Errors::default();
            errors.insert_with_default_key(AppError::NotFound);
            return view! {
                <ErrorTemplate outside_errors=errors />
            }
            .into_view();
        }
    };
    let uid = get_random_u64().unwrap();
    let game_state = game_state_updates(room_id, uid);
    let avg_bet = create_memo(move |_| {
        game_state.with(|state| {
            let bets = state
                .players
                .iter()
                .chain(iter::once(&state.self_state))
                .filter_map(|state| state.card);
            let sm: u64 = bets.clone().sum();
            let cnt = bets.count();
            if cnt == 0 {
                0
            } else {
                sm / cnt as u64
            }
        })
    });

    let current_name = create_memo(move |_| game_state.with(|state| state.self_state.name.clone()));

    view! {
        <div class="max-w-4xl mx-auto px-8 sm:px-4 lg:px-6 pt-6">
            <h1 class="text-base md:text-xl lg:text-3xl font-bold my-1 text-center">"Let's play poker!"</h1>
            <h2 class="text-base md:text-lg lg:text-xl font-semibold my-1 text-center">"Room #" { room_id }</h2>
            <div class="mt-2">
                <div>
                    <GameStateTable game_state=game_state />
                </div>
                <div class="mt-2">
                    <CardChange
                        cards=create_memo(move |_| game_state.with(|state| state.cards.clone()))
                        self_card=create_memo(move |_| game_state.with(|state| state.self_state.card))
                        creds=(uid, room_id)
                    />
                </div>
                <div class="mt-2">
                    <HideReveal
                        hidden=create_memo(move |_| game_state.with(|state| state.hidden))
                        avg=avg_bet
                        creds=(uid, room_id)
                    />
                </div>
                <div class="mt-2">
                { move || {
                    view!{
                        <NameChange
                            current_name=current_name.get()
                            creds=(uid, room_id)
                        />
                    }
                }}
                </div>
            </div>
        </div>
    }.into_view()
}
