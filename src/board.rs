use crate::data::{LeaderboardEntry, Position, TileType, Unit};
use crate::utils;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{self, Formatter, Write};
use uuid::Uuid;

use rand::seq::SliceRandom;
use rand::Rng;

use crate::constants;

#[derive(Clone)]
pub struct Board {
    grid: HashMap<Position, Unit>,

    // Map between team UUID and positions of their cells.
    teams: HashMap<Uuid, HashSet<Position>>,

    players: HashMap<Uuid, PlayerInformation>,

    // Map between the tiles and their positions.
    types: HashMap<TileType, HashSet<Position>>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for y in 0..constants::Y_SIZE {
            for x in 0..constants::X_SIZE {
                write!(&mut s, "{}", self.get(Position::new(x, y)));
            }
            s.push('\n');
        }
        fmt::Display::fmt(&s, f)
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.tile.fmt(f)
    }
}

impl Unit {
    const EMPTY: Unit = Unit {
        tile: TileType::EMPTY,
        team: Uuid::nil(),
        hp: 0,
        am: 0,
        target_pos: Position::new(0, 0),
    };

    pub fn new_queen(team_id: Uuid, position: Position) -> Unit {
        Unit {
            tile: TileType::QUEEN,
            team: team_id,
            hp: constants::MAX_HP,
            am: 0,
            target_pos: position,
        }
    }

    pub fn new_unit(id: Uuid, position: Position, tile: TileType) -> Unit {
        Unit {
            tile,
            team: id,
            hp: tile.get_base_hp(),
            am: 0,
            target_pos: position,
        }
    }

    pub fn spawn_unit(&self, position: Position, tile: TileType) -> Unit {
        if tile == TileType::EMPTY {
            Unit::EMPTY
        } else {
            Self::new_unit(self.team, position, tile)
        }
    }

    pub fn is_same_team_as(&self, other: Unit) -> bool {
        self.team == other.team
    }

    pub fn is_some(&self) -> bool {
        !self.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.tile == TileType::EMPTY
    }
}

impl fmt::Display for TileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TileType::EMPTY => " ",
            TileType::BASE => "b",
            TileType::SPAWNER => "S",
            TileType::FEEDER => "F",
            TileType::BOLSTER => "B",
            TileType::GUARD { .. } => "G",
            TileType::ATTACK { .. } => "A",
            TileType::QUEEN => "Q",
        }
        .fmt(f)
    }
}

#[derive(Clone)]
pub struct PlayerInformation {
    pub id: Uuid,
    pub name: Option<String>,
    pub energy: u32,
}

impl Board {
    pub fn new() -> Self {
        Self {
            grid: HashMap::new(),
            teams: HashMap::new(),
            types: HashMap::new(),
            players: HashMap::new(),
        }
    }

    fn adj_position(
        &self,
        Position { x, y }: Position,
        (dx, dy): (isize, isize),
    ) -> Option<Position> {
        if dx < 0 && x == 0
            || dy < 0 && y == 0
            || dx > 0 && x == constants::X_SIZE - 1
            || dy > 0 && y == constants::Y_SIZE - 1
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
        constants::ALL_OFFSETS
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
        self.adj_position(position, offset).map(|pos| self.get(pos))
    }

    pub fn get(&self, pos: Position) -> Unit {
        self.grid.get(&pos).copied().unwrap_or(Unit::EMPTY)
    }

    pub fn get_mut(&mut self, pos: Position) -> &mut Unit {
        utils::get_mut_or_put(&mut self.grid, pos, || Unit::EMPTY)
    }

    pub fn set(&mut self, position: Position, unit: Unit) {
        self.delete(position);
        utils::get_mut_or_put(&mut self.types, unit.tile, || HashSet::new()).insert(position);
        utils::get_mut_or_put(&mut self.teams, unit.team, || HashSet::new()).insert(position);
        self.grid.insert(position, unit);
    }

    pub fn delete(&mut self, position: Position) -> Unit {
        let unit = self.get(position);
        self.types
            .get_mut(&unit.tile)
            .map(|set| set.remove(&position));
        self.teams
            .get_mut(&unit.team)
            .map(|set| set.remove(&position));
        self.grid.remove(&position);
        unit
    }

    pub fn next(&mut self) {
        self.queen_gen();
        self.feeder_gen();
        self.bolster_gen();
        self.spawner_gen();
        self.guard_gen();
        self.attacker_gen();
    }

