use std::{fmt, str::FromStr, sync::LazyLock};

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
    lock::Mutex,
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

static SERVER_STORE: LazyLock<Mutex<ServerList>> =
    LazyLock::new(|| Mutex::new(ServerList::default()));

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

#[derive(Default)]
pub struct ServerList {
    pub servers: Vec<IrcConnection>,
}

#[wasm_bindgen]
impl ServerList {
    fn max_id(&self) -> Option<i32> {
        self.servers.iter().map(|s| s.id).max()
    }

    fn by_id(&self, id: i32) -> Option<IrcConnection> {
        self.servers.iter().find(|s| s.id == id).cloned()
    }
}

#[wasm_bindgen]
pub async fn initialize_orbit() {
    SERVER_STORE.lock().await;
}

#[wasm_bindgen]
pub async fn connect(url: String) -> Result<i32, OrbitError> {
    let id = {
        let store = SERVER_STORE.lock().await;

        store.max_id().unwrap_or(-1) + 1
    };
    let connection = IrcConnection::connect(id, url).await?;

    {
        let mut store = SERVER_STORE.lock().await;

        store.servers.push(connection.clone());
    };

    Ok(id)
}

#[derive(Clone)]
pub struct IrcConnection {
    id: i32,
    address: UnboundedSender<ActorMessage>,
}

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
}

#[wasm_bindgen]
pub async fn server_state(server_id: i32) -> Result<Js<Server>, OrbitError> {
    let (tx, rx) = oneshot::channel();
    let mut server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;
    server
        .address
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
pub async fn channel_list(server_id: i32) -> Result<JsValue, OrbitError> {
    let (tx, rx) = oneshot::channel();
    let mut server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;
    server
        .address
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
pub async fn on_data(
    server_id: i32,
    #[wasm_bindgen(unchecked_param_type = "(event: ServerEvent) => void")] f: js_sys::Function,
) -> Result<(), OrbitError> {
    let (handler_tx, mut handler_rx) = mpsc::unbounded();

    let server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;
    let mut address = server.address.clone();
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

    Ok(())
}

#[wasm_bindgen]
pub async fn on_error(
    server_id: i32,
    #[wasm_bindgen(unchecked_param_type = "(event: ServerError) => void")] f: js_sys::Function,
) -> Result<(), OrbitError> {
    let (handler_tx, mut handler_rx) = mpsc::unbounded();

    let server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;
    let mut address = server.address.clone();
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

    Ok(())
}

#[wasm_bindgen]
pub async fn on_disconnect(
    server_id: i32,
    #[wasm_bindgen(unchecked_param_type = "(event: string) => void")] f: js_sys::Function,
) -> Result<(), OrbitError> {
    let (handler_tx, mut handler_rx) = mpsc::unbounded();

    let server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;
    let mut address = server.address.clone();
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

    Ok(())
}

#[wasm_bindgen]
pub async fn sign_in(
    server_id: i32,
    nick: String,
    user: String,
    realname: String,
    password: String,
) -> Result<Js<SignedIn>, OrbitError> {
    let (tx, rx) = oneshot::channel();
    let mut server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;
    server
        .address
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
    server_id: i32,
    nick: String,
    user: String,
    realname: String,
) -> Result<Js<SignedIn>, OrbitError> {
    let (tx, rx) = oneshot::channel();
    let mut server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;
    server
        .address
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
    server_id: i32,
    channel: String,
    password: Option<String>,
) -> Result<String, OrbitError> {
    let (tx, rx) = oneshot::channel();
    let mut server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;
    server
        .address
        .send(ActorMessage {
            command: ActorCommand::Join { channel, password },
            reply_tx: Some(tx),
        })
        .await
        .context("Failed to send ActorMessage")?;

    let resp = rx.await.context("Failed to await actor join message")?;
    let channel = cmd_resp!(resp, CommandResponse::Join)?;

    Ok(channel.metadata.name)
}

#[wasm_bindgen]
pub async fn history_before(
    server_id: i32,
    channel: String,
    before_msgid: String,
) -> Result<Js<History>, OrbitError> {
    let (tx, rx) = oneshot::channel();
    let mut server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;
    server
        .address
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

#[wasm_bindgen]
pub async fn channel_state(
    server_id: i32,
    channel_name: String,
) -> Result<Js<Option<Channel>>, OrbitError> {
    let mut server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;

    let (tx, rx) = oneshot::channel();
    server
        .address
        .send(ActorMessage {
            command: ActorCommand::GetChannelState(channel_name),
            reply_tx: Some(tx),
        })
        .await
        .context("Failed to send ActorMessage")?;

    let resp = rx.await.context("Failed to await actor state message")?;
    let channel = cmd_resp!(resp, CommandResponse::GetChannelState)?;

    Ok(Js(*channel))
}

#[wasm_bindgen]
pub async fn send_message(
    server_id: i32,
    channel_name: String,
    text: String,
) -> Result<Js<Message>, OrbitError> {
    let mut server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;

    let (tx, rx) = oneshot::channel();
    server
        .address
        .send(ActorMessage {
            command: ActorCommand::Privmsg {
                target: channel_name,
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
