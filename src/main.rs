#![allow(unused)]
#![feature(vec_remove_item)]

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;
extern crate env_logger;

mod board;
mod msg;
mod server;
mod utils;

use board::*;
use uuid::Uuid;

fn main() -> ws::Result<()> {
    let mut board = board::Board::new();

    let team_1 = Uuid::new_v4();

    board.set(
        board::Position { x: 5, y: 5 },
        Unit::new_queen(team_1),
    );

    board.set(
        board::Position { x: 8, y: 8 },
        Unit {
            hp: 1,
            tile: TileType::Feeder,
            team: team_1,
        },
    );

    board.set(
        board::Position { x: 0, y: 9 },
        Unit {
            hp: 5,
            tile: TileType::Attacker,
            team: Uuid::new_v4(),
        },
    );

    for _ in 0..60 {
        board.next();
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!("{}", board);
        // board.get(board::Position { x: 5, y: 5 }).map(|u| println!("{}", u));
    }

    return Ok(());

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("ws::handler", log::LevelFilter::Info)
        .init();

    let mut server = server::Server::new();

    ws::listen("127.0.0.1:2794", |out| server.new_client(out));
}
