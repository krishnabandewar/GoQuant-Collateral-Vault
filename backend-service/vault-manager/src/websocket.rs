use actix::prelude::*;
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// --- Messages ---

#[derive(Serialize, Deserialize, Clone, Debug, Message)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "balance_update")]
    BalanceUpdate {
        vault_pubkey: String,
        balance: i64,
        timestamp: i64,
    },
    #[serde(rename = "deposit")]
    Deposit {
        vault_pubkey: String,
        amount: i64,
        signature: String,
        timestamp: i64,
    },
    #[serde(rename = "withdrawal")]
    Withdrawal {
        vault_pubkey: String,
        amount: i64,
        signature: String,
        timestamp: i64,
    },
    #[serde(rename = "tvl_update")]
    TvlUpdate {
        total_value_locked: i64,
        timestamp: i64,
    },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}

// Internal message to register a new client
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
}

// Internal message to remove a client
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub addr: Recipient<WsMessage>,
}

// --- Broadcaster Actor ---

pub struct Broadcaster {
    sessions: HashSet<Recipient<WsMessage>>,
}

impl Broadcaster {
    pub fn new() -> Self {
        Broadcaster {
            sessions: HashSet::new(),
        }
    }
}

impl Actor for Broadcaster {
    type Context = Context<Self>;
}

impl Handler<Connect> for Broadcaster {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.sessions.insert(msg.addr);
    }
}

impl Handler<Disconnect> for Broadcaster {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.addr);
    }
}

impl Handler<WsMessage> for Broadcaster {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, _: &mut Context<Self>) {
        for session in self.sessions.iter() {
            session.do_send(msg.clone());
        }
    }
}

// --- WebSocket Actor ---

pub struct VaultWebSocket {
    hb: Instant,
    broadcaster: Addr<Broadcaster>,
}

impl VaultWebSocket {
    pub fn new(broadcaster: Addr<Broadcaster>) -> Self {
        Self {
            hb: Instant::now(),
            broadcaster,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("WebSocket Client heartbeat failed, disconnecting!");
                act.broadcaster.do_send(Disconnect {
                    addr: ctx.address().recipient(),
                });
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for VaultWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        // Register self with broadcaster
        let addr = ctx.address();
        self.broadcaster.do_send(Connect {
            addr: addr.recipient(),
        });
        println!("WebSocket connection established");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        // Remove self from broadcaster
        self.broadcaster.do_send(Disconnect {
            addr: ctx.address().recipient(),
        });
        println!("WebSocket connection closed");
    }
}

// Handle incoming messages from the client (e.g. Ping)
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for VaultWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // If the client sends a "Ping" JSON, we reply with "Pong"
                if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                     if let WsMessage::Ping = msg {
                         let _ = ctx.text(serde_json::to_string(&WsMessage::Pong).unwrap());
                     }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

// Allow the actor to receive WsMessage and send it as a string to the client
impl Handler<WsMessage> for VaultWebSocket {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }
}
