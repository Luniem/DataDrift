use std::{collections::HashMap, f32::consts::PI, sync::Arc, time::Duration};

use async_std::{net::TcpStream, sync::Mutex};
use async_tungstenite::{tungstenite::Message, WebSocketStream};
use futures::{stream::SplitSink, SinkExt};
use rand::Rng;
use shared::models::{
    direction::Direction,
    network_message::{ConnectionInfoMessage, GameStateMessage, NetworkMessage},
    player_states::{LobbyState, PlayerStates},
    GAME_BOARD_HEIGHT, GAME_BOARD_WIDTH, MILLIS_PER_TICK,
};
use tokio::time::interval;
use uuid::Uuid;

/// GameState that handles all the state in the game
pub struct GameState {
    pub players: Vec<PlayerStates>,
    pub lobby_state: LobbyState,
    pub player_write_sockets: HashMap<String, SplitSink<WebSocketStream<TcpStream>, Message>>,
}

impl GameState {
    pub fn connecting_player(
        &mut self,
        write_socket: SplitSink<WebSocketStream<TcpStream>, Message>,
    ) -> String {
        let uuid = Uuid::new_v4().to_string();
        let new_player = PlayerStates::new(&uuid);
        self.players.push(new_player);

        self.player_write_sockets.insert(uuid.clone(), write_socket);

        uuid
    }

    pub fn disconnecting_player(&mut self, uuid: &str) -> () {
        let remove_index = self
            .players
            .iter()
            .position(|p| p.id == uuid)
            .expect("Could not find player!");

        self.players.remove(remove_index);

        // remove socket
        self.player_write_sockets.remove(uuid);
    }

    pub fn init_game(&mut self) -> () {
        // Initialize random generator
        let mut rng = rand::rng();

        // Set bounds for random position
        let x_min = -250.0;
        let x_max = 250.0;
        let y_min = -250.0;
        let y_max = 250.0;

        // Iterate over all players to set a random position and direction
        for player in self.players.iter_mut() {
            // Randomize player position within the defined bounds
            player.position_x = rng.random_range(x_min..x_max);
            player.position_y = rng.random_range(y_min..y_max);

            // Randomize the player's direction (angle) between 0 and 2 * PI
            player.direction = rng.random_range(0.0..(2.0 * PI));
            player.is_alive = true;
            player.current_direction = Direction::Straight;
            player.trail = Vec::new();
        }
    }

    pub fn next_step(&mut self) -> () {
        // move all players - calculate directions
        for player in self.players.as_mut_slice() {
            if player.is_alive {
                player.steer_player();
                player.move_player();
            }
        }

        //check for collisions
        self.check_collision();

        // check if finished (is finished when only 1 or none player are alive)
        let alive_players_count = self.players.iter().filter(|p| p.is_alive).count();
        if alive_players_count <= 1 {
            self.lobby_state = LobbyState::Finished;
        }
    }

    pub fn check_collision(&mut self) -> () {
        for i in 0..self.players.len() {
            let (first_part, second_part) = self.players.split_at_mut(i + 1);
            let first_player = &mut first_part[i];

            if !first_player.is_alive {
                continue;
            }

            // check if we are out of bounds
            if first_player.position_x > (GAME_BOARD_WIDTH / 2.0)
                || first_player.position_x < (GAME_BOARD_WIDTH / 2.0 * -1.0)
                || first_player.position_y > (GAME_BOARD_HEIGHT / 2.0)
                || first_player.position_y < (GAME_BOARD_HEIGHT / 2.0 * -1.0)
            {
                first_player.is_alive = false;
            }

            for second_player in second_part {
                if second_player.is_alive
                    && second_player.collide_with_trail_collection(&first_player.trail)
                {
                    second_player.is_alive = false;
                }

                first_player.collides_with_other_player(second_player);
            }
        }
    }

    pub async fn notify_about_player_joining(&mut self) -> () {
        let player_count = self.player_write_sockets.len();

        for (uuid, write_socket) in &mut self.player_write_sockets {
            let message = NetworkMessage::ConnectionInfo(ConnectionInfoMessage {
                player_id: uuid.to_string(),
                players_connected: player_count as u32,
            });

            let serialized_player = serde_json::to_string(&message).unwrap();

            if let Err(e) = write_socket.send(Message::Text(serialized_player)).await {
                println!("Failed to send message to {}: {:?}", uuid, e);
            }
        }
    }

    pub async fn notify_all_players(&mut self, message: String) -> () {
        for (uuid, write_socket) in &mut self.player_write_sockets {
            if let Err(e) = write_socket.send(Message::Text(message.clone())).await {
                println!("Failed to send message to {}: {:?}", uuid, e);
            }
        }
    }

    pub fn get_game_state_message(&self) -> NetworkMessage {
        let lobby_state = match self.lobby_state {
            LobbyState::Waiting => LobbyState::Waiting,
            LobbyState::Countdown(countdown) => LobbyState::Countdown(countdown),
            LobbyState::Running => LobbyState::Running,
            LobbyState::Finished => LobbyState::Finished,
        };

        let player_states: Vec<PlayerStates> = self.players.clone();

        NetworkMessage::GameState(GameStateMessage {
            lobby_state: lobby_state,
            player_states: player_states,
        })
    }

    pub fn update_player(&mut self, player_id: &str, direction: Direction) -> () {
        let player_index = self
            .players
            .iter()
            .position(|p| p.id == player_id)
            .expect("Could not find player!");

        // update player direction
        self.players
            .get_mut(player_index)
            .unwrap()
            .set_direction(direction);
    }
}

pub fn start_up_game_loop(game_state: Arc<Mutex<GameState>>) -> () {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(MILLIS_PER_TICK as u64));

        loop {
            // we just wait for the interval tick down
            // if computation inside the game-loop is taking to long, we wouldn't be able to produce the wished ticks per second anymore
            // then we would have to keep track of time elapsed since we started the game-loop computation
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
                    // move to next step in game-state
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
