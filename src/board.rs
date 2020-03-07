use crate::utils;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{self, Formatter, Write};
use uuid::Uuid;

static ALL_OFFSETS: [(isize, isize); 8] = [
    (-1, 0),
    (0, -1),
    (1, 0),
    (0, 1),
    (-1, -1),
    (1, -1),
    (1, 1),
    (-1, 1),
];
// static ORTH_OFFSETS: [(isize, isize); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

const X_SIZE: usize = 10;
const Y_SIZE: usize = 10;

const MAX_HEALTH: usize = 10;

#[derive(Clone)]
pub struct Board {
    grid: [[Unit; X_SIZE]; Y_SIZE],

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
                write!(&mut s, "{}", self.grid[y][x]);
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

impl Unit {
    const EMPTY: Unit = Unit {
        tile: TileType::Empty,
        team: Uuid::nil(),
        hp: 0,
    };

    pub fn new_queen(team_id: Uuid) -> Unit {
        Unit {
            tile: TileType::Queen,
            team: team_id,
            hp: 10,
        }
    }

    pub fn spawn_base(&self) -> Unit {
        self.spawn_unit(TileType::Base, 3)
    }

    pub fn spawn_unit(&self, tile: TileType, hp: usize) -> Unit {
        Unit {
            tile,
            team: self.team,
            hp,
        }
    }

    pub fn is_same_team_as(&self, other: Unit) -> bool {
        self.team == other.team
    }

