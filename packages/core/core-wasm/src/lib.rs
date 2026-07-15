use std::{fmt, str::FromStr};

use anyhow::bail;
use core_shared::{
    SendCommand,
    actor::{self, ActorCommand, ActorMessage, CommandResponse, IrcActor},
    state::{self, Capabilities, Message, ServerMetadata, User},
};
use futures::{
    SinkExt, StreamExt,
    channel::{
        mpsc::{self, UnboundedSender},
        oneshot,
    },
    stream::{Fuse, LocalBoxStream, SplitSink},
};
use gloo_net::websocket::{self, WebSocketError, futures::WebSocket};
use js_sys::{JsString, Map};
use tracing::debug;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, spawn_local};

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

fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(MakeConsoleWriter::default())
        .with_ansi(false)
        .without_time()
        .with_file(true)
        .with_line_number(true)
        .with_target(true);

    tracing_subscriber::registry().with(fmt_layer).init();
}

#[wasm_bindgen(start)]
fn init() {
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
    pub servers: Vec<IrcConnection>,
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
    pub async fn connect(&mut self, url: &str) -> Result<ConnectResult, JsError> {
        let (connection, data) = IrcConnection::connect(url).await?;
        self.servers.push(connection.clone());

        Ok(ConnectResult { connection, data })
    }
}

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct ConnectResult {
    pub connection: IrcConnection,
    pub data: Server,
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct IrcConnection {
    address: UnboundedSender<ActorMessage>,
}

#[wasm_bindgen]
impl IrcConnection {
    async fn connect(url: &str) -> Result<(Self, Server), JsError> {
        let connection = WsConnection::new(url)?;
        let (address, data) =
            IrcActor::new(connection, |actor| spawn_local(async { actor.run().await }))
                .await
                .map_err(|e| JsError::new(&e.to_string()))?;

        Ok((Self { address }, data.into()))
    }

    #[wasm_bindgen]
    pub fn on_data(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "(event: ServerEvent) => void")] f: js_sys::Function,
    ) {
        let (handler_tx, mut handler_rx) = mpsc::unbounded();

        let mut address = self.address.clone();
        spawn_local(async move {
            address
                .send(ActorMessage {
                    command: ActorCommand::AddEventHandler {
                        handler: handler_tx,
                    },
                    reply_tx: None,
                })
                .await
                .expect("can send actor message");

            while let Ok(event) = handler_rx.recv().await {
                if let Err(e) = f.call1(&JsValue::null(), &event.into()) {
                    gloo_console::error!("Error during event callback: {}", e);
                }
            }
        });
    }

    #[wasm_bindgen]
    pub fn on_error(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "(event: ServerError) => void")] f: js_sys::Function,
    ) {
        let (handler_tx, mut handler_rx) = mpsc::unbounded();

        let mut address = self.address.clone();
        spawn_local(async move {
            address
                .send(ActorMessage {
                    command: ActorCommand::AddErrorHandler {
                        handler: handler_tx,
                    },
                    reply_tx: None,
                })
                .await
                .expect("can send actor message");

            while let Ok(event) = handler_rx.recv().await {
                if let Err(e) = f.call1(&JsValue::null(), &event.into()) {
                    gloo_console::error!("Error during event callback: {}", e);
                }
            }
        });
    }

    #[wasm_bindgen]
    pub fn on_disconnect(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "(event: string) => void")] f: js_sys::Function,
    ) {
        let (handler_tx, mut handler_rx) = mpsc::unbounded();

        let mut address = self.address.clone();
        spawn_local(async move {
            address
                .send(ActorMessage {
                    command: ActorCommand::AddDisconectHandler {
                        handler: handler_tx,
                    },
                    reply_tx: None,
                })
                .await
                .expect("can send actor message");

            while let Ok(event) = handler_rx.recv().await {
                if let Err(e) = f.call1(&JsValue::null(), &event.into()) {
                    gloo_console::error!("Error during event callback: {}", e);
                }
            }
        });
    }

    #[wasm_bindgen]
    pub async fn sign_in(
        &mut self,
        nick: String,
        user: String,
        realname: String,
        password: String,
    ) -> Result<(), JsError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::SignIn {
                    nick,
                    user,
                    realname,
                    password,
                },
                reply_tx: Some(tx),
            })
            .await?;

        let resp = rx.await?;
        let CommandResponse::SignIn(result) = resp else {
            unreachable!("expected sign in, got: {:?}", resp);
        };

        Ok(result.map_err(|e| JsError::new(&e.to_string()))?)
    }

    #[wasm_bindgen]
    pub async fn sign_in_anonymous(
        &mut self,
        nick: String,
        user: String,
        realname: String,
    ) -> Result<(), JsError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::SignInAnonymous {
                    nick,
                    user,
                    realname,
                },
                reply_tx: Some(tx),
            })
            .await?;

        let resp = rx.await?;
        let CommandResponse::SignIn(result) = resp else {
            unreachable!("expected sign in, got: {:?}", resp);
        };

        Ok(result.map_err(|e| JsError::new(&e.to_string()))?)
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
                reply_tx: Some(tx),
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
                reply_tx: Some(tx),
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
    fn new(url: &str) -> Result<Self, JsError> {
        Ok(WsConnection {
            socket: WebSocket::open(url)?,
        })
    }
}

impl actor::IrcConnection for WsConnection {
    type Incoming = Fuse<LocalBoxStream<'static, anyhow::Result<irc_proto::Message>>>;
    type Outgoing = OutgoingSink;

    fn in_out(self) -> (Self::Incoming, Self::Outgoing) {
        let (sink, stream) = self.socket.split();
        let incoming = stream
            .map(|msg| {
                let websocket::Message::Text(msg) = msg? else {
                    bail!("unexpected binary message");
                };

                Ok(irc_proto::Message::from_str(&msg)?)
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

#[derive(Debug, Clone, Tsify)]
#[wasm_bindgen(getter_with_clone)]
pub struct Server {
    pub metadata: ServerMetadata,

    #[tsify(type = "Map<string, Channel>")]
    pub channels: Map<JsString, JsValue>,

    pub capabilities: Capabilities,

    #[tsify(type = "Map<string, User>")]
    pub users: Map<JsString, JsValue>,

    pub me: Option<User>,
}

impl From<state::Server> for Server {
    fn from(server: state::Server) -> Self {
        let channels = js_sys::Map::new_typed();
        for (k, v) in server.channels {
            channels.set(&JsString::from(k), &JsValue::from(v));
        }

        let users = js_sys::Map::new_typed();
        for (k, v) in server.users {
            users.set(&JsString::from(k), &JsValue::from(v));
        }

        Self {
            metadata: server.metadata,
            channels,
            capabilities: server.capabilities,
            users,
            me: server.me,
        }
    }
}
