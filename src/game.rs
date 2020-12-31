use std::collections::HashMap;

#[derive(Debug)]
pub struct Players {
    players: Option<HashMap<String, Player>>,
}

impl Players {
    pub fn get(&mut self) -> &mut HashMap<String, Player> {
        if self.players.is_none() {
            self.players = Some(HashMap::new());
        }
        self.players.as_mut().unwrap()
    }
}
pub static mut PLAYERS: Players = Players { players: None };

#[derive(Debug)]
pub struct Game {
    team1: Vec<String>,
    team2: Vec<String>,
    result: f32,
}

impl Game {
    const K: f32 = 120f32;
    const R: f32 = 400f32;

    pub fn new<T: Into<String>>(participants: Vec<T>, result: f32) -> Self {
        assert_eq!(0, participants.len() % 2, "Odd number of participants");
        let mut team1 = participants;
        let team2 = team1.split_off(team1.len() / 2);
        Self {
            team1: team1.into_iter().map(|x| x.into()).collect(),
            team2: team2.into_iter().map(|x| x.into()).collect(),
            result,
        }
    }

    pub fn process(&self) {
        let n = self.team1.len();
        let players = unsafe { PLAYERS.get() };
        let team1_elo = {
            let mut elo = 0f32;
            for player in &self.team1 {
                elo += if let Some(player) = players.get(player) {
                    player.elo
                } else {
                    players.insert(player.to_string(), Player::new());
                    Player::DEFAULT_ELO
                }
            }
            elo / n as f32
        };
        let team2_elo = {
            let mut elo = 0f32;
            for player in &self.team2 {
                elo += if let Some(player) = players.get(player) {
                    player.elo
                } else {
                    players.insert(player.to_string(), Player::new());
                    Player::DEFAULT_ELO
                }
            }
            elo / n as f32
        };
        let expected = Self::expected_score(team1_elo, team2_elo);
        let delta = Self::K * (self.result - expected);
        for player in &self.team1 {
            players.get_mut(player).unwrap().elo += delta;
        }
        for player in &self.team2 {
            players.get_mut(player).unwrap().elo -= delta;
        }
    }

    fn expected_score(elo1: f32, elo2: f32) -> f32 {
        let delta = (elo2 - elo1).max(-Self::R).min(Self::R);
        1f32 / (1f32 + 10f32.powf(delta / Self::R))
    }
}

#[derive(Debug)]
pub struct Player {
    elo: f32,
}

impl Player {
    const DEFAULT_ELO: f32 = 2000f32;

    pub fn new() -> Self {
        Self::with_elo(Self::DEFAULT_ELO)
    }

    pub fn with_elo(elo: f32) -> Self {
        Self { elo }
    }

    pub fn elo(&self) -> f32 {
        self.elo
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
