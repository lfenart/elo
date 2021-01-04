use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};

use elo::*;

fn main() -> std::io::Result<()> {
    let mut elo_manager = EloManager::new();
    {
        let file = File::open("data/initial.csv")?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let values = line.split(',').into_iter().collect::<Vec<&str>>();
            let player = values[0].to_string();
            let elo = values[1].parse::<f32>().unwrap();
            elo_manager.insert(player, elo.into());
        }
    }
    let scores = {
        let mut scores = BTreeMap::new();
        let file = File::open("data/scores.csv")?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') {
                continue;
            }
            let values = line.split(',').into_iter().collect::<Vec<&str>>();
            let id = values[0].parse::<u16>().unwrap();
            let score = Score::try_from(values[1].chars().next().unwrap()).unwrap();
            scores.insert(id, score);
        }
        scores
    };
    let teams = {
        let mut teams = HashMap::new();
        let file = File::open("data/games.csv")?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') {
                continue;
            }
            let values = line.split(',').into_iter().collect::<Vec<&str>>();
            let id = values[0].parse::<u16>().unwrap();
            let team = values[1].parse::<u16>().unwrap();
            let player = values[2].to_string();
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
    };
    for (id, score) in &scores {
        let (team1, team2) = teams.get(&id).unwrap();
        let game = Game::new(&team1, &team2, *score);
        elo_manager.process(&game);
    }
    println!("{} games analyzed.", scores.len());
    let mut players = elo_manager.players().iter().collect::<Vec<_>>();
    players.sort_unstable_by(|(_, a), (_, b)| b.partial_cmp(&a).unwrap());
    let mut elo_file = File::create("data/elo.csv")?;
    for (i, (id, player)) in players.into_iter().enumerate() {
        elo_file.write_all(
            format!("{},{},{}\n", i + 1, id, f32::round((*player).into()) as u16).as_bytes(),
        )?;
    }
    Ok(())
}
