use bevy::prelude::*;
use networking::{setup_network_client, NetworkClient, UnboundedReceiverResource};
use player::Player;
use shared::models::{
    direction::Direction,
    network_message::{NetworkMessage, PlayerUpdateMessage},
};

const BACKEND_WEBSOCKET_URL: &str = "ws://localhost:9001";

mod menu;
mod networking;
mod player;
mod splash;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Splash,
    Menu,
    Game,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_seconds(0.02))
        .init_state::<GameState>()
        .add_systems(Startup, (setup_camera, setup_network_client))
        .add_systems(Update, (check_exit_game, handle_exit))
        // .add_systems(
        //     FixedUpdate,
        //     (send_player_updates, handle_websocket_messages),
        // )
        // .add_systems(Update, move_player)
        .add_plugins((splash::splash_plugin, menu::menu_plugin))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
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

fn send_player_updates(query: Query<&Player, With<Player>>, network_client: Res<NetworkClient>) {
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

        // send messsage to backend
        network_client.send_message(update_message);
    }
}

fn handle_websocket_messages(mut message_receiver: ResMut<UnboundedReceiverResource>) {
    if !message_receiver.receiver.is_empty() {
        println!("Wait for message");
        let message = message_receiver.receiver.blocking_recv();
        if let Some(message) = message {
            match message {
                NetworkMessage::StartGame(_start_game_message) => {
                    todo!("set game state to game and spawn all entites etc")
                }
                NetworkMessage::ConnectionInfo(connection_info_message) => {
                    println!(
                        "We got a connection info: {}",
                        connection_info_message.player_id
                    );
                }
                NetworkMessage::GameState(_game_state_message) => todo!(),
                _ => {}
            };
        }
    }
}

// used for despaning all entities with a specific component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
