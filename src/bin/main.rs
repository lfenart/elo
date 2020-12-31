use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};

use elo::*;

fn main() -> std::io::Result<()> {
    let file = File::open("data/initial.csv")?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let values = line.split(',').into_iter().collect::<Vec<&str>>();
        let player = values[0].to_string();
        let elo = values[1].parse::<f32>().unwrap();
        unsafe { PLAYERS.get() }.insert(player, Player::with_elo(elo));
    }
    let file = File::open("data/games.csv")?;
    let reader = BufReader::new(file);
    let mut games = Vec::new();
    let mut players = Vec::new();
    let mut game_id = 0;
    let mut result = 0f32;
    for line in reader.lines().skip(1) {
        let line = line?;
        let values = line.split(',').into_iter().collect::<Vec<&str>>();
        let id = values[0].parse::<u16>().unwrap();
        let player = values[3].to_string();
        if id != game_id {
            if game_id != 0 {
                games.push(Game::new(players, result));
                players = Vec::new();
            }
            game_id = id;
            result = match values[2] {
                "W" => 1f32,
                "L" => 0f32,
                "D" => 0.5f32,
                _ => panic!(),
            };
        }
        players.push(player);
    }
    games.push(Game::new(players, result));
    println!("{} games analyzed.", games.len());
    for game in games {
        game.process();
    }
    let mut players = unsafe { PLAYERS.get() }.iter().collect::<Vec<_>>();
    players.sort_unstable_by(|(_, a), (_, b)| b.elo().partial_cmp(&a.elo()).unwrap());
    let mut elo_file = File::create("data/elo.csv")?;
    for (i, (id, player)) in players.iter().enumerate() {
        elo_file
            .write_all(format!("{},{},{}\n", i + 1, id, player.elo().round() as u16).as_bytes())?;
    }
    Ok(())
}
