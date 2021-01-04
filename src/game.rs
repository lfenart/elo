use std::collections::HashMap;
use std::hash::Hash;

pub struct EloManager<Id> {
    players: HashMap<Id, Player>,
}

impl<Id> EloManager<Id> {
    const K: f32 = 60f32;
    const R: f32 = 400f32;

    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn expected_score(elo1: f32, elo2: f32) -> f32 {
        let delta = (elo2 - elo1).max(-Self::R).min(Self::R);
        1f32 / (1f32 + 10f32.powf(delta / Self::R))
    }

    pub fn players(&self) -> &HashMap<Id, Player> {
        &self.players
    }
}

impl<Id: Eq + Hash> EloManager<Id> {
    pub fn insert(&mut self, id: Id, player: Player) {
        self.players.insert(id, player);
    }
}

impl<Id: Eq + Hash + Clone> EloManager<Id> {
    pub fn process(&mut self, game: &Game<Id>) {
        let expected = Self::expected_score(
            self.mean_elo_insert(&game.team1),
            self.mean_elo_insert(&game.team2),
        );
        let score: f32 = game.score.into();
        let delta = Self::K * (score - expected);
        for player in &game.team1 {
            self.players.get_mut(player).unwrap().0 += delta;
        }
        for player in &game.team2 {
            self.players.get_mut(player).unwrap().0 -= delta;
        }
    }

    fn elo(&self, player: &Id) -> f32 {
        self.players
            .get(player)
            .map(|x| x.into())
            .unwrap_or(Player::DEFAULT_ELO)
    }

    pub fn mean_elo(&self, team: &[Id]) -> f32 {
        team.iter().map(|x| self.elo(x)).sum::<f32>() / team.len() as f32
    }

    fn elo_insert(&mut self, player: &Id) -> f32 {
        self.players
            .get(player)
            .map(|x| x.into())
            .unwrap_or_else(|| {
                self.players.insert(player.clone(), Player::new());
                Player::DEFAULT_ELO
            })
    }

    fn mean_elo_insert(&mut self, team: &[Id]) -> f32 {
        team.iter().map(|x| self.elo_insert(x)).sum::<f32>() / team.len() as f32
    }

    pub fn find_teams<'a>(&self, players: &'a [Id]) -> (Vec<&'a Id>, Vec<&'a Id>) {
        let mut best_teams = None;
        let mut best_score = f32::INFINITY;
        let mean_elo = self.mean_elo(players);
        for i in 0..(1 << players.len()) {
            if (i as u16).count_ones() as usize != players.len() / 2 {
                continue;
            }
            let mut elo = 0f32;
            for (j, player) in players.iter().enumerate() {
                if i & (1 << j) != 0 {
                    elo += self.elo(player);
                }
            }
            elo /= (players.len() / 2) as f32;
            let score = f32::abs(mean_elo - elo);
            if score < best_score {
                best_score = score;
                best_teams = Some(i);
            }
        }
        let best_teams = best_teams.unwrap();
        let mut teams = (Vec::new(), Vec::new());
        for (j, player) in players.iter().enumerate() {
            if best_teams & (1 << j) != 0 {
                teams.0.push(player);
            } else {
                teams.1.push(player);
            }
        }
        teams
    }
}

impl<Id: Clone + Eq + Hash> Default for EloManager<Id> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Game<Id> {
    team1: Vec<Id>,
    team2: Vec<Id>,
    score: Score,
}

impl<Id: Clone> Game<Id> {
    pub fn new(team1: Vec<Id>, team2: Vec<Id>, score: Score) -> Self {
        assert_eq!(team1.len(), team2.len(), "Teams should have the same size.");
        Self {
            team1,
            team2,
            score,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Player(f32);

impl Player {
    const DEFAULT_ELO: f32 = 2000f32;

    pub fn new() -> Self {
        Self::with_elo(Self::DEFAULT_ELO)
    }

    pub fn with_elo(elo: f32) -> Self {
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
            x => Err(format!("Cannot convert '{}' into Score.", x)),
        }
    }
}
