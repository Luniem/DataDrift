use async_std::{net::TcpListener, sync::Mutex};
use async_tungstenite::accept_async;
use futures::StreamExt;
use game_state::GameState;
use serde_json::Error;
use shared::models::{
    network_message::NetworkMessage, player_states::LobbyState, MILLIS_PER_TICK, PORT,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{task::JoinHandle, time::interval};

pub mod game_state;

#[tokio::main]
async fn main() {
    let game_state = Arc::new(Mutex::new(GameState {
        players: Vec::new(),
        lobby_state: LobbyState::Waiting,
        player_write_sockets: HashMap::new(),
    }));

    let listener_thread = spawn_up_listener_thread(Arc::clone(&game_state));
    spawn_up_game_ticks(game_state);

    listener_thread.await.unwrap();
}

fn spawn_up_listener_thread(game_state: Arc<Mutex<GameState>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let server = TcpListener::bind(format!("127.0.0.1:{}", PORT))
            .await
            .unwrap();

        while let Ok((stream, _)) = server.accept().await {
            let cloned_game_state = Arc::clone(&game_state);

            tokio::spawn(async move {
                let websocket = accept_async(stream).await.unwrap();
                // split the websocket into a read and write stream
                let (write_stream, mut read_stream) = websocket.split();

                let uuid = {
                    let mut game_state = cloned_game_state.lock().await;
                    // add the new player to the game state
                    let uuid = game_state.connecting_player();
                    game_state
                        .player_write_sockets
                        .insert(uuid.clone(), write_stream);

                    game_state.notify_about_player_joining().await;
                    uuid
                };

                // handle incoming messages
                loop {
                    let msg = read_stream.next().await;

                    if msg.is_some() {
                        match msg.unwrap() {
                            Ok(msg) => {
                                let message_text = msg.to_text();
                                if message_text.is_ok() {
                                    let parsed_message: Result<NetworkMessage, Error> =
                                        serde_json::from_str(message_text.unwrap());

                                    if parsed_message.is_ok() {
                                        match parsed_message.unwrap() {
                                            NetworkMessage::RequestStart(_) => {
                                                // first init the game

                                                // TODO: clean that up a bit
                                                {
                                                    let mut game_state =
                                                        cloned_game_state.lock().await;
                                                    game_state.init_game();
                                                }

                                                let mut interval = interval(Duration::from_secs(1));
                                                let mut countdown = 6;

                                                for _ in 0..6 {
                                                    {
                                                        // make countdown
                                                        let mut game_state =
                                                            cloned_game_state.lock().await;
                                                        game_state.lobby_state =
                                                            LobbyState::Countdown(countdown)
                                                    }

                                                    countdown -= 1;
                                                    interval.tick().await;
                                                }

                                                {
                                                    let mut game_state =
                                                        cloned_game_state.lock().await;
                                                    game_state.lobby_state = LobbyState::Running;
                                                }
                                            }
                                            NetworkMessage::PlayerUpdate(player_update_message) => {
                                                let mut game_state = cloned_game_state.lock().await;
                                                game_state.update_player(
                                                    &uuid,
                                                    player_update_message.current_direction,
                                                );
                                            }
                                            _ => {}
                                        };
                                    } else {
                                        break;
                                    }
                                }
                            }
                            Err(_) => {
                                println!("Connection closed!");
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }

                {
                    let mut game_state = cloned_game_state.lock().await;
                    game_state.disconnecting_player(&uuid);
                    game_state.notify_about_player_joining().await;
                }
            });
        }
    })
}

fn spawn_up_game_ticks(game_state: Arc<Mutex<GameState>>) -> () {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(30));

        loop {
            interval.tick().await;
            let mut game_state = game_state.lock().await;
            let should_update = match game_state.lobby_state {
                LobbyState::Waiting => false,
                LobbyState::Countdown(_) => true,
                LobbyState::Running => true,
                LobbyState::Finished => false,
            };

            if should_update {
                if game_state.lobby_state == LobbyState::Running {
                    // update game-state
                    game_state.next_step();
                }

                // send update to all clients
                let message = game_state.get_game_state_message();
                game_state
                    .notify_all_players(serde_json::to_string(&message).unwrap())
                    .await;
            }
        }
    });
}