    fn queen_gen(&mut self) {
        let mut new_board = self.clone();

        // Queen will fill one cell as close to itself as possible with a base unit (equidistant is chosen randomly)
        // if let Some(vec) = self.types.get(&TileType::QUEEN) {
        self.types.get(&TileType::QUEEN).map(|vec| {
            vec.iter().for_each(|&queen_pos| {
                if let Some(base_pos) = self.nearest_unoccupied_position(queen_pos, 1) {
                    new_board.set(
                        base_pos,
                        self.get(queen_pos).spawn_unit(base_pos, TileType::BASE),
                    )
                }
            });
        });

        *self = new_board;
    }

    pub fn get_player(&self, id: Uuid) -> Option<&PlayerInformation> {
        self.players.get(&id)
    }

    pub fn get_player_mut(&mut self, id: Uuid) -> Option<&mut PlayerInformation> {
        self.players.get_mut(&id)
    }

    pub fn add_player(&mut self, player: PlayerInformation) {
        self.players.insert(player.id, player);
    }

    pub fn remove_player(&mut self, id: Uuid) {
        let mut new_board = self.clone();
        self.teams.get(&id).map(|set| {
            set.iter().for_each(|&pos| {
                new_board.delete(pos);
            })
        });
        self.players.remove(&id);
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
                .filter(|&&pos| !self.within_friendly_range(pos, TileType::FEEDER, 5))
                .filter(|&&pos| self.get(pos).tile != TileType::FEEDER)
            {
                let unit = new_board.get_mut(pos);
                unit.hp = unit.hp.saturating_sub(1);
                if unit.hp == 0 {
                    if unit.tile == TileType::QUEEN {
                        let team = unit.team;
                        new_board.remove_player(team);
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
        self.teams.iter().for_each(|(team_id, list)| {
            list.iter()
                .filter(|&&pos| self.within_friendly_range(pos, TileType::BOLSTER, 3))
                .for_each(|&pos| {
                    let unit = new_board.get_mut(pos);
                    unit.am = unit.am.saturating_add(1);
                    if unit.am > constants::MAX_AM {
                        unit.am = constants::MAX_AM;
                    }
                });
        });

        *self = new_board;
    }

    fn spawner_gen(&mut self) {
        let mut new_board = self.clone();
        let mut rng = rand::thread_rng();

        self.types.get(&TileType::SPAWNER).map(|vec| {
            vec.iter().for_each(|&spawner_pos| {
                if let Some(unit_pos) = self.nearest_unoccupied_position(spawner_pos, 5) {
                    // new_board.set(base_pos, self.get(spawner_pos).spawn_base())
                    let tile = match rng.gen_range(0, 100) {
                        0..=94 => TileType::BASE,
                        95 => TileType::ATTACK,
                        96 => TileType::SPAWNER,
                        97 => TileType::FEEDER,
                        98 => TileType::BOLSTER,
                        99 => TileType::GUARD,
                        _ => unreachable!(),
                    };
                    new_board.set(unit_pos, self.get(spawner_pos).spawn_unit(unit_pos, tile))
                }
            });
        });

        *self = new_board;
    }

    fn guard_gen(&mut self) {
        let mut new_board = self.clone();

        if let Some(vec) = self.types.get(&TileType::GUARD) {
            for &guard_pos in vec {
                if let Some(enemy_pos) = self.nearest_enemy_position(guard_pos, 3) {
                    if self.is_adj_position(guard_pos, enemy_pos) {
                        let target_unit = new_board.get_mut(enemy_pos);

                        if target_unit.am != 0 {
                            if target_unit.am < constants::ATK_DMG {
                                target_unit.hp = target_unit
                                    .hp
                                    .saturating_sub(constants::GRD_DMG - target_unit.am);
                                target_unit.am = 0;
                            } else {
                                target_unit.am = target_unit.am.saturating_sub(constants::GRD_DMG);
                            }
                        } else {
                            target_unit.hp = target_unit.hp.saturating_sub(constants::GRD_DMG);
                        }

                        if target_unit.hp == 0 {
                            if target_unit.tile == TileType::QUEEN {
                                let team = target_unit.team;
                                new_board.remove_player(team);
                            }
                            new_board.move_unit(guard_pos, enemy_pos);
                        }
                    } else {
                        let target = self.adj_position_towards(guard_pos, enemy_pos);
                        new_board.move_unit(guard_pos, target);
                    }
                } else {
                    let target_pos = self.get(guard_pos).target_pos;
                    if guard_pos != target_pos {
                        let target = self.adj_position_towards(guard_pos, target_pos);
                        if self.get(target).is_empty() {
                            new_board.move_unit(guard_pos, target);
                        }
                    }
                }
            }
        }

        *self = new_board;
    }

    fn attacker_gen(&mut self) {
        let mut new_board = self.clone();

        if let Some(vec) = self.types.get(&TileType::ATTACK) {
            for &attacker_pos in vec {
                if let Some(enemy_pos) = self.nearest_enemy_position(attacker_pos, 5) {
                    if self.is_adj_position(attacker_pos, enemy_pos) {
                        let target_unit = new_board.get_mut(enemy_pos);

                        if target_unit.am != 0 {
                            if target_unit.am < constants::ATK_DMG {
                                target_unit.hp = target_unit
                                    .hp
                                    .saturating_sub(constants::ATK_DMG - target_unit.am);
                                target_unit.am = 0;
                            } else {
                                target_unit.am = target_unit.am.saturating_sub(constants::ATK_DMG);
                            }
                        } else {
                            target_unit.hp = target_unit.hp.saturating_sub(constants::ATK_DMG);
                        }

                        if target_unit.hp == 0 {
                            if target_unit.tile == TileType::QUEEN {
                                let team = target_unit.team;
                                new_board.remove_player(team);
                            }
                            new_board.move_unit(attacker_pos, enemy_pos);
                        }
                    } else {
                        let target = self.adj_position_towards(attacker_pos, enemy_pos);
                        if self.get(target).is_empty() {
                            new_board.move_unit(attacker_pos, target);
                        }
                    }
                }
            }
        }

        *self = new_board;
    }

    pub fn find_random_safe_position(&self, distance: usize) -> Option<Position> {
        let mut rng = rand::thread_rng();

        for _ in 0..50 {
            let x = rng.gen_range(0, constants::X_SIZE);
            let y = rng.gen_range(0, constants::Y_SIZE);

            if x <= distance
                || x >= constants::X_SIZE - distance
                || y <= distance
                || y >= constants::Y_SIZE - distance
            {
                continue;
            }

            let position = Position::new(x, y);

            if self
                .bfs(position, distance as u16, false, |p| self.get(p).is_some())
                .is_none()
            {
                return Some(position);
            }
        }
        None
    }

    // BFS the grid
    fn nearest_unoccupied_position(&self, position: Position, max_depth: u16) -> Option<Position> {
        self.bfs(position, max_depth, true, |pos| self.get(pos).is_empty())
    }

    fn nearest_enemy_position(&self, position: Position, max_depth: u16) -> Option<Position> {
        let unit = self.get(position);
        if unit.is_empty() {
            None
        } else {
            self.bfs(position, max_depth, true, |pos| {
                let target = self.get(pos);
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

    pub fn get_leaderboard(&self) -> Vec<LeaderboardEntry> {

        let mut teams = self.teams.iter().collect::<Vec<_>>();
        teams.sort_unstable_by_key(|(_, set)| set.len());

        teams.iter().take(5).map(|(&id, set)| {
            LeaderboardEntry {
                name: self.get_player(id).and_then(|player| player.name.clone()),
                score: set.len(),
            }
        }).collect::<Vec<_>>()
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

            let mut dirs = constants::ALL_OFFSETS;
            if randomized {
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

    pub fn get_window(
        &self,
        x_origin: usize,
        y_origin: usize,
        x_size: usize,
        y_size: usize,
    ) -> Vec<Vec<Unit>> {
        let x_min = x_origin.min(constants::X_SIZE);
        let x_max = (x_origin + x_size).min(constants::X_SIZE);
        let y_min = y_origin.min(constants::Y_SIZE);
        let y_max = (y_origin + y_size).min(constants::Y_SIZE);
        let mut vec = Vec::with_capacity(y_max - y_min);
        for y in y_min..y_max {
            let mut inner_vec = Vec::with_capacity(x_max - x_min);
            for x in x_min..x_max {
                inner_vec.push(self.get(Position::new(x, y)))
            }
            vec.push(inner_vec);
        }
        vec
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
