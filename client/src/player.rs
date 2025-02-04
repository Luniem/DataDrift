use bevy::prelude::*;
use shared::models::direction::Direction;

const MOVE_SPEED: f32 = 200.0;
const ROTATION_SPEED: f32 = 2.5;

#[derive(Resource)]
pub struct ConnectionInfo {
    pub uuid: String,
    pub players_connected: u32,
}

#[derive(Component)]
pub struct Player {
    pub uuid: String,
    pub rotation: Quat,
    pub current_direction: Direction,
}

impl Player {
    pub fn default(uuid: String) -> Self {
        Self {
            uuid: uuid,
            rotation: Quat::IDENTITY,
            current_direction: Direction::Straight,
        }
    }

    pub fn steer_player(&mut self, keys: &Res<ButtonInput<KeyCode>>, time_delta: f32) -> () {
        let left_is_clicked = keys.pressed(KeyCode::ArrowLeft);
        let right_is_clicked = keys.pressed(KeyCode::ArrowRight);

        if left_is_clicked == right_is_clicked {
            if self.current_direction != Direction::Straight {
                self.current_direction = Direction::Straight;
            }
        } else if keys.pressed(KeyCode::ArrowLeft) {
            let rotation = Quat::from_rotation_z(ROTATION_SPEED * time_delta);
            self.rotation = rotation * self.rotation;

            if self.current_direction != Direction::Left {
                self.current_direction = Direction::Left;
            }
        } else if keys.pressed(KeyCode::ArrowRight) {
            let rotation = Quat::from_rotation_z(-ROTATION_SPEED * time_delta);
            self.rotation = rotation * self.rotation;

            if self.current_direction != Direction::Right {
                self.current_direction = Direction::Right;
            }
        }
    }
}

pub fn move_player(
    mut query: Query<(&mut Player, &mut Transform), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut player, mut transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        player.steer_player(&keys, time.delta_secs());

        // move our player according to local input
        direction.x += MOVE_SPEED * time.delta_secs();
        direction = player.rotation.mul_vec3(direction);

        if 0.0 < direction.length() {
            transform.translation += direction;
            transform.rotation = player.rotation;
        }
    }
}
