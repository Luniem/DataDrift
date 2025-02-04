use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Left = 0,
    Right = 1,
    Straight = 2,
}
