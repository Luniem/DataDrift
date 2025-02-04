use game_state::GameState;
use shared::models::network_message::{ConnectionInfoMessage, NetworkMessage};
use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
    thread::{spawn, JoinHandle},
    time::Duration,
};
use tokio::time::interval;
use tungstenite::accept;

pub mod game_state;

#[tokio::main]
async fn main() {
    let game_state = Arc::new(Mutex::new(GameState {
        players: Vec::new(),
        game_started: false,
    }));

    let listener_thread = spawn_up_listener_thread(Arc::clone(&game_state));
    spawn_up_game_ticks(game_state);

    listener_thread.join().unwrap();
}

fn spawn_up_listener_thread(game_state: Arc<Mutex<GameState>>) -> JoinHandle<()> {
    spawn(move || {
        let server = TcpListener::bind("127.0.0.1:9001").unwrap();

        for stream in server.incoming() {
            // copy the game state for the new connection
            let cloned_game_state = Arc::clone(&game_state);

            spawn(move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                {
                    let mut game_state = cloned_game_state.lock().unwrap();
                    let uuid = game_state.connecting_player();
                    println!("Connected: {}", uuid);

                    // send back the uuid
                    let conn_info_msg =
                        NetworkMessage::ConnectionInfo(ConnectionInfoMessage { player_id: uuid });
                    let serialized_message = serde_json::to_string(&conn_info_msg).unwrap();
                    let _ = websocket.send(serialized_message.into()); // normally we would check here for working message
                }

                loop {
                    let msg = websocket.read();

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
                    let mut game_state = cloned_game_state.lock().unwrap();
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
            let game_state = game_state.lock().unwrap();
        }
    });
}
