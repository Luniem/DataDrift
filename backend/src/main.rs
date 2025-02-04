use async_std::{net::TcpListener, sync::Mutex};
use async_tungstenite::accept_async;
use futures::StreamExt;
use game_state::GameState;
use shared::models::network_message::{ConnectionInfoMessage, NetworkMessage};
use std::{
    collections::HashMap,
    hash::Hash,
    sync::Arc,
    thread::{sleep, spawn},
    time::Duration,
};
use tokio::{task::JoinHandle, time::interval};

pub mod game_state;

#[tokio::main]
async fn main() {
    let game_state = Arc::new(Mutex::new(GameState {
        players: Vec::new(),
        game_started: false,
        player_write_sockets: HashMap::new(),
        // player_sockets: HashMap::new(),
    }));

    let listener_thread = spawn_up_listener_thread(Arc::clone(&game_state));
    spawn_up_game_ticks(game_state);

    listener_thread.await.unwrap();
}

fn spawn_up_listener_thread(game_state: Arc<Mutex<GameState>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let server = TcpListener::bind("127.0.0.1:9001").await.unwrap();

        while let Ok((stream, addr)) = server.accept().await {
            let cloned_game_state = Arc::clone(&game_state);

            tokio::spawn(async move {
                let mut websocket = accept_async(stream).await.unwrap();
                // split the websocket into a read and write stream
                let (write_stream, mut read_stream) = websocket.split();

                {
                    let mut game_state = cloned_game_state.lock().await;
                    // add the new player to the game state
                    let uuid = game_state.connecting_player();
                    game_state
                        .player_write_sockets
                        .insert(uuid.clone(), write_stream);

                    game_state.notify_about_player_joining().await;
                }

                // handle incoming messages
                loop {
                    let msg = read_stream.next().await.unwrap();

                    match msg {
                        Ok(msg) => {
                            // TODO: player send update - store it somewhere

                            println!("{}", msg.len());
                        }
                        Err(_) => {
                            println!("Connection closed!");
                            break;
                        }
                    }
                }

                {
                    let mut game_state = cloned_game_state.lock().await;
                    game_state.players.remove(0);
                }
            });
        }
    })
}

fn spawn_up_game_ticks(game_state: Arc<Mutex<GameState>>) -> () {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(20));

        loop {
            interval.tick().await;
            let game_state = game_state.lock().await;
        }
    });
}
