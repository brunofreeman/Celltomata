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
mod data;
mod server;
mod utils;

use board::*;
use uuid::Uuid;
use data::{TileType, Unit, Position};

fn main() -> ws::Result<()> {
    // let mut board = board::Board::new();

    // let team_1 = Uuid::new_v4();

    // board.set(
    //     Position { x: 5, y: 5 },
    //     Unit::new_queen(team_1),
    // );

    // board.set(
    //     Position { x: 8, y: 8 },
    //     Unit {
    //         hp: 1,
    //         tile: TileType::FEEDER,
    //         team: team_1,
    //         ..Default::default()
    //     },
    // );
    // board.set(
    //     Position { x: 4, y: 6 },
    //     Unit {
    //         hp: 600,
    //         tile: TileType::GUARD,
    //         team: team_1,
    //         ..Default::default()
    //     },
    // );
    // board.set(
    //     Position { x: 8, y: 7 },
    //     Unit {
    //         hp: 1,
    //         tile: TileType::BOLSTER,
    //         team: team_1,
    //         ..Default::default()
    //     },
    // );

    // let team_2 = Uuid::new_v4();

    // board.set(
    //     Position { x: 0, y: 9 },
    //     Unit {
    //         hp: 10,
    //         tile: TileType::ATTACK,
    //         team: team_2,
    //         ..Default::default()
    //     },
    // );

    // board.set(
    //     Position { x: 1, y: 9 },
    //     Unit {
    //         hp: 10,
    //         tile: TileType::ATTACK,
    //         team: team_2,
    //         ..Default::default()
    //     },
    // );
    

    // for _ in 0..50 {
    //     board.next();
    //     std::thread::sleep(std::time::Duration::from_millis(200));
    //     println!("{}", board);
    //     // board.get(board::Position { x: 5, y: 5 }).map(|u| println!("{}", u));
    // }

    // let window = board.get_window(5, 5, 5, 5);

    // let mut s = String::new();
    // for row in window {
    //     for col in row {
    //         print!("{}", col);
    //     }
    //     println!();
    // }
    // println!("{}", s);
    

    // return Ok(());

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("ws::handler", log::LevelFilter::Info)
        .init();

    let mut server = server::Server::new();

    ws::listen("127.0.0.1:2794", |out| server.new_client(out))
}
