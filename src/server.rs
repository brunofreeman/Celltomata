use dashmap::DashMap;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use uuid::Uuid;
use ws;

use crate::board::*;
use crate::data::{Position, TileType, Unit};
use crate::data::{Request, Response};
use crate::server;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

pub const PROTOCOL: &'static str = "game-of-strife";

// todo how to not use raw pointers
pub struct Server {
    pub running: Arc<AtomicBool>,
    pub allow_write: Arc<AtomicBool>,
    pub board: Arc<RwLock<Board>>,
    pub clients: DashMap<Uuid, ClientHandler>,
}

impl Server {
    pub fn new() -> Self {
        let board: Arc<RwLock<Board>> = Arc::new(Board::new().into());

        let running = Arc::new(AtomicBool::new(true));

        let allow_write = Arc::new(AtomicBool::new(false));

        Self {
            running,
            allow_write,
            board,
            clients: DashMap::new(),
        }
    }

    pub fn broadcast(&self, data: &Response) {
        self.clients.iter().for_each(|e| e.value().send(data));
    }

    pub fn new_client(arcself: Arc<Self>, out: ws::Sender) -> ClientHandler {
        let mut client = ClientHandler {
            id: Uuid::new_v4(),
            server: arcself.clone(),
            out: Arc::new(out),
        };

        debug!("Creating a new client (id: {}).", client.id);
        arcself.clients.insert(client.id, client.clone());
        client
    }

    pub fn remove_client(&self, cid: Uuid, disconnect: bool) {
        let client = self.clients.remove(&cid).unwrap();
        if disconnect {
            let temp = client.1;
            debug!("Client (id: {}) kicked by the server.", temp.id);
            temp.disconnect();
        }
    }
}

#[derive(Clone)]
pub struct ClientHandler {
    pub id: Uuid,
    pub server: Arc<Server>,

    pub out: Arc<ws::Sender>,
}

impl ws::Handler for ClientHandler {
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        // let server = unsafe { &mut *self.server };
        match msg {
            ws::Message::Text(buf) => match serde_json::from_str::<Request>(&buf) {
                // Ok(Request::JOIN_GAME { name, game_size }) => {
                //     // let instance_id = server.find_or_new_instance(game_size);
                //     // let joined = server
                //     //     .get_instance(instance_id)
                //     //     .unwrap()
                //     //     .add_client(self as *mut ClientHandler, name);

                //     // if !joined {
                //     //     return Err(ws::Error::new(ws::ErrorKind::Capacity, "Instance is full"));
                //     // }

                //     // self.instance_id = Some(instance_id);
                //     // debug!("Successfully setted connecting instance.");
                //     // Ok(())
                // }
                Ok(Request::REQUEST_FRAME {
                    x_origin,
                    y_origin,
                    x_size,
                    y_size,
                }) => {
                    self.server
                        .board
                        .read()
                        .map(|board| board.get_window(x_origin, y_origin, x_size, y_size))
                        .map(|window| {
                            self.send(&Response::FRAME {
                                x_size: window[0].len(),
                                y_size: window.len(),
                                window,
                            });
                        });
                    Ok(())
                }
                Ok(Request::EXIT_GAME) => {
                    // self.instance_id.map(|o| {
                    //     server
                    //         .get_instance(o)
                    //         .map(|i| i.remove_client(self.id, false))
                    // });
                    // self.clear_instance();
                    Ok(())
                }
                // Ok(data) => {
                //     // self.instance_id
                //     //     .map(|o| server.get_instance(o).map(|i| i.process(self.id, data)));
                //     // Ok(())
                // }
                _ => Err(ws::Error::new(ws::ErrorKind::Protocol, "Unrecognized data")),
                Err(_) => Err(ws::Error::new(
                    ws::ErrorKind::Protocol,
                    "Unparsable data sent",
                )),
            },
            ws::Message::Binary(_) => Err(ws::Error::new(
                ws::ErrorKind::Protocol,
                "Binary not accepted",
            )),
        }
    }

    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        let id = self.id;
        info!("Identified client with {}", id);

        self.server.board.write().map(|mut board| {
            if let Some(spawn_pos) = board.find_random_safe_position(5) {
                board.set(spawn_pos, Unit::new_queen(id));
                board.set(Position { x: spawn_pos.x, y: spawn_pos.y + 1 }, Unit {
                    hp: 1,
                    tile: TileType::FEEDER,
                    team: id,
                    ..Default::default()
                });

                self.out
                    .send(
                        serde_json::to_string(&Response::IDENTIFY {
                            id: id,
                            origin: spawn_pos,
                        })
                        .unwrap(),
                    )
                    .unwrap();
            } else {
                self.disconnect()
            }
        });

        Ok(())
    }

    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        let mut response = ws::Response::from_request(req)?;
        response.set_protocol(PROTOCOL);
        Ok(response)
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        // let server = unsafe { &mut *self.server };
        // self.instance_id.map(|o| {
        //     server
        //         .get_instance(o)
        //         .map(|i| i.remove_client(self.id, false))
        // });

        self.server.remove_client(self.id, false);

        match code {
            ws::CloseCode::Normal => info!("Client (id: {}) has closed the connection.", self.id),
            ws::CloseCode::Away => info!("Client (id: {}) is leaving the website.", self.id),
            _ => warn!(
                "Client (id: {}) has encountered an error ({:?}): {}.",
                self.id, code, reason
            ),
        }
    }
}

impl ClientHandler {
    pub fn send(&self, data: &Response) {
        info!("Sending message...");
        self.out
            .send(serde_json::to_string(data).expect("Can not serialize"))
            .expect("Error while sending");
    }

    pub fn disconnect(&self) {
        self.out
            .close(ws::CloseCode::Normal)
            .expect("Error when closing connection");
    }
}

impl Drop for ClientHandler {
    fn drop(&mut self) {
        debug!("Dropping client (id: {}).", self.id);
    }
}
