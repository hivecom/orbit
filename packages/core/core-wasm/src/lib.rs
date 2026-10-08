use std::{fmt, str::FromStr, sync::LazyLock};

use anyhow::{Context, bail};
use core_shared::{
    SendCommand,
    actor::{self, ActorCommand, ActorMessage, IrcActor},
    response_channels::CommandResponse,
    state::{Channel, ChannelInfo, History, Message, Server, SignedIn},
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
use tracing::debug;
use tsify::Ts;
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
pub async fn initialize_orbit() -> Result<Vec<Ts<Server>>, OrbitError> {
    let ids = {
        let store = SERVER_STORE.lock().await;
        store
            .servers
            .iter()
            .map(|s| &s.id)
            .copied()
            .collect::<Vec<_>>()
    };

    let mut states = Vec::new();
    for id in ids {
        states.push(Ts::from_rust(&server_state(id).await?).context("Failed to convert to Ts")?);
    }

    Ok(states)
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

async fn server_state(server_id: i32) -> Result<Server, OrbitError> {
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

    Ok(*server)
}

#[wasm_bindgen]
pub async fn server_connect(url: String) -> Result<Ts<Server>, OrbitError> {
    let id = {
        let store = SERVER_STORE.lock().await;

        store.max_id().unwrap_or(-1) + 1
    };
    let connection = IrcConnection::connect(id, url).await?;

    {
        let mut store = SERVER_STORE.lock().await;

        store.servers.push(connection.clone());
    };

    Ok(Ts::from_rust(&server_state(id).await?).context("Failed to convert to Ts")?)
}

#[wasm_bindgen]
pub async fn server_channel_list(server_id: i32) -> Result<Vec<Ts<ChannelInfo>>, OrbitError> {
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

    let list = list
        .into_iter()
        .map(|c| Ts::from_rust(&c))
        .collect::<Result<Vec<_>, tsify::Error>>()
        .context("Failed to convert to Ts")?;

    Ok(list)
}

#[wasm_bindgen]
pub async fn server_on_data(
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
            if let Err(e) = f.call1(
                &JsValue::null(),
                &Ts::from_rust(&event)
                    .map(JsValue::from)
                    .unwrap_or_else(|e| e.to_string().into()),
            ) {
                gloo_console::error!("Error during event callback: {}", e);
            }
        }
    });

    Ok(())
}

#[wasm_bindgen]
pub async fn server_on_error(
    server_id: i32,
    #[wasm_bindgen(unchecked_param_type = "(error: OrbitError) => void")] f: js_sys::Function,
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

        while let Ok(error) = handler_rx.recv().await {
            if let Err(e) = f.call1(
                &JsValue::null(),
                &Ts::from_rust(&OrbitError::from(error))
                    .map(JsValue::from)
                    .unwrap_or_else(|e| e.to_string().into()),
            ) {
                gloo_console::error!("Error during error callback: {}", e);
            }
        }
    });

    Ok(())
}

#[wasm_bindgen]
pub async fn server_on_disconnect(
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
            if let Err(e) = f.call1(&JsValue::null(), &JsValue::from_str(&event)) {
                gloo_console::error!("Error during event callback: {}", e);
            }
        }
    });

    Ok(())
}

#[wasm_bindgen]
pub async fn server_sign_in(
    server_id: i32,
    nick: String,
    user: String,
    realname: String,
    password: String,
) -> Result<Ts<SignedIn>, OrbitError> {
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

    Ok(Ts::from_rust(&result).context("Failed to convert to Ts")?)
}

#[wasm_bindgen]
pub async fn server_sign_in_anonymous(
    server_id: i32,
    nick: String,
    user: String,
    realname: String,
) -> Result<Ts<SignedIn>, OrbitError> {
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

    Ok(Ts::from_rust(&result).context("Failed to convert to Ts")?)
}

#[wasm_bindgen]
pub async fn chat_channel_join(
    server_id: i32,
    channel: String,
    password: Option<String>,
) -> Result<Ts<Channel>, OrbitError> {
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

    Ok(Ts::from_rust(&*channel).context("Failed to convert to Ts")?)
}

#[wasm_bindgen]
pub async fn chat_channel_leave(server_id: i32, channel: String) -> Result<(), OrbitError> {
    let (tx, rx) = oneshot::channel();
    let mut server =
        { SERVER_STORE.lock().await.by_id(server_id) }.ok_or(OrbitError::unknown_server())?;
    server
        .address
        .send(ActorMessage {
            command: ActorCommand::Part { channel },
            reply_tx: Some(tx),
        })
        .await
        .context("Failed to send ActorMessage")?;

    let resp = rx.await.context("Failed to await actor part message")?;
    cmd_resp!(resp, CommandResponse::Part)?;

    Ok(())
}

#[wasm_bindgen]
pub async fn chat_channel_history_before(
    server_id: i32,
    channel: String,
    before_msgid: String,
) -> Result<Ts<History>, OrbitError> {
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

    Ok(Ts::from_rust(&history).context("Failed to convert to Ts")?)
}

#[wasm_bindgen]
pub async fn chat_channel_send_message(
    server_id: i32,
    channel_name: String,
    text: String,
) -> Result<Ts<Message>, OrbitError> {
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

    Ok(Ts::from_rust(&*message).context("Failed to convert to Ts")?)
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
