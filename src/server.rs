use serde_json;
use uuid::Uuid;
use ws;
use crate::server;
use std::collections::HashMap;
use crate::msg::{Response, Request};
use std::sync::Arc;
use dashmap::DashMap;

pub const PROTOCOL: &'static str = "game-of-strife";

// todo how to not use raw pointers
pub struct Server<'a> {
    pub clients: DashMap<Uuid, *mut ClientHandler<'a>>,
}

impl<'a> Server<'a> {
    pub fn new() -> Self {
        Self {
            clients: DashMap::new()
        }
    }

    pub fn new_client(&'a self, out: ws::Sender) -> ClientHandler {
        let mut client = ClientHandler {
            id: Uuid::new_v4(),
            server: self,
            out: out,
        };

        // debug!("Creating a new client (id: {}).", client.id);

        // let arc = Arc::new(client)

        self.clients.insert(client.id, &mut client as *mut ClientHandler);
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
        self.out
            .send(serde_json::to_string(&Response::IDENTIFY { id: self.id }).unwrap())
            .unwrap();
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
            ws::CloseCode::Normal => debug!("Client (id: {}) has closed the connection.", self.id),
            ws::CloseCode::Away => debug!("Client (id: {}) is leaving the website.", self.id),
            _ => warn!(
                "Client (id: {}) has encountered an error ({:?}): {}.",
                self.id, code, reason
            ),
        }
    }
}

impl ClientHandler<'_> {
    pub fn send(&self, data: &Response) {
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