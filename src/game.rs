use std::collections::HashMap;

pub struct EloManager {
    players: HashMap<String, Player>,
}

impl EloManager {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, player: Player) {
        self.players.insert(name, player);
    }

    pub fn players(&self) -> &HashMap<String, Player> {
        &self.players
    }

    pub fn process(&mut self, game: &Game) {
        let expected = Game::expected_score(self.mean_elo(&game.team1), self.mean_elo(&game.team2));
        let score: f32 = game.score.into();
        let delta = Game::K * (score - expected);
        for player in &game.team1 {
            self.players.get_mut(player).unwrap().0 += delta;
        }
        for player in &game.team2 {
            self.players.get_mut(player).unwrap().0 -= delta;
        }
    }

    fn elo(&mut self, player: &str) -> f32 {
        self.players
            .get(player)
            .map(|x| x.into())
            .unwrap_or_else(|| {
                self.players.insert(player.to_string(), Player::new());
                Player::DEFAULT_ELO
            })
    }

    fn mean_elo(&mut self, team: &[String]) -> f32 {
        team.iter().map(|x| self.elo(x)).sum::<f32>() / team.len() as f32
    }
}

impl Default for EloManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Game {
    team1: Vec<String>,
    team2: Vec<String>,
    score: Score,
}

impl Game {
    const K: f32 = 60f32;
    const R: f32 = 400f32;

    pub fn new<T: AsRef<str>>(team1: &[T], team2: &[T], score: Score) -> Self {
        assert_eq!(team1.len(), team2.len(), "Different size of teams");
        Self {
            team1: team1.iter().map(|x| x.as_ref().to_string()).collect(),
            team2: team2.iter().map(|x| x.as_ref().to_string()).collect(),
            score,
        }
    }

    fn expected_score(elo1: f32, elo2: f32) -> f32 {
        let delta = (elo2 - elo1).max(-Self::R).min(Self::R);
        1f32 / (1f32 + 10f32.powf(delta / Self::R))
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

impl Into<f32> for &Player {
    fn into(self) -> f32 {
        self.0
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
