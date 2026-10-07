use std::{fmt, str::FromStr};

use anyhow::{Context, bail};
use core_shared::{
    SendCommand,
    actor::{self, ActorCommand, ActorMessage, IrcActor},
    response_channels::CommandResponse,
    state::{Channel, History, Message, Server, ServerEvent, SignedIn},
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
use serde::Serialize;
use serde_wasm_bindgen::to_value;
use tracing::debug;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, spawn_local};

pub use error::OrbitError;

mod error;

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

macro_rules! cmd_resp {
    ($e:expr, $p:path) => {
        match $e {
            $p(value) => Ok(value),
            CommandResponse::Error(e) => Err(e),
            _ => unreachable!("expected {}, got: {:?}", stringify!($p), $e),
        }
    };
}

use tracing_subscriber::prelude::*;
use tracing_subscriber_wasm::MakeConsoleWriter;

use crate::database::IndexedDb;

mod database;

const DATABASE_NAME: &str = "orbit-core";

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
pub async fn initialize_orbit() -> Result<ServerList, OrbitError> {
    ServerList::new().await
}

#[wasm_bindgen(getter_with_clone)]
pub struct ServerList {
    pub servers: Vec<IrcConnection>,
}

#[wasm_bindgen]
impl ServerList {
    #[wasm_bindgen]
    pub async fn new() -> Result<Self, OrbitError> {
        Ok(Self {
            servers: Vec::new(),
        })
    }

    #[wasm_bindgen]
    pub async fn connect(&mut self, url: String) -> Result<IrcConnection, OrbitError> {
        let id = self.max_id().unwrap_or(-1) + 1;
        let connection = IrcConnection::connect(id, url).await?;
        self.servers.push(connection.clone());

        Ok(connection)
    }

    fn max_id(&mut self) -> Option<i32> {
        self.servers.iter().map(|s| s.id()).max()
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct IrcConnection {
    id: i32,
    address: UnboundedSender<ActorMessage>,
}

#[wasm_bindgen]
impl IrcConnection {
    async fn connect(id: i32, url: String) -> Result<Self, OrbitError> {
        let connection = WsConnection::new(url)?;
        let database = IndexedDb::new(DATABASE_NAME).await?;
        let address = IrcActor::start(id, connection, database, |actor| {
            spawn_local(async { actor.run().await })
        })
        .await?;

        Ok(Self { id, address })
    }

    #[wasm_bindgen]
    pub async fn state(&mut self) -> Result<Js<Server>, OrbitError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::GetState,
                reply_tx: Some(tx),
            })
            .await
            .context("Failed to send ActorMessage")?;

        let resp = rx.await.context("Failed to await actor state message")?;
        let server = cmd_resp!(resp, CommandResponse::GetState)?;

