use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

use super::{direction::Direction, COLLISION_RADIUS, MOVE_SPEED_PER_TICK, ROTATION_SPEED_PER_TICK};

#[derive(Serialize, Deserialize, PartialEq)]
pub enum LobbyState {
    Waiting,
    Countdown(u32),
    Running,
    Finished,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerStates {
    pub id: String,
    pub position_x: f32,
    pub position_y: f32,
    pub direction: f32,
    pub is_alive: bool,
    pub trail: Vec<(f32, f32)>,
    pub current_direction: Direction,
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

    pub fn set_direction(&mut self, direction: Direction) -> () {
        self.current_direction = direction;
    }

    pub fn collide_with_trail_collection(&self, trail_collection: &Vec<(f32, f32)>) -> bool {
        for (px, py) in trail_collection {
            let dx = px - self.position_x;
            let dy = py - self.position_y;

            if (dx * dx + dy * dy).sqrt() < COLLISION_RADIUS {
                println!("Collided!");
                return true;
            }
        }

        false
    }

    pub fn steer_player(&mut self) -> () {
        match self.current_direction {
            Direction::Left => {
                self.direction += ROTATION_SPEED_PER_TICK;
                self.direction = self.direction % (PI * 2.0);
            }
            Direction::Right => {
                self.direction -= ROTATION_SPEED_PER_TICK;
                self.direction = self.direction % (PI * 2.0);
            }
            Direction::Straight => {}
        };
    }

    pub fn move_player(&mut self) -> () {
        let dx = MOVE_SPEED_PER_TICK * self.direction.cos();
        let dy = MOVE_SPEED_PER_TICK * self.direction.sin();

        self.position_x += dx;
        self.position_y += dy;

        self.trail.push((self.position_x, self.position_y));
    }
}

impl Clone for PlayerStates {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            position_x: self.position_x.clone(),
            position_y: self.position_y.clone(),
            direction: self.direction.clone(),
            is_alive: self.is_alive.clone(),
            trail: self.trail.clone(),
            current_direction: self.current_direction.clone(),
        }
    }
}
