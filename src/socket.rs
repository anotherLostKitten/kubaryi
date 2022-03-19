use actix::{fut, ActorContext, WrapFuture, ContextFutureSpawner, ActorFuture};
use actix::{Actor, Addr, Running, StreamHandler};
use actix::{AsyncContext, Handler};
use actix::prelude::{Message, Recipient, Context};
use std::collections::{HashMap, HashSet};
use actix_web_actors::ws;
use actix_web_actors::ws::Message::{Text, Binary};
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Conn {
    room: usize,
    game_addr: Addr<Game>,
    hb: Instant,
    user: usize,
}

type ChatResult = ();

impl Conn {
    pub fn new(room: usize, game: Addr<Game>, user: usize) -> Conn {
        Conn {
            room: room,
            game_addr: game,
            user: user,
            hb: Instant::now(),
        }
    }
}

impl Actor for Conn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.game_addr.send(Connect {
            addr: addr.recipient(),
            room: self.room,
            user: self.user,
        })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => {},
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.game_addr.do_send(Disconnect {
            user: self.user,
            room: self.room,
        });
        Running::Stop
    }
}

impl Conn {
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                act.game_addr.do_send(Disconnect {
                    user: act.user,
                    room: act.room,
                });
                ctx.stop();
                return;
            }

            ctx.ping(b"big chungus");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Conn {
    fn handle(&mut self, msg:Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            },
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            },
            Ok(Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            },
            Ok(ws::Message::Continuation(_)) => {},
            Ok(ws::Message::Nop) => {},
            Ok(Text(s)) => self.game_addr.do_send(ClientActorMessage {
                user: self.user,
                msg: s,
                room: self.room,
            }),
            Err(e) => {
                eprintln!("{}", e);
                ctx.stop();
            }
        }
    }
}

impl Handler<ChatMsg> for Conn {
    type Result = ChatResult;
    
    fn handle(&mut self, msg: ChatMsg, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

#[derive(Message)]
#[rtype(result="ChatResult")]
pub struct ChatMsg(pub String);

#[derive(Message)]
#[rtype(result="()")]
pub struct Connect {
    pub addr: Recipient<ChatMsg>,
    pub room: usize,
    pub user: usize,
}

#[derive(Message)]
#[rtype(result="()")]
pub struct Disconnect {
    pub room: usize,
    pub user: usize,
}

#[derive(Message)]
#[rtype(result="()")]
pub struct ClientActorMessage {
    pub msg: String,
    pub room: usize,
    pub user: usize,
}


pub struct Game {
    sessions: HashMap<usize, Recipient<ChatMsg>>,
    rooms: HashMap<usize, HashSet<usize>>,
}

impl Default for Game {
    fn default() -> Game {
        Game {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
        }
    }
}

impl Actor for Game {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Game {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        println!("{} has ceased", msg.user);
        if self.sessions.remove(&msg.user).is_some() {
            if let Some(players) = self.rooms.get_mut(&msg.room) {
                if players.len() > 1 {
                    players.remove(&msg.user);
                    for user in players.iter() {
                        if let Some(rec) = self.sessions.get(user) {
                            let _ =  rec.do_send(ChatMsg(format!("{} has ceased", msg.user)));
                        }
                    }
                } else {
                    self.rooms.remove(&msg.room);
                }
            }
        }
    }
}

impl Handler<Connect> for Game {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        println!("{} has corporialized", msg.user);
        self.sessions.insert(msg.user, msg.addr);
        
        if let Some(players) = self.rooms.get_mut(&msg.room) {
            for user in players.iter() {
                if let Some(rec) = self.sessions.get(user) {
                    let _ = rec.do_send(ChatMsg(format!("{} has corporialized", msg.user)));
                }
            }
            players.insert(msg.user);
        } else {
            self.rooms.entry(msg.room).or_insert_with(HashSet::new).insert(msg.user);
        }
        
    }
}

impl Handler<ClientActorMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Self::Context) -> Self::Result {
        if let Some(players) = self.rooms.get_mut(&msg.room) {
            for user in players.iter() {
                if let Some(rec) = self.sessions.get(user) {
                    let _ = rec.do_send(ChatMsg(msg.msg.to_owned()));
                }
            }
        }
    }
}
