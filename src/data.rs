use uuid::Uuid;

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub tile: TileType,
    pub team: Uuid,
    pub hp: u32,
    pub am: u32,
    pub target_pos: Option<Position>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TileType {
    EMPTY,
    BASE,
    SPAWNER,
    FEEDER,
    BOLSTER,
    GUARD,
    ATTACK,
    QUEEN,
}

#[allow(non_camel_case_types)]
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Response {
    IDENTIFY {
        id: Uuid,
        origin: Position,
    },
    GENERATION_PING,
    FRAME {
        x_size: usize,
        y_size: usize,
        window: Vec<Vec<Unit>>
        /*
        [
            [{hp...}, {hp...}, {hp...}],
            [{hp...}, {hp...}, {hp...}],
            [{hp...}, {hp...}, {hp...}],
            [{hp...}, {hp...}, {hp...}],
        ]
        */
    }
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    NEW_PLAYER {
        username: String,
    },
    EXIT_GAME,
    REQUEST_FRAME {
        x_origin: usize,
        y_origin: usize,
        x_size: usize,
        y_size: usize,
    },
    INPUT {
        position: Position,
        unit: Unit,
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize, Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}