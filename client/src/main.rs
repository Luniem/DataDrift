use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use networking::{setup_network_client, NetworkClient, UnboundedReceiverResource};
use player::{move_player, Player};
use shared::models::{
    direction::Direction,
    network_message::{NetworkMessage, PlayerUpdateMessage},
};

const BACKEND_WEBSOCKET_URL: &str = "ws://localhost:9001";

mod networking;
mod player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_seconds(0.02))
        .add_systems(Startup, (setup, setup_network_client))
        .add_systems(
            FixedUpdate,
            (send_player_updates, handle_websocket_messages),
        )
        .add_systems(Update, (move_player, check_exit_game, handle_exit))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());
    // We want to have a startscreen first.
    // commands.spawn((
    //     Player::default(),
    //     Sprite::from_image(asset_server.load("bullet.png")),
    //     Transform::from_xyz(100.0, 100.0, 0.0),
    // ));
}

/// Here we check if the user presses ESC for closing the game
fn check_exit_game(keys: Res<ButtonInput<KeyCode>>, mut app_exit_writer: EventWriter<AppExit>) {
    // check if we pressed ESC
    if keys.pressed(KeyCode::Escape) {
        app_exit_writer.send(AppExit::Success);
    }
}

/// Handle disconnect of network-client on game-quit
fn handle_exit(
    mut app_exit_reader: EventReader<AppExit>,
    mut network_client: ResMut<NetworkClient>,
) {
    // if someone request exit of app, disconnect network-client
    for app_exit_event in app_exit_reader.read() {
        if app_exit_event == &AppExit::Success {
            network_client.disconnect();
        }
    }
}

fn send_player_updates(
    query: Query<&Player, With<Player>>,
    mut network_client: ResMut<NetworkClient>,
) {
    for player in query.iter() {
        // update the player to the backend
        let curr_dir = match player.current_direction {
            Direction::Left => Direction::Left,
            Direction::Right => Direction::Right,
            Direction::Straight => Direction::Straight,
        };
        let update_message = NetworkMessage::PlayerUpdate(PlayerUpdateMessage {
            current_direction: curr_dir,
        });
        network_client.send_message(update_message);
    }
}

fn handle_websocket_messages(mut message_receiver: ResMut<UnboundedReceiverResource>) {
    if message_receiver.receiver.is_empty() {
        println!("Wait for message");
        let message = message_receiver.receiver.blocking_recv();
        if let Some(message) = message {
            match message {
                NetworkMessage::StartGame(start_game_message) => todo!(),
                NetworkMessage::ConnectionInfo(connection_info_message) => {
                    println!("{}", connection_info_message.player_id);
                }
                NetworkMessage::GameState(game_state_message) => todo!(),
                NetworkMessage::PlayerUpdate(player_update_message) => {
                    panic!("Should not get a player update message from backend!")
                }
            };
        }
    }
}
