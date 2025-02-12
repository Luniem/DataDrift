use std::{sync::Arc, time::Duration};

use async_std::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use async_tungstenite::{accept_async, tungstenite::Message};
use futures::StreamExt;
use shared::models::{network_message::NetworkMessage, player_states::LobbyState, PORT};
use tokio::{task::JoinHandle, time::interval};

use crate::game_state::GameState;

/// Opens up a new thread that listens for new connections - for each connection there will be another thread to handle incoming messages
pub fn spawn_up_listener_thread(game_state: Arc<Mutex<GameState>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let server = TcpListener::bind(format!("0.0.0.0:{}", PORT))
            .await
            .expect("Failed to start TCP-Listener!");

        while let Ok((stream, _)) = server.accept().await {
            // open new thread for each connection and give it the game state
            let cloned_game_state = Arc::clone(&game_state);
            tokio::spawn(handle_connection(stream, cloned_game_state));
        }
    })
}

/// Handles the websocket-connection, holds it open and listens for incoming messages
async fn handle_connection(stream: TcpStream, game_state: Arc<Mutex<GameState>>) {
    let websocket = accept_async(stream)
        .await
        .expect("Error during websocket handshake occured!");

    // split the websocket into a read and write stream
    let (write_stream, mut read_stream) = websocket.split();

    let uuid = {
        let mut game_state = game_state.lock().await;
        // add the new player to the game state
        let uuid = game_state.connecting_player(write_stream);
        // tell the other players about the joined player
        game_state.notify_about_player_joining().await;
        uuid
    };

    // handle incoming messages
    'messageloop: loop {
        let incoming_message = read_stream.next().await;

        if let Some(incoming_message) = incoming_message {
            match incoming_message {
                Ok(incoming_message) => {
                    handle_valid_message(incoming_message, &game_state, &uuid).await
                }
                Err(err) => {
                    // log the error - for now just connection closing
                    match err {
                        async_tungstenite::tungstenite::Error::ConnectionClosed => {
                            println!("Connection closed!");
                            break 'messageloop;
                        }
                        _ => {
                            println!("Unexpected Error occured!");
                        }
                    };

                    // Let us break out of the message loop here.
                    // it could be possible that not all errors lead to a unvalid connection - but for the sake of simplicity
                    break 'messageloop;
                }
            };
        } else {
            break;
        }
    }

    // remove the player from the game state and notify the other players
    let mut game_state = game_state.lock().await;
    game_state.disconnecting_player(&uuid);
    game_state.notify_about_player_joining().await;
    drop(game_state);
}

/// Handles a valid message - deserializes it and acts accordingly
async fn handle_valid_message(
    message: Message,
    cloned_game_state: &Arc<Mutex<GameState>>,
    uuid: &str,
) -> () {
    let parsed_message: Result<NetworkMessage, String> = message
        .to_text()
        .map_err(|err| format!("{:?}", err)) // convert the error to a string
        .and_then(|message_text| {
            serde_json::from_str(message_text)
                .map_err(|err| format!("Failed deserializing: {}", err))
        });

    match parsed_message {
        Ok(parsed_message) => {
            // we only want to listen to request-start and player-update messages
            match parsed_message {
                NetworkMessage::RequestStart(_) => {
                    // first init the game
                    let mut game_state = cloned_game_state.lock().await;
                    game_state.init_game();
                    drop(game_state);

                    // then start the countdown
                    let mut countdown = 5;
                    let mut interval = interval(Duration::from_secs(1));
                    interval.tick().await; // first tick will elapse immediately

                    while countdown > 0 {
                        // count the start of the game down
                        //game-loop will send info to the players
                        let mut game_state = cloned_game_state.lock().await;
                        game_state.lobby_state = LobbyState::Countdown(countdown);
                        drop(game_state);

                        countdown -= 1;
                        interval.tick().await;
                    }

                    let mut game_state = cloned_game_state.lock().await;
                    game_state.lobby_state = LobbyState::Running;
                    // lock will be dropped here automatically
                }
                NetworkMessage::PlayerUpdate(player_update_message) => {
                    let mut game_state = cloned_game_state.lock().await;
                    game_state.update_player(&uuid, player_update_message.current_direction);
                    // lock will be dropped here automatically
                }
                _ => println!("Received unexpected message!"),
            }
        }
        Err(err) => println!("Error during parsing of message: {:?}", err),
    };
}
