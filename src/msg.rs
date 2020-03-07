use uuid::Uuid;

#[allow(non_camel_case_types)]
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Response {
    IDENTIFY {
		id: Uuid,
	}
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Request {
    EXIT_GAME
}