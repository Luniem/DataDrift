pub mod direction;
pub mod network_message;
pub mod player_states;

pub const PORT: &str = "9001";

const COLLISION_RADIUS: f32 = 20.0;

pub const MOVE_SPEED: f32 = 200.0;
pub const ROTATION_SPEED: f32 = 2.5;
pub const TICKS_PER_SECOND: f32 = 30.0; // Backend ticks per second
pub const MILLIS_PER_TICK: f32 = 1000.0 / TICKS_PER_SECOND;
pub const ROTATION_SPEED_PER_TICK: f32 = ROTATION_SPEED / TICKS_PER_SECOND;
pub const MOVE_SPEED_PER_TICK: f32 = MOVE_SPEED / TICKS_PER_SECOND;

pub const GAME_BOARD_WIDTH: f32 = 1000.0;
pub const GAME_BOARD_HEIGHT: f32 = 1000.0;
