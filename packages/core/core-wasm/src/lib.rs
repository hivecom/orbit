use std::{fmt, str::FromStr};

use core_shared::{
    SendCommand,
    actor::{ActorCommand, ActorMessage, CommandResponse, IrcActor, IrcConnection},
    state::{Message, Server},
};
use futures::{
    SinkExt, StreamExt,
    channel::{mpsc::UnboundedSender, oneshot},
    stream::{Fuse, LocalBoxStream, SplitSink},
};
use gloo_console::debug;
use gloo_net::websocket::{self, WebSocketError, futures::WebSocket};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[macro_export]
macro_rules! dbg {
    () => {
        ::gloo_console::debug!(&format!("[{}:{}:{}]", file!(), line!(), column!()));
    };

    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                ::gloo_console::debug!(&format!("[{}:{}:{}] {} = {:#?}",
                    file!(),
                    line!(),
                    column!(),
                    stringify!($val),
                    &&tmp as &dyn std::fmt::Debug,
                ));
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

use tracing_subscriber::prelude::*;
use tracing_subscriber_wasm::MakeConsoleWriter;

pub fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(MakeConsoleWriter::default()) // Bridges to browser console
        .with_ansi(false) // Browser console doesn't support colors
        .without_time() // Browser already adds timestamps
        .with_file(true) // Shows file
        .with_line_number(true) // Shows line
        .with_target(true); // Shows module/function path

    tracing_subscriber::registry().with(fmt_layer).init();
}

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    // tracing_wasm::set_as_global_default();

    init_tracing();

    debug!("WASM panic hook & logger initialized");
}

#[wasm_bindgen]
pub async fn initialize_orbit() -> Result<ServerList, JsError> {
    ServerList::new().await
}

#[wasm_bindgen(getter_with_clone)]
pub struct ServerList {
    pub servers: Vec<IrcServer>,
}

#[wasm_bindgen]
impl ServerList {
    #[wasm_bindgen]
    pub async fn new() -> Result<Self, JsError> {
        Ok(Self {
            servers: Vec::new(),
        })
    }

    #[wasm_bindgen]
    pub async fn connect(&mut self, url: &str) -> Result<(), JsError> {
        self.servers.push(IrcServer::connect(url).await?);

        Ok(())
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct IrcServer {
    address: UnboundedSender<ActorMessage>,
}

#[wasm_bindgen]
impl IrcServer {
    pub async fn connect(url: &str) -> Result<Self, JsError> {
        let connection = WsConnection::new(url)?;
        let address =
            IrcActor::new(connection, |actor| spawn_local(async { actor.run().await })).await;

        Ok(Self { address })
    }

    #[wasm_bindgen]
    pub async fn register(
        &mut self,
        nick: String,
        user: String,
        realname: String,
    ) -> Result<Server, JsError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::Register {
                    nick,
                    user,
                    realname,
                },
                reply_tx: tx,
            })
            .await?;

        let resp = rx.await?;
        let CommandResponse::Register(server) = resp else {
            unreachable!("expected register, got: {:?}", resp);
        };

        Ok(server)
    }

    #[wasm_bindgen]
    pub async fn join_channel(
        &mut self,
        channel: String,
        password: Option<String>,
    ) -> Result<IrcChannel, JsError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::Join { channel, password },
                reply_tx: tx,
            })
            .await?;

        let resp = rx.await?;
        let CommandResponse::Join(name) = resp else {
            unreachable!("expected join, got: {:?}", resp);
        };

        Ok(IrcChannel {
            name,
            address: self.address.clone(),
        })
    }
}

#[wasm_bindgen]
pub struct IrcChannel {
    name: String,
    address: UnboundedSender<ActorMessage>,
}

#[wasm_bindgen]
impl IrcChannel {
    #[wasm_bindgen]
    pub async fn send_message(&mut self, text: String) -> Result<Message, JsError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::Privmsg {
                    target: self.name.clone(),
                    text,
                },
                reply_tx: tx,
            })
            .await?;

        let resp = rx.await?;
        let CommandResponse::Privmsg(message) = resp else {
            unreachable!("expected join, got: {:?}", resp);
        };

        Ok(message)
    }
}

struct WsConnection {
    socket: WebSocket,
}

impl fmt::Debug for WsConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WsConnection").finish_non_exhaustive()
    }
}

impl WsConnection {
    pub fn new(url: &str) -> Result<Self, JsError> {
        Ok(WsConnection {
            socket: WebSocket::open(url)?,
        })
    }
}

impl IrcConnection for WsConnection {
    type Incoming = Fuse<LocalBoxStream<'static, irc_proto::Message>>;
    type Outgoing = OutgoingSink;

    fn in_out(self) -> (Self::Incoming, Self::Outgoing) {
        let (sink, stream) = self.socket.split();
        let incoming = stream
            .map(|msg| {
                let msg = msg.unwrap();
                let websocket::Message::Text(msg) = msg else {
                    panic!("unexpected binary message");
                };
                irc_proto::Message::from_str(&msg).unwrap()
            })
            .boxed_local();

        (incoming.fuse(), OutgoingSink { inner: sink })
    }
}

struct OutgoingSink {
    inner: SplitSink<WebSocket, websocket::Message>,
}

impl SendCommand for OutgoingSink {
    type Error = WebSocketError;
    async fn message(&mut self, message: irc_proto::Message) -> Result<(), Self::Error> {
        self.inner
            .send(websocket::Message::Text(message.to_string()))
            .await?;

        Ok(())
    }
}