    pub fn is_some(&self) -> bool {
        !self.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.tile == TileType::Empty
    }
}

#[serde(tag = "tile_type")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Empty,
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
            TileType::Empty => " ",
            TileType::Base => "b",
            TileType::Spawner => "S",
            TileType::Feeder => "F",
            TileType::Bolsterer => "B",
            TileType::Guard { .. } => "G",
            TileType::Attacker { .. } => "A",
            TileType::Queen => "Q",
        }
        .fmt(f)
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
            grid: [[Unit::EMPTY; X_SIZE]; Y_SIZE],
            teams: HashMap::new(),
            types: HashMap::new(),
        }
    }

    fn adj_position(
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

    fn is_adj_position(&self, origin: Position, target: Position) -> bool {
        ALL_OFFSETS
            .iter()
            .filter_map(|&offset| self.adj_position(origin, offset))
            .any(|pos| pos == target)
    }

    fn adj_position_towards(&self, origin: Position, target: Position) -> Position {
        let dx: isize = if target.x < origin.x {
            -1
        } else if target.x == origin.x {
            0
        } else {
            1
        };
        let dy: isize = if target.y < origin.y {
            -1
        } else if target.y == origin.y {
            0
        } else {
            1
        };
        self.adj_position(origin, (dx, dy)).unwrap()
    }

    fn move_unit(&mut self, origin: Position, target: Position) {
        let unit = self.delete(origin);
        self.set(target, unit);
    }

    fn adj_unit(&self, position: Position, offset: (isize, isize)) -> Option<Unit> {
        self.adj_position(position, offset)
            .map(|Position { x, y }| self.grid[y][x])
    }

    pub fn get(&self, Position { x, y }: Position) -> Unit {
        self.grid[y][x]
    }

    pub fn get_ref(&self, Position { x, y }: Position) -> &Unit {
        &self.grid[y][x]
    }

    pub fn get_mut(&mut self, Position { x, y }: Position) -> &mut Unit {
        &mut self.grid[y][x]
    }

    pub fn set(&mut self, position: Position, unit: Unit) {
        self.delete(position);
        utils::get_mut_or_put(&mut self.types, unit.tile, || HashSet::new()).insert(position);
        utils::get_mut_or_put(&mut self.teams, unit.team, || HashSet::new()).insert(position);
        self.grid[position.y][position.x] = unit;
    }

    pub fn delete(&mut self, position: Position) -> Unit {
        let unit = self.get(position);
        self.types
            .get_mut(&unit.tile)
            .map(|set| set.remove(&position));
        self.teams
            .get_mut(&unit.team)
            .map(|set| set.remove(&position));
        self.grid[position.y][position.x] = Unit::EMPTY;
        unit
    }

    pub fn next(&mut self) {
        self.queen_gen();
        self.feeder_gen();
        self.bolster_gen();
        self.attacker_gen();
    }

    fn queen_gen(&mut self) {
        let mut new_board = self.clone();

        // Queen will fill one cell as close to itself as possible with a base unit (equidistant is chosen randomly)
        if let Some(vec) = self.types.get(&TileType::Queen) {
            for &queen_pos in vec {
                if let Some(base_pos) = self.nearest_unoccupied_position(queen_pos, 5) {
                    new_board.set(base_pos, self.get(queen_pos).spawn_base())
                }
            }
        }

        *self = new_board;
    }

    fn kill_team(&mut self, id: Uuid) {
        let mut new_board = self.clone();
        self.teams.get(&id).map(|set| {
            set.iter().for_each(|&pos| {
                new_board.delete(pos);
            })
        });
        new_board.teams.remove(&id);
        *self = new_board;
    }

    fn feeder_gen(&mut self) {
        let mut new_board = self.clone();

        // All units not within a friendly Feeder’s range loose 1hp due to Starvation,
        // if farther away than 10 tiles from Queen or a Feeder, loose 3hp
        'z: for (team_id, list) in &self.teams {
            for &pos in list
                .iter()
                .filter(|&&pos| !self.within_friendly_range(pos, TileType::Feeder, 5))
                .filter(|&&pos| self.get(pos).tile != TileType::Feeder)
            {
                let unit = new_board.get_mut(pos);
                unit.hp = unit.hp.saturating_sub(1);
                if unit.hp == 0 {
                    if unit.tile == TileType::Queen {
                        let team = unit.team;
                        new_board.kill_team(team);
                        continue 'z;
                    } else {
                        new_board.delete(pos);
                    }
                }
            }
        }

        // Spawners spawn

        // Attacker move and lock in

        *self = new_board;
    }

    fn bolster_gen(&mut self) {
        let mut new_board = self.clone();
        // TODO: ARMOR instead of HP?
        for (team_id, list) in &self.teams {
            for &pos in list
                .iter()
                .filter(|&&pos| self.within_friendly_range(pos, TileType::Bolsterer, 5))
            {
                let unit = new_board.get_mut(pos);
                unit.hp = unit.hp.saturating_add(1);
                if unit.hp > MAX_HEALTH {
                    unit.hp = MAX_HEALTH;
                }
            }
        }
        *self = new_board;
    }

    fn attacker_gen(&mut self) {
        let mut new_board = self.clone();

        if let Some(vec) = self.types.get(&TileType::Attacker) {
            for &attacker_pos in vec {
                if let Some(enemy_pos) = self.nearest_enemy_position(attacker_pos, 5) {
                    if self.is_adj_position(attacker_pos, enemy_pos) {
                        let target_unit = new_board.get_mut(enemy_pos);
                        target_unit.hp = target_unit.hp.saturating_sub(4);
                        if target_unit.hp == 0 {
                            if target_unit.tile == TileType::Queen {
                                let team = target_unit.team;
                                new_board.kill_team(team);
                                // continue 'z;
                            }
                            new_board.move_unit(attacker_pos, enemy_pos);
                        }
                    } else {
                        let target = self.adj_position_towards(attacker_pos, enemy_pos);
                        new_board.move_unit(attacker_pos, target);
                    }
                }
            }
        }

        *self = new_board;
    }

    // BFS the grid
    fn nearest_unoccupied_position(&self, position: Position, max_depth: u16) -> Option<Position> {
        self.bfs(position, max_depth, true, |pos| {
            self.get_ref(pos).is_empty()
        })
    }

    fn nearest_enemy_position(&self, position: Position, max_depth: u16) -> Option<Position> {
        let unit = self.get(position);
        if unit.is_empty() {
            None
        } else {
            self.bfs(position, max_depth, true, |pos| {
                let target = self.get_ref(pos);
                target.is_some() && !target.is_same_team_as(unit)
            })
        }
    }

    fn within_friendly_range(&self, position: Position, tile: TileType, max_depth: u16) -> bool {
        let unit = self.get(position);
        if unit.is_empty() {
            false
        } else {
            self.bfs(position, max_depth, false, |pos| self.get(pos).tile == tile)
                .map_or(false, |fpos| self.get(fpos).is_same_team_as(unit))
        }
    }

    #[inline]
    fn bfs<F>(
        &self,
        position: Position,
        max_depth: u16,
        randomized: bool,
        predicate: F,
    ) -> Option<Position>
    where
        F: Fn(Position) -> bool,
    {
        let mut queue = VecDeque::new();
        let mut seen = HashSet::new();

        queue.push_back((position, 0u16));

        while let Some((position, depth)) = queue.pop_front() {
            if predicate(position) {
                return Some(position);
            } else if (depth >= max_depth) {
                continue;
            }

            let mut dirs = ALL_OFFSETS;
            if randomized {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                dirs[0..4].shuffle(&mut rng);
                dirs[4..8].shuffle(&mut rng);
            }

            for &offset in dirs.iter() {
                self.adj_position(position, offset).map(|p| {
                    if !seen.contains(&p) {
                        queue.push_back((p, depth + 1));
                        seen.insert(p);
                    }
                });
            }
        }

        None
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
