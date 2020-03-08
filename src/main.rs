#![allow(unused)]

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
mod constants;

use board::*;
use data::{Position, TileType, Unit};
use server::Server;
use uuid::Uuid;

fn main() -> ws::Result<()> {
    // let mut board = board::Board::new();

    // let team_1 = Uuid::new_v4();

    // board.set(Position { x: 5, y: 5 }, Unit::new_queen(team_1));

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
        .filter_level(log::LevelFilter::Info)
        .filter_module("ws::handler", log::LevelFilter::Info)
        .init();

    let mut server = Server::new();

    let mut arcserver = Arc::new(server);
    make_game_thread(arcserver.clone());

    ws::listen("127.0.0.1:2794", |out| Server::new_client(arcserver.clone(), out))
}

use crate::data::{Request, Response};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

fn make_game_thread(server: Arc<Server>) {
    let board = server.board.clone();
    let running = server.running.clone();

    let handle = std::thread::spawn(move || {
        let mut gen: usize = 0;
        while running.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_secs(1));
            gen += 1;

            let start = Instant::now();

            board.write().map(|mut board| {
                board.next();
                
                if gen % 20 == 0 {
                    server.broadcast(&Response::LEADERBOARD_UPDATE {
                        leaderboard: board.get_leaderboard()
                    });
                    info!("Broadcasting leaderboards...");
                }
            });

            let elapsed = start.elapsed();

            server.broadcast(&Response::GENERATION_PING { gen });

            info!("Generation {} generated in {} ms ({} ns)", gen, elapsed.as_millis(), elapsed.as_nanos());
        }
        info!("Done.");
    });
    // handle
}
