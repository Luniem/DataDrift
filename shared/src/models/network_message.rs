use serde::{Deserialize, Serialize};

use super::{direction::Direction, player_states::PlayerStates};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkMessage {
    StartGame(StartGameMessage), // event that tells that the game starts, includes the first states already
    ConnectionInfo(ConnectionInfoMessage), // info that is send to the user when he is connecting
    GameState(GameStateMessage), // cyclic update of the game
    PlayerUpdate(PlayerUpdateMessage), // the update that the player sents to the server
}

#[derive(Serialize, Deserialize)]
pub struct StartGameMessage {
    pub start_time: u64,
    pub player_states: Vec<PlayerStates>,
}

#[derive(Serialize, Deserialize)]
pub struct ConnectionInfoMessage {
    pub player_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct GameStateMessage {
    pub player_states: Vec<PlayerStates>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerUpdateMessage {
    pub current_direction: Direction,
}
