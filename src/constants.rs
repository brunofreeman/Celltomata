pub static ALL_OFFSETS: [(isize, isize); 8] = [
    (-1, 0),
    (0, -1),
    (1, 0),
    (0, 1),
    (-1, -1),
    (1, -1),
    (1, 1),
    (-1, 1),
];

pub const X_SIZE: usize = 500;
pub const Y_SIZE: usize = 500;

pub const INIT_ERG: u32 = 2020;

pub const MAX_HP: u32 = 8;
pub const MAX_AM: u32 = 8;
pub const ATK_DMG: u32 = 4;
pub const GRD_DMG: u32 = 3;

use crate::data::TileType;
use std::u32;

impl TileType {
    pub fn get_base_hp(self) -> u32 {
        match self {
            TileType::EMPTY => 0,
            TileType::BASE => 3,
            TileType::SPAWNER => 3,
            TileType::FEEDER => 4,
            TileType::BOLSTER => 1,
            TileType::GUARD => 10,
            TileType::ATTACK => 6,
            TileType::QUEEN => 10,
        }
    }

    pub fn get_cost(self) -> u32 {
        match self {
            TileType::EMPTY => u32::MAX,
            TileType::BASE => 100,
            TileType::SPAWNER => 750,
            TileType::FEEDER => 325,
            TileType::BOLSTER => 500,
            TileType::GUARD => 650,
            TileType::ATTACK => 725,
            TileType::QUEEN => u32::MAX,
        }
    }
}
