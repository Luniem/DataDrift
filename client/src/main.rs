use bevy::prelude::*;
use game::FrontendLobbyState;
use networking::{setup_network_client, NetworkClient, UnboundedReceiverResource};
use player::ConnectionInfo;
use shared::models::{network_message::NetworkMessage, player_states::PlayerStates};

// this could be implemented in a way that the user can select its own server
const BACKEND_WEBSOCKET_URL: &str = "ws://localhost";

mod game;
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

#[derive(Resource)]
pub struct BackendState {
    pub countdown: u32,
    pub players: Vec<PlayerStates>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_seconds(0.01))
        .insert_resource(BackendState {
            countdown: 0,
            players: Vec::new(),
        })
        .init_state::<GameState>()
        .add_systems(Startup, (setup_camera, setup_network_client))
        .add_systems(
            Update,
            (handle_websocket_messages, check_exit_game, handle_exit),
        )
        // .add_systems(FixedUpdate, send_player_updates)
        .add_plugins((splash::splash_plugin, menu::menu_plugin, game::game_plugin))
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

fn handle_websocket_messages(
    mut commands: Commands,
    mut message_receiver: ResMut<UnboundedReceiverResource>,
    mut lobby_state: ResMut<NextState<FrontendLobbyState>>,
    mut backend_state: ResMut<BackendState>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if !message_receiver.receiver.is_empty() {
        let message = message_receiver.receiver.blocking_recv();
        if let Some(message) = message {
            match message {
                NetworkMessage::ConnectionInfo(connection_info_message) => {
                    commands.insert_resource(ConnectionInfo {
                        uuid: connection_info_message.player_id,
                        players_connected: connection_info_message.players_connected,
                    });
                }
                NetworkMessage::GameState(game_state_message) => {
                    // check lobby-state
                    game_state.set(GameState::Game);

                    backend_state.players = game_state_message.player_states;

                    match game_state_message.lobby_state {
                        shared::models::player_states::LobbyState::Waiting => {}
                        shared::models::player_states::LobbyState::Countdown(num) => {
                            backend_state.countdown = num;
                            lobby_state.set(FrontendLobbyState::Countdown);
                        }
                        shared::models::player_states::LobbyState::Running => {
                            lobby_state.set(FrontendLobbyState::Running)
                        }
                        shared::models::player_states::LobbyState::Finished => {
                            lobby_state.set(FrontendLobbyState::Finished)
                        }
                    };
                }
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
