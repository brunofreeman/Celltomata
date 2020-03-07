#![allow(unused)]

static ALL_OFFSETS: [(isize, isize); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];
static ORTH_OFFSETS: [(isize, isize); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

const X_SIZE: usize = 100;
const Y_SIZE: usize = 100;

use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

pub struct Board {
    grid: [[Option<Unit>; X_SIZE]; Y_SIZE],

    // Map between team UUID and positions of their cells.
    teams: HashMap<Uuid, Vec<Position>>,

    // Map between the tiles and their positions.
    types: HashMap<TileType, Vec<Position>>,
}

#[derive(Copy, Clone)]
pub struct Unit {
    tile: TileType,
    team: Uuid,
    hp: usize,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum TileType {
    Base,
    Spawner,
    Feeder,
    Bolsterer,
    Guard,
    Attacker,
    Queen,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Metadata {
    target_pos: Position,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Position {
    x: usize,
    y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            grid: [[None; X_SIZE]; Y_SIZE],
            teams: HashMap::new(),
            types: HashMap::new(),
        }
    }

    fn get_neighbor_position(
        &self,
        position: Position,
        offset: (isize, isize),
    ) -> Option<Position> {
        if offset.0 < 0 && position.x == 0
            || offset.1 < 0 && position.y == 0
            || offset.0 > 0 && position.x == X_SIZE - 1
            || offset.1 > 0 && position.y == Y_SIZE - 1
        {
            None
        } else {
            let x = if offset.0 < 0 {
                position.x - offset.0 as usize
            } else {
                position.x + offset.0 as usize
            };

            let y = if offset.1 < 0 {
                position.y - offset.1 as usize
            } else {
                position.y + offset.1 as usize
            };

            Some(Position::new(x, y))
        }
    }

    fn get_neighbor(&self, position: Position, offset: (isize, isize)) -> Option<Option<Unit>> {
        self.get_neighbor_position(position, offset)
            .map(|Position { x, y }| self.grid[x][y])
    }

    pub fn next_gen(&self) -> Self {
        let mut board = Board::new();

        use rand::{thread_rng, Rng};
        rand::thread_rng().gen_range(0, 5);

        // self.types.get(&TileType::Queen);
        unimplemented!()
    }

    fn find_nearest_unoccupied_cell(&self, position: Position) {
        let mut queue = VecDeque::new();

        queue.push_back(position);
        unimplemented!()
    }
}

/*
Queen will fill one cell as close to itself as possible with a base unit (equidistant is chosen randomly)
All units not within a friendly Feeder’s range loose 1hp due to Starvation, if farther away than 10 tiles from Queen or a Feeder, loose 3hp
Bolsterers increase HP
Spawners spawn
All units that want to move will attempt to move 1 tile towards their location
Attacking units deal damage. If both are in range, both deal damage.
*/
