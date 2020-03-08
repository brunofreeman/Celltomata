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
use crate::constants;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

pub const PROTOCOL: &'static str = "game-of-strife";

// todo how to not use raw pointers
pub struct Server {
    pub running: Arc<AtomicBool>,
    pub board: Arc<RwLock<Board>>,
    pub clients: DashMap<Uuid, ClientHandler>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
            board: Arc::new(Board::new().into()),
            clients: DashMap::new(),
        }
    }

    pub fn broadcast(&self, data: &Response) {
        self.clients.iter().for_each(|e| e.value().send(data));
    }

    pub fn new_client(arcself: Arc<Self>, out: ws::Sender) -> ClientHandler {
        let mut client = ClientHandler {
            id: Uuid::new_v4(),
            name: None,
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
    pub name: Option<String>,
    pub server: Arc<Server>,

    pub out: Arc<ws::Sender>,
}

impl ws::Handler for ClientHandler {
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        // let server = unsafe { &mut *self.server };
        match msg {
            ws::Message::Text(buf) => match serde_json::from_str::<Request>(&buf) {
                Ok(Request::NEW_PLAYER {
                    username
                }) => {
                    self.name = Some(username);
                    Ok(())
                }
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
                Ok(Request::PUT {
                    position,
                    tile
                }) => {
                    self.server.board.write().map(|mut board| {
                        if board.get(position).is_empty() {
                            let energy = board.get_erg_mut(self.id).unwrap();
                            if tile.get_cost() < *energy {
                                *energy -= tile.get_cost();
                                self.send(&Response::ENERGY_UPDATE {
                                    erg: *energy
                                });
                                board.set(position, Unit::new_unit(self.id, position, tile));
                            }
                        }
                    });
                    Ok(())
                }
                Ok(Request::EXIT_GAME) => {
                    self.disconnect();
                    Ok(())
                }
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
                // care package
                let queen = Unit::new_queen(id, spawn_pos);
                board.set(spawn_pos, queen);

                let feeder_pos = Position { x: spawn_pos.x, y: spawn_pos.y + 1 };
                board.set(feeder_pos, queen.spawn_unit(feeder_pos, TileType::FEEDER));

                // board.set(Position { x: spawn_pos.x - 2, y: spawn_pos.y - 2 }, queen.spawn_unit(TileType::SPAWNER));

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
        self.server.remove_client(self.id, false);

        self.server.board.write().map(|mut board| {
            board.kill_team(self.id);
        });

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
