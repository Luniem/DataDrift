use async_std::sync::Mutex;
use game_state::{start_up_game_loop, GameState};
use incoming_networking::spawn_up_listener_thread;
use shared::models::player_states::LobbyState;
use std::{collections::HashMap, sync::Arc};

mod game_state;
mod incoming_networking;

#[tokio::main]
async fn main() {
    // firing up a clean game_state
    let game_state = Arc::new(Mutex::new(GameState {
        players: Vec::new(),
        lobby_state: LobbyState::Waiting,
        player_write_sockets: HashMap::new(),
    }));

    // add the listener for new connections
    let listener_thread = spawn_up_listener_thread(Arc::clone(&game_state));
    // start the game-loop
    start_up_game_loop(game_state);

    // keep main-thread running as long as the listener-thread is up
    listener_thread.await.unwrap();
}
