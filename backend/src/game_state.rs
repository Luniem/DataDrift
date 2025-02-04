use std::{collections::HashMap, sync::Arc};

use async_std::net::TcpStream;
use async_tungstenite::{tungstenite::Message, WebSocketStream};
use futures::{stream::SplitSink, SinkExt};
use shared::models::{
    network_message::{ConnectionInfoMessage, NetworkMessage},
    player_states::PlayerStates,
};
use uuid::Uuid;

pub struct GameState {
    pub players: Vec<PlayerStates>,
    pub game_started: bool,
    pub player_write_sockets: HashMap<String, SplitSink<WebSocketStream<TcpStream>, Message>>,
}

impl GameState {
    pub fn connecting_player(&mut self) -> String {
        let uuid = Uuid::new_v4().to_string();
        let new_player = PlayerStates::new(&uuid);
        self.players.push(new_player);
        uuid
    }

    pub fn move_players(&self) -> () {}

    pub fn check_collision(&mut self) -> () {
        for i in 0..self.players.len() {
            let (first_part, second_part) = self.players.split_at_mut(i + 1);
            let first_player = &mut first_part[i];

            if !first_player.is_alive {
                continue;
            }

            first_player.collides_with_own_trail();

            for second_player in second_part {
                first_player.collides_with_other_player(second_player);
            }
        }
    }

    pub async fn notify_about_player_joining(&mut self) -> () {
        let player_count = self.player_write_sockets.len();
        println!("Player count: {}", player_count);

        for (uuid, write_socket) in &mut self.player_write_sockets {
            let message = NetworkMessage::ConnectionInfo(ConnectionInfoMessage {
                player_id: uuid.to_string(),
                players_connected: player_count as u32,
            });

            let serialized_player = serde_json::to_string(&message).unwrap();
            println!("Sending message: {}", uuid);

            // Await the send operation to ensure the message is sent
            if let Err(e) = write_socket.send(Message::Text(serialized_player)).await {
                println!("Failed to send message to {}: {:?}", uuid, e);
            }
        }
    }

    pub fn notify_all_players(&mut self, message: NetworkMessage) -> () {
        self.player_write_sockets
            .iter_mut()
            .for_each(|(uuid, write_socket)| {
                let serialized_player = serde_json::to_string(&message).unwrap();
                let _ = write_socket.send(Message::Text(serialized_player));
            });
    }

    // pub fn notify_single_players(&mut self, uuid: String, message: NetworkMessage) -> () {
    //     self.player_write_sockets
    //         .iter()
    //         .for_each(|(uuid, mut write_socket)| {
    //             let serialized_player = serde_json::to_string(&message).unwrap();
    //             tokio::spawn(async move {
    //                 let _ = write_socket.send(Message::Text(serialized_player));
    //             });
    //         });
    // }
}
