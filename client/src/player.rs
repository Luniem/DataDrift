use bevy::prelude::*;
use shared::models::{
    direction::Direction,
    network_message::{NetworkMessage, PlayerUpdateMessage},
    MOVE_SPEED, ROTATION_SPEED,
};

use crate::{game::OnGameScreen, networking::NetworkClient, BackendState};

#[derive(Resource)]
pub struct ConnectionInfo {
    pub uuid: String,
    pub players_connected: u32,
}

#[derive(Resource)]
pub struct RenderedTrails {
    pub count: u32,
}

#[derive(Component)]
pub struct Player {
    pub uuid: String,
    pub rotation: Quat,
    pub is_alive: bool,
    pub is_own_player: bool,
    pub current_direction: Direction,
}

impl Player {
    pub fn steer_player(
        &mut self,
        keys: &Res<ButtonInput<KeyCode>>,
        time_delta: f32,
    ) -> Option<PlayerUpdateMessage> {
        let left_is_clicked = keys.pressed(KeyCode::ArrowLeft);
        let right_is_clicked = keys.pressed(KeyCode::ArrowRight);

        if left_is_clicked == right_is_clicked {
            if self.current_direction != Direction::Straight {
                self.current_direction = Direction::Straight;
                return Some(PlayerUpdateMessage {
                    current_direction: Direction::Straight,
                });
            }
        } else if keys.pressed(KeyCode::ArrowLeft) {
            let rotation = Quat::from_rotation_z(ROTATION_SPEED * time_delta);
            self.rotation = rotation * self.rotation;

            if self.current_direction != Direction::Left {
                self.current_direction = Direction::Left;
                return Some(PlayerUpdateMessage {
                    current_direction: Direction::Left,
                });
            }
        } else if keys.pressed(KeyCode::ArrowRight) {
            let rotation = Quat::from_rotation_z(-ROTATION_SPEED * time_delta);
            self.rotation = rotation * self.rotation;

            if self.current_direction != Direction::Right {
                self.current_direction = Direction::Right;

                return Some(PlayerUpdateMessage {
                    current_direction: Direction::Right,
                });
            }
        }

        None
    }
}

pub fn move_player(
    mut query: Query<(&mut Player, &mut Transform), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    network_client: ResMut<NetworkClient>,
) {
    for (mut player, mut transform) in query.iter_mut() {
        if player.is_alive {
            let mut direction = Vec3::ZERO;

            if player.is_own_player {
                let player_update = player.steer_player(&keys, time.delta_secs());
                if let Some(player_update) = player_update {
                    network_client.send_message(NetworkMessage::PlayerUpdate(player_update));
                }
            }

            // move our player according to local input
            direction.x += MOVE_SPEED * time.delta_secs();
            direction = player.rotation.mul_vec3(direction);

            if 0.0 < direction.length() {
                transform.translation += direction;
                transform.rotation = player.rotation;
            }
        }
    }
}

pub fn spawn_players_according_to_backend(
    mut commands: Commands,
    backend_state: Res<BackendState>,
    connection_info: Res<ConnectionInfo>,
    asset_server: Res<AssetServer>,
) {
    let player_id = &connection_info.uuid;
    for player in backend_state.players.iter().filter(|p| &p.id != player_id) {
        let quat = Quat::from_rotation_z(player.direction);

        commands.spawn((
            Sprite::from_image(asset_server.load("bullet.png")),
            Transform::from_xyz(player.position_x, player.position_y, 1.0).with_rotation(quat),
            Player {
                uuid: player.id.clone(),
                current_direction: player.current_direction.clone(),
                is_alive: player.is_alive,
                is_own_player: false,
                rotation: Quat::from_rotation_z(player.direction),
            },
            OnGameScreen,
        ));
    }

    // own player
    let own_player = backend_state
        .players
        .iter()
        .position(|p| &p.id == player_id)
        .map(|index| backend_state.players.get(index))
        .flatten();

    if let Some(own_player) = own_player {
        let quat = Quat::from_rotation_z(own_player.direction);
        
        commands.spawn((
            Sprite::from_image(asset_server.load("own_bullet.png")),
            Transform::from_xyz(own_player.position_x, own_player.position_y, 1.0).with_rotation(quat),
            Player {
                uuid: own_player.id.clone(),
                current_direction: Direction::Straight,
                is_alive: own_player.is_alive,
                is_own_player: true,
                rotation: Quat::from_rotation_z(own_player.direction),
            },
            OnGameScreen,
        ));
    }
}
