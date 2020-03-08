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
pub struct Server<'a> {
    pub running: Arc<AtomicBool>,
    pub allow_write: Arc<AtomicBool>,
    pub board: Arc<RwLock<Board>>,
    pub thread: JoinHandle<()>,
    pub clients: DashMap<Uuid, *mut ClientHandler<'a>>,
}

impl<'a> Server<'a> {
    pub fn new() -> Self {
        let board: Arc<RwLock<Board>> = Arc::new(Board::new().into());
        let board_handle = board.clone();

        let running = Arc::new(AtomicBool::new(true));
        let running_handle = running.clone();

        let allow_write = Arc::new(AtomicBool::new(false));
        let allow_write_handle = allow_write.clone();

        let handle = std::thread::spawn(move || {
            let mut n = 10;
            let mut gen: usize = 0;
            while running_handle.load(Ordering::SeqCst) {
                if n == 0 {
                    allow_write_handle.store(true, Ordering::SeqCst);
                    warn!("Allow writing.");
                    std::thread::sleep(Duration::from_secs(10));
                    allow_write_handle.store(false, Ordering::SeqCst);
                    warn!("Lock.");
                    n = 10;
                } else {
                    std::thread::sleep(Duration::from_secs(1));
                    n -= 1;
                }
                gen += 1;

                board_handle.write().map(|mut board| board.next());
                
                info!("Generation {} generated.", gen);
            }
            info!("Done.");
        });

        Self {
            running,
            allow_write,
            board,
            clients: DashMap::new(),
            thread: handle,
        }
    }

    pub fn new_client(&'a self, out: ws::Sender) -> ClientHandler {
        let mut client = ClientHandler {
            id: Uuid::new_v4(),
            server: self,
            out,
        };

        debug!("Creating a new client (id: {}).", client.id);
        self.clients
            .insert(client.id, &mut client as *mut ClientHandler);
        client
    }

    pub fn remove_client(&self, cid: Uuid, disconnect: bool) {
        let client = self.clients.remove(&cid).unwrap();
        if disconnect {
            let temp = unsafe { &mut *client.1 };
            debug!("Client (id: {}) kicked by the server.", temp.id);
            temp.disconnect();
        }
    }
}

pub struct ClientHandler<'a> {
    pub id: Uuid,
    pub server: &'a Server<'a>,

    pub out: ws::Sender,
}

impl<'a> ws::Handler for ClientHandler<'a> {
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
                                    window
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
        self.out
            .send(serde_json::to_string(&Response::IDENTIFY { id: id }).unwrap())
            .unwrap();

        self.server.board.write().map(|mut board| {
            board.set(
                Position { x: 5, y: 5 },
                Unit::new_queen(id),
            );

            board.set(
                Position { x: 8, y: 8 },
                Unit {
                    hp: 1,
                    tile: TileType::FEEDER,
                    team: id,
                    ..Default::default()
                },
            );
            board.set(
                Position { x: 4, y: 6 },
                Unit {
                    hp: 5,
                    tile: TileType::GUARD,
                    team: id,
                    ..Default::default()
                },
            );
            board.set(
                Position { x: 8, y: 7 },
                Unit {
                    hp: 1,
                    tile: TileType::BOLSTER,
                    team: id,
                    ..Default::default()
                },
            );
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

impl ClientHandler<'_> {
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

impl Drop for ClientHandler<'_> {
    fn drop(&mut self) {
        debug!("Dropping client (id: {}).", self.id);
    }
}
