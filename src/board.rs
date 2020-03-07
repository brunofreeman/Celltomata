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

const X_SIZE: usize = 10;
const Y_SIZE: usize = 10;

use std::fmt::{Formatter, self, Write};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

#[derive(Clone)]
pub struct Board {
    grid: [[Option<Unit>; X_SIZE]; Y_SIZE],

    // Map between team UUID and positions of their cells.
    teams: HashMap<Uuid, HashSet<Position>>,

    // Map between the tiles and their positions.
    types: HashMap<TileType, HashSet<Position>>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { 
        let mut s = String::new();
        for y in 0..Y_SIZE {
            for x in 0..X_SIZE {
                if let Some(unit) = self.grid[y][x] {
                    write!(&mut s, "{}", unit);
                } else {
                    write!(&mut s, " ");
                }
            }
            s.push('\n');
        }
        fmt::Display::fmt(&s, f)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Unit {
    pub tile: TileType,
    pub team: Uuid,
    pub hp: usize,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.tile.fmt(f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum TileType {
    Base,
    Spawner,
    Feeder,
    Bolsterer,
    Guard,
    Attacker,
    Queen,
}

impl fmt::Display for TileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result { 
        match self {
            TileType::Base => "b",
            TileType::Spawner => "S",
            TileType::Feeder => "F",
            TileType::Bolsterer => "B",
            TileType::Guard => "G",
            TileType::Attacker => "A",
            TileType::Queen => "Q",
        }.fmt(f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Metadata {
    target_pos: Position,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
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
        Position { x, y }: Position,
        (dx, dy): (isize, isize),
    ) -> Option<Position> {
        if dx < 0 && x == 0
            || dy < 0 && y == 0
            || dx > 0 && x == X_SIZE - 1
            || dy > 0 && y == Y_SIZE - 1
        {
            None
        } else {
            let x = if dx < 0 {
                x - -dx as usize
            } else {
                x + dx as usize
            };

            let y = if dy < 0 {
                y - -dy as usize
            } else {
                y + dy as usize
            };

            Some(Position::new(x, y))
        }
    }

    fn get_neighbor(&self, position: Position, offset: (isize, isize)) -> Option<Option<Unit>> {
        self.get_neighbor_position(position, offset)
            .map(|Position { x, y }| self.grid[y][x])
    }

    pub fn get_unit(&self, Position { x, y }: Position) -> Option<Unit> {
        self.grid[y][x]
    }

    pub fn set_unit(&mut self, position: Position, unit: Unit) {
        self.delete_unit(position);
        self.grid[position.y][position.x] = Some(unit);
        get_mut_or_put(&mut self.types, unit.tile, || HashSet::new()).insert(position);
        get_mut_or_put(&mut self.teams, unit.team, || HashSet::new()).insert(position);
    }

    pub fn delete_unit(&mut self, position: Position) {
        self.get_unit(position).map(|unit| {
            self.types
                .get_mut(&unit.tile)
                .map(|set| set.remove(&position));
            self.teams
                .get_mut(&unit.team)
                .map(|set| set.remove(&position));
        });
        self.grid[position.y][position.x] = None;
    }

    pub fn next_gen(&mut self) {
        let mut new_board = self.clone();

        if let Some(vec) = self.types.get(&TileType::Queen) {
            for &queen_pos in vec {
                if let Some(base_pos) = self.find_nearest_unoccupied_position(queen_pos) {
                    new_board.set_unit(base_pos, Unit {
                        tile: TileType::Base,
                        team: Uuid::nil(),
                        hp: 3
                    })
                }
            }
        }

        *self = new_board;
    }

    fn find_nearest_unoccupied_position(&self, position: Position) -> Option<Position> {
        let mut queue = VecDeque::new();

        queue.push_back(position);

        while let Some(position) = queue.pop_front() {
            if self.get_unit(position) == None {
                return Some(position);
            }

            for &offset in ALL_OFFSETS.iter() {
                self.get_neighbor_position(position, offset)
                    .map(|p| queue.push_back(p));
            }
        }

        None
    }
}

fn get_mut_or_put<'m, K, V, F>(map: &'m mut HashMap<K, V>, k: K, f: F) -> &'m mut V
where
    F: FnOnce() -> V,
    K: Eq + std::hash::Hash + Copy,
{
    if !map.contains_key(&k) {
        map.insert(k, f());
    }
    map.get_mut(&k).unwrap()
}

/*
Queen will fill one cell as close to itself as possible with a base unit (equidistant is chosen randomly)
All units not within a friendly Feeder’s range loose 1hp due to Starvation, if farther away than 10 tiles from Queen or a Feeder, loose 3hp
Bolsterers increase HP
Spawners spawn
All units that want to move will attempt to move 1 tile towards their location
Attacking units deal damage. If both are in range, both deal damage.
*/
