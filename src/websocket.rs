use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::PixelUpdateMessage;

pub type Clients = Arc<RwLock<HashMap<Uuid, HashMap<usize, Recipient<PixelUpdateMessage>>>>>;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub game_id: Uuid,
    pub addr: Recipient<PixelUpdateMessage>,
    pub client_id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub game_id: Uuid,
    pub client_id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PixelUpdate {
    pub game_id: Uuid,
    pub message: PixelUpdateMessage,
}

pub struct WebSocketServer {
    pub clients: Clients,
    client_counter: usize,
}

impl WebSocketServer {
    pub fn new() -> Self {
        WebSocketServer {
            clients: Arc::new(RwLock::new(HashMap::new())),
            client_counter: 0,
        }
    }
}

impl Actor for WebSocketServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let clients: Clients = Arc::clone(&self.clients);
        let game_id = msg.game_id;
        let client_id = msg.client_id;
        let addr = msg.addr;

        actix::spawn(async move {
            let mut clients_lock = clients.write().await;
            clients_lock
                .entry(game_id)
                .or_insert_with(HashMap::new)
                .insert(client_id, addr);
        });
    }
}

impl Handler<Disconnect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        let clients: Clients = Arc::clone(&self.clients);
        let game_id = msg.game_id;
        let client_id = msg.client_id;

        actix::spawn(async move {
            let mut clients_lock = clients.write().await;
            if let Some(game_clients) = clients_lock.get_mut(&game_id) {
                game_clients.remove(&client_id);
                if game_clients.is_empty() {
                    clients_lock.remove(&game_id);
                }
            }
        });
    }
}

impl Handler<PixelUpdate> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: PixelUpdate, _: &mut Context<Self>) -> Self::Result {
        let clients: Clients = Arc::clone(&self.clients);
        let game_id = msg.game_id;
        let message = msg.message;

        actix::spawn(async move {
            let clients_lock = clients.read().await;
            if let Some(game_clients) = clients_lock.get(&game_id) {
                for (_, client) in game_clients.iter() {
                    let _ = client.do_send(message.clone());
                }
            }
        });
    }
}

pub struct WebSocketSession {
    pub game_id: Uuid,
    pub client_id: usize,
    pub addr: Addr<WebSocketServer>,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address().recipient();
        self.addr.do_send(Connect {
            game_id: self.game_id,
            addr,
            client_id: self.client_id,
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        self.addr.do_send(Disconnect {
            game_id: self.game_id,
            client_id: self.client_id,
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl Handler<PixelUpdateMessage> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: PixelUpdateMessage, ctx: &mut Self::Context) -> Self::Result {
        if let Ok(text) = serde_json::to_string(&msg) {
            ctx.text(text);
        }
    }
}

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<Uuid>,
    data: web::Data<Addr<WebSocketServer>>,
) -> Result<HttpResponse, Error> {
    let game_id = path.into_inner();
    let client_id = rand::random::<usize>();
    
    let session = WebSocketSession {
        game_id,
        client_id,
        addr: data.get_ref().clone(),
    };

    ws::start(session, &req, stream)
}