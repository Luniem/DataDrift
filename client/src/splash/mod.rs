use bevy::prelude::*;

use crate::{despawn_screen, player::ConnectionInfo};

use super::GameState;

pub fn splash_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Splash), splash_setup)
        .add_systems(
            Update,
            check_for_connection_info.run_if(in_state(GameState::Splash)),
        )
        .add_systems(OnExit(GameState::Splash), despawn_screen::<OnSplashScreen>);
}

#[derive(Component)]
struct OnSplashScreen;

fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load("logo.png");
    // Display the logo
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            OnSplashScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(icon),
                Node {
                    width: Val::Px(200.0),
                    ..default()
                },
            ));
        });
}

// turn to menu once we are connected
fn check_for_connection_info(
    connection_info: Option<Res<ConnectionInfo>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if connection_info.is_some() {
        // end splash screen
        game_state.set(GameState::Menu);
    }
}

// let message = message_receiver.receiver.blocking_recv();
//         if let Some(message) = message {
//             match message {
//                 NetworkMessage::ConnectionInfo(connection_info_message) => {
//                     // put connection info as resource
//                     commands.insert_resource(ConnectionInfo {
//                         uuid: connection_info_message.player_id,
//                         players_connected: connection_info_message.players_connected,
//                     });
//                     // end splash screen
//                     game_state.set(GameState::Menu);
//                 }
//                 _ => {}
//             };
//         }
