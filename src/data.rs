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
    },
    FRAME {
        x_origin: usize,
        y_origin: usize,
        x_size: usize,
        y_size: usize,
        window: Vec<Vec<Unit>>
    }
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Request {
    EXIT_GAME,
    REQUEST_FRAME {
        x_origin: usize,
        y_origin: usize,
        x_size: usize,
        y_size: usize,
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}