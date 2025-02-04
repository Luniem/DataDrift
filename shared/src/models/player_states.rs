use serde::{Deserialize, Serialize};

use super::{direction::Direction, COLLISION_RADIUS};

// macro for calculating distance
macro_rules! distance {
    ($x1: expr, $y1: expr, $x2: expr, $y2: expr) => {
        (($x2 - $x1).powi(2) + ($y2 - $y1).powi(2)).sqrt()
    };
}

#[derive(Serialize, Deserialize)]
pub struct PlayerStates {
    pub id: String,
    position_x: f32,
    position_y: f32,
    direction: f32,
    pub is_alive: bool,
    trail: Vec<(f32, f32)>,
    current_direction: Direction,
}

impl PlayerStates {
    pub fn new(uuid: &str) -> Self {
        Self {
            id: uuid.to_string(),
            position_x: 0.0,
            position_y: 0.0,
            direction: 0.0,
            is_alive: true,
            trail: Vec::new(),
            current_direction: Direction::Straight,
        }
    }

    pub fn collides_with_own_trail(&mut self) -> () {
        if self.collide_with_trail_collection(&self.trail) {
            self.is_alive = false;
        }
    }

    pub fn collides_with_other_player(&mut self, other_player: &PlayerStates) -> () {
        if self.collide_with_trail_collection(&other_player.trail) {
            self.is_alive = false;
        }
    }

    fn collide_with_trail_collection(&self, trail_collection: &Vec<(f32, f32)>) -> bool {
        for (px, py) in trail_collection {
            if distance!(self.position_x, self.position_y, px, py) < COLLISION_RADIUS {
                return true;
            }
        }

        false
    }
}
