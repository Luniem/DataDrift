use shared::models::player_states::PlayerStates;
use uuid::Uuid;

pub struct GameState {
    pub players: Vec<PlayerStates>,
    pub game_started: bool,
}

impl GameState {
    pub fn connecting_player(&mut self) -> String {
        let uuid = Uuid::new_v4().to_string();
        let new_player = PlayerStates::new(&uuid);
        self.players.push(new_player);
        uuid
    }

    pub fn move_players(&self) -> () {}

    pub fn check_collision(&mut self) -> () {
        for i in 0..self.players.len() {
            let (first_part, second_part) = self.players.split_at_mut(i + 1);
            let first_player = &mut first_part[i];

            if !first_player.is_alive {
                continue;
            }

            first_player.collides_with_own_trail();

            for second_player in second_part {
                first_player.collides_with_other_player(second_player);
            }
        }
    }
}
