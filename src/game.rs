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

    fn mean_elo(&mut self, team: &Team) -> f32 {
        let player_list = self.get();
        let mut elo = 0f32;
        for player in team.iter() {
            elo += player_list
                .get(player)
                .map(|&player| player.into())
                .unwrap_or_else(|| {
                    player_list.insert(player.clone(), Player::new());
                    Player::DEFAULT_ELO
                });
        }
        elo / team.len() as f32
    }
}
pub static mut PLAYERS: Players = Players { players: None };

#[derive(Debug)]
pub struct Game {
    team1: Team,
    team2: Team,
    score: Score,
}

impl Game {
    const K: f32 = 60f32;
    const R: f32 = 400f32;

    pub fn new<T: Into<String>>(participants: Vec<T>, score: Score) -> Self {
        assert_eq!(0, participants.len() % 2, "Odd number of participants");
        let mut team1 = participants;
        let team2 = team1.split_off(team1.len() / 2);
        Self {
            team1: Team(team1.into_iter().map(|x| x.into()).collect()),
            team2: Team(team2.into_iter().map(|x| x.into()).collect()),
            score,
        }
    }

    pub fn process(&self) {
        let players = unsafe { PLAYERS.get() };
        let expected = Self::expected_score(&self.team1, &self.team2);
        let score: f32 = self.score.into();
        let delta = Self::K * (score - expected);
        for player in self.team1.0.iter() {
            players.get_mut(player).unwrap().0 += delta;
        }
        for player in self.team2.0.iter() {
            players.get_mut(player).unwrap().0 -= delta;
        }
    }

    fn expected_score<T: Into<f32>>(elo1: T, elo2: T) -> f32 {
        let delta = (elo2.into() - elo1.into()).max(-Self::R).min(Self::R);
        1f32 / (1f32 + 10f32.powf(delta / Self::R))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Player(f32);

impl Player {
    const DEFAULT_ELO: f32 = 2000f32;

    pub fn new() -> Self {
        Self::DEFAULT_ELO.into()
    }
}

impl From<f32> for Player {
    fn from(elo: f32) -> Self {
        Self(elo)
    }
}

impl Into<f32> for Player {
    fn into(self) -> f32 {
        self.0
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Team(Vec<String>);

impl Into<f32> for &Team {
    fn into(self) -> f32 {
        unsafe { PLAYERS.mean_elo(self) }
    }
}

impl std::ops::Deref for Team {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Score {
    Win,
    Loss,
    Draw,
}

impl Into<f32> for Score {
    fn into(self) -> f32 {
        match self {
            Score::Win => 1f32,
            Score::Loss => 0f32,
            Score::Draw => 0.5f32,
        }
    }
}

impl std::convert::TryFrom<char> for Score {
    type Error = String;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'W' | '1' => Ok(Score::Win),
            'L' | '2' => Ok(Score::Loss),
            'D' => Ok(Score::Draw),
            x => Err(format!("Cannot convert '{}' into Score", x)),
        }
    }
}
