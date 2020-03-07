#![allow(unused)]
#![feature(vec_remove_item)]

mod board;
use board::*;
use uuid::Uuid;

fn main() {
    let mut board = board::Board::new();

    board.set(board::Position { x: 5, y: 5 }, Unit { hp: 1, tile: TileType::Queen, team: Uuid::nil() });
    for _ in 0..60 {
        board.next();
        std::thread::sleep_ms(50);
        println!("{}", board);
    }
}