        Ok(Js(*server))
    }

    #[wasm_bindgen]
    pub fn id(&self) -> i32 {
        self.id
    }

    #[wasm_bindgen]
    pub async fn channel_list(&mut self) -> Result<JsValue, OrbitError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::GetChannelList,
                reply_tx: Some(tx),
            })
            .await
            .context("Failed to send ActorMessage")?;

        let resp = rx.await.context("Failed to await actor state message")?;
        let CommandResponse::ChannelList(list) = resp else {
            unreachable!("expected channel list, got: {:?}", resp);
        };

        Ok(to_value(&list)?)
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
                if let Err(e) = f.call1(&JsValue::null(), &Js(ServerEvent::from(event)).into()) {
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
                if let Err(e) = f.call1(&JsValue::null(), &OrbitError::from(event).into()) {
                    gloo_console::error!("Error during error callback: {}", e);
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
    ) -> Result<Js<SignedIn>, OrbitError> {
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
            .await
            .context("Failed to send ActorMessage")?;

        let resp = rx.await.context("Failed to await actor sign in message")?;
        let result = cmd_resp!(resp, CommandResponse::SignIn)?;

        Ok(Js(result))
    }

    #[wasm_bindgen]
    pub async fn sign_in_anonymous(
        &mut self,
        nick: String,
        user: String,
        realname: String,
    ) -> Result<Js<SignedIn>, OrbitError> {
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
            .await
            .context("Failed to send ActorMessage")?;

        let resp = rx.await.context("Failed to await actor sign in message")?;

        let result = cmd_resp!(resp, CommandResponse::SignIn)?;

        Ok(Js(result))
    }

    #[wasm_bindgen]
    pub async fn join_channel(
        &mut self,
        channel: String,
        password: Option<String>,
    ) -> Result<IrcChannel, OrbitError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::Join { channel, password },
                reply_tx: Some(tx),
            })
            .await
            .context("Failed to send ActorMessage")?;

        let resp = rx.await.context("Failed to await actor join message")?;
        let channel = cmd_resp!(resp, CommandResponse::Join)?;

        Ok(IrcChannel {
            name: channel.metadata.name,
            address: self.address.clone(),
        })
    }

    #[wasm_bindgen]
    pub async fn history_before(
        &mut self,
        channel: String,
        before_msgid: String,
    ) -> Result<Js<History>, OrbitError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::RequestHistory {
                    channel,
                    before_msgid,
                },
                reply_tx: Some(tx),
            })
            .await
            .context("Failed to send ActorMessage")?;

        let resp = rx.await.context("Failed to await actor history message")?;
        let history = cmd_resp!(resp, CommandResponse::History)?;

        Ok(Js(history))
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
    pub async fn state(&mut self) -> Result<Js<Option<Channel>>, OrbitError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::GetChannelState(self.name.clone()),
                reply_tx: Some(tx),
            })
            .await
            .context("Failed to send ActorMessage")?;

        let resp = rx.await.context("Failed to await actor state message")?;
        let channel = cmd_resp!(resp, CommandResponse::GetChannelState)?;

        Ok(Js(*channel))
    }

    #[wasm_bindgen]
    pub async fn send_message(&mut self, text: String) -> Result<Js<Message>, OrbitError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::Privmsg {
                    target: self.name.clone(),
                    text,
                },
                reply_tx: Some(tx),
            })
            .await
            .context("Failed to send ActorMessage")?;

        let resp = rx.await.context("Failed to await actor privmessage")?;
        let message = cmd_resp!(resp, CommandResponse::Privmsg)?;

        Ok(Js(*message))
    }
}

struct WsConnection {
    address: String,
    socket: WebSocket,
}

impl fmt::Debug for WsConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WsConnection").finish_non_exhaustive()
    }
}

impl WsConnection {
    fn new(url: String) -> Result<Self, OrbitError> {
        Ok(WsConnection {
            socket: WebSocket::open(&url).context("Failed to open WebSocket")?,
            address: url,
        })
    }
}

impl actor::IrcConnection for WsConnection {
    type Incoming = Fuse<LocalBoxStream<'static, anyhow::Result<irc_proto::Message>>>;
    type Outgoing = OutgoingSink;

    fn address(&self) -> &str {
        &self.address
    }

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

pub struct Js<T: Serialize>(T);

impl<T: Serialize> From<T> for Js<T> {
    fn from(value: T) -> Self {
        Js(value)
    }
}

use wasm_bindgen::convert::IntoWasmAbi;
use wasm_bindgen::describe::WasmDescribe;

impl<T: Serialize> From<Js<T>> for JsValue {
    fn from(v: Js<T>) -> Self {
        serde_wasm_bindgen::to_value(&v.0).unwrap_or_else(|e| e.to_string().into())
    }
}

impl<T: Serialize> WasmDescribe for Js<T> {
    fn describe() {
        wasm_bindgen::JsValue::describe()
    }
}

impl<T: Serialize> IntoWasmAbi for Js<T> {
    type Abi = <wasm_bindgen::JsValue as IntoWasmAbi>::Abi;

    fn into_abi(self) -> Self::Abi {
        let js_val: wasm_bindgen::JsValue = self.into();
        js_val.into_abi()
    }
}
