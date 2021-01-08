use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};

use elo::*;

type Id = String;

fn parse_initial(elo_manager: &mut EloManager<Id>) {
    if let Ok(file) = File::open("data/initial.csv") {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let values = line.split(',').into_iter().collect::<Vec<&str>>();
            let player = values[0].parse::<Id>().unwrap();
            let elo = values[1].parse::<f32>().unwrap();
            elo_manager.insert(player, Player::with_elo(elo));
        }
    }
}

fn parse_scores() -> BTreeMap<usize, Score> {
    let mut scores = BTreeMap::new();
    let file = File::open("data/scores.csv").unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with('#') {
            continue;
        }
        let values = line.split(',').into_iter().collect::<Vec<&str>>();
        let id = values[0].parse::<usize>().unwrap();
        let score = Score::try_from(values[1].chars().next().unwrap()).unwrap();
        scores.insert(id, score);
    }
    scores
}

fn parse_teams() -> HashMap<usize, (Vec<Id>, Vec<Id>)> {
    let mut teams = HashMap::new();
    let file = File::open("data/games.csv").unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with('#') {
            continue;
        }
        let values = line.split(',').into_iter().collect::<Vec<&str>>();
        let id = values[0].parse::<usize>().unwrap();
        let team = values[1].parse::<usize>().unwrap();
        let player = values[2].parse::<Id>().unwrap();
        if teams.get(&id).is_none() {
            teams.insert(id, (Vec::new(), Vec::new()));
        }
        if team == 1 {
            teams.get_mut(&id).unwrap().0.push(player);
        } else {
            teams.get_mut(&id).unwrap().1.push(player);
        }
    }
    teams
}

fn process_games(
    elo_manager: &mut EloManager<Id>,
    teams: &HashMap<usize, (Vec<Id>, Vec<Id>)>,
    scores: &BTreeMap<usize, Score>,
) {
    for (id, score) in scores {
        let (team1, team2) = teams.get(&id).unwrap();
        let game = Game::new(team1.clone(), team2.clone(), *score);
        elo_manager.process(&game);
    }
}

fn save_ratings(elo_manager: &EloManager<Id>) {
    let mut players = elo_manager.players().iter().collect::<Vec<_>>();
    players.sort_unstable_by(|(_, a), (_, b)| b.partial_cmp(&a).unwrap());
    let mut elo_file = File::create("data/elo.csv").unwrap();
    for (i, (id, player)) in players.into_iter().enumerate() {
        elo_file
            .write_all(
                format!("{},{},{}\n", i + 1, id, f32::from(player).round() as u16).as_bytes(),
            )
            .unwrap();
    }
}

fn predict_teams(elo_manager: &EloManager<Id>) -> Option<(Vec<Id>, Vec<Id>, f32)> {
    let players = {
        let mut players = Vec::new();
        let file = File::open("data/players.csv").ok()?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.ok()?;
            if line.starts_with('#') {
                continue;
            }
            players.push(line.parse::<Id>().unwrap());
        }
        players
    };
    if !players.is_empty() {
        let teams = elo_manager.find_teams(&players);
        let team1 = teams.0.iter().map(|&x| x.clone()).collect::<Vec<_>>();
        let team2 = teams.1.iter().map(|&x| x.clone()).collect::<Vec<_>>();
        let expected = EloManager::<Id>::expected_score(
            elo_manager.mean_elo(&team1),
            elo_manager.mean_elo(&team2),
        );
        return Some((team1, team2, expected));
    }
    None
}

fn main() -> std::io::Result<()> {
    let mut elo_manager = EloManager::new();
    parse_initial(&mut elo_manager);
    let scores = parse_scores();
    let teams = parse_teams();
    process_games(&mut elo_manager, &teams, &scores);
    println!("{} games analyzed.", scores.len());
    save_ratings(&elo_manager);
    if let Some(prediction) = predict_teams(&elo_manager) {
        println!("Team 1: {:?}", prediction.0);
        println!("Team 2: {:?}", prediction.1);
        println!("Expected: {}", prediction.2);
    }
    Ok(())
}
