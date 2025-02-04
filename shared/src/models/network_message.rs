use serde::{Deserialize, Serialize};

use super::{
    direction::Direction,
    player_states::{LobbyState, PlayerStates},
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkMessage {
    ConnectionInfo(ConnectionInfoMessage), // info that is send to the user when he is connecting
    RequestStart(()),                      // player requests start of game
    PlayerUpdate(PlayerUpdateMessage),     // the update that the player sents to the server
    GameState(GameStateMessage),           // cyclic update of the game
}

#[derive(Serialize, Deserialize)]
pub struct ConnectionInfoMessage {
    pub player_id: String,
    pub players_connected: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GameStateMessage {
    pub lobby_state: LobbyState,
    pub player_states: Vec<PlayerStates>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerUpdateMessage {
    pub current_direction: Direction,
}
