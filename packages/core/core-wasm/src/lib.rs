use std::{fmt, str::FromStr};

use anyhow::{Context, bail};
use core_shared::{
    SendCommand,
    actor::{self, ActorCommand, ActorMessage, IrcActor},
    response_channels::CommandResponse,
    state::{
        self, Capabilities, ChannelMetadata, ChannelUser, MessageMetadata, MessageReference,
        ServerMetadata, SignedIn, User,
    },
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

use crate::database::IndexedDb;

mod database;

const DATABASE_NAME: &str = "obit-core";

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
    pub async fn state(&mut self) -> Result<Server, OrbitError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::GetState,
                reply_tx: Some(tx),
            })
            .await
            .context("Failed to send ActorMessage")?;

        let resp = rx.await.context("Failed to await actor state message")?;
        let CommandResponse::GetState(server) = resp else {
            unreachable!("expected state, got: {:?}", resp);
        };

        Ok((*server).into())
    }

    #[wasm_bindgen]
    pub fn id(&self) -> i32 {
        self.id
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
                if let Err(e) = f.call1(&JsValue::null(), &ServerEvent::from(event).into()) {
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
    ) -> Result<SignedIn, OrbitError> {
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
        let CommandResponse::SignIn(result) = resp else {
            unreachable!("expected sign in, got: {:?}", resp);
        };

        Ok(result?)
    }

    #[wasm_bindgen]
    pub async fn sign_in_anonymous(
        &mut self,
        nick: String,
        user: String,
        realname: String,
    ) -> Result<SignedIn, OrbitError> {
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

        let CommandResponse::SignIn(result) = resp else {
            unreachable!("expected sign in, got: {:?}", resp);
        };

        Ok(result?)
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
        let CommandResponse::Join(name) = resp else {
            unreachable!("expected join, got: {:?}", resp);
        };

        Ok(IrcChannel {
            name,
            address: self.address.clone(),
        })
    }

    #[wasm_bindgen]
    pub async fn history_before(
        &mut self,
        channel: String,
        before_msgid: String,
    ) -> Result<History, OrbitError> {
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
        let CommandResponse::History(history) = resp else {
            unreachable!("expected history, got: {:?}", resp);
        };

        Ok(history.into())
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
    pub async fn state(&mut self) -> Result<Option<Channel>, OrbitError> {
        let (tx, rx) = oneshot::channel();
        self.address
            .send(ActorMessage {
                command: ActorCommand::GetChannelState(self.name.clone()),
                reply_tx: Some(tx),
            })
            .await
            .context("Failed to send ActorMessage")?;

        let resp = rx.await.context("Failed to await actor state message")?;
        let CommandResponse::GetChannelState(channel) = resp else {
            unreachable!("expected state, got: {:?}", resp);
        };

        Ok((*channel).map(Into::into))
    }

    #[wasm_bindgen]
    pub async fn send_message(&mut self, text: String) -> Result<Message, OrbitError> {
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

        let resp = rx.await.context("Failed to await actor message")?;
        let CommandResponse::Privmsg(message) = resp else {
            unreachable!("expected privmsg, got: {:?}", resp);
        };

        Ok((*message).into())
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

#[derive(Debug, Clone, Tsify)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct Server {
    pub id: i32,
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
        let mut channels = js_sys::Map::new_typed();
        for (k, v) in server.channels {
            channels = channels.set(&JsString::from(k), &JsValue::from(Channel::from(v)));
        }

        let mut users = js_sys::Map::new_typed();
        for (k, v) in server.users {
            users = users.set(&JsString::from(k), &JsValue::from(v));
        }

        Self {
            id: server.id,
            metadata: server.metadata,
            channels,
            capabilities: server.capabilities,
            users,
            me: server.me,
        }
    }
}

#[derive(Debug, Clone, Tsify)]
#[wasm_bindgen(getter_with_clone)]
pub struct Channel {
    pub metadata: ChannelMetadata,
    pub messages: Vec<Message>,
    pub users: Vec<ChannelUser>,
}

impl From<state::Channel> for Channel {
    fn from(channel: state::Channel) -> Self {
        Self {
            metadata: channel.metadata,
            messages: channel.messages.into_iter().map(Into::into).collect(),
            users: channel.users,
        }
    }
}

#[derive(Debug, Clone, Tsify)]
#[wasm_bindgen]
#[serde(untagged)]
pub enum ServerEvent {
    Joined(Channel),
    ChannelUpdated(ChannelMetadata),
    ServerInfo(ServerMetadata),
    UserList(UserList),
    Privmsg(ChannelMessage),
    React(React),
}

impl From<state::ServerEvent> for ServerEvent {
    fn from(event: state::ServerEvent) -> Self {
        match event {
            state::ServerEvent::Joined(c) => Self::Joined(c.into()),
            state::ServerEvent::ChannelUpdated(cm) => Self::ChannelUpdated(cm),
            state::ServerEvent::ServerInfo(sm) => Self::ServerInfo(sm),
            state::ServerEvent::UserList { channel, users } => {
                Self::UserList(UserList { channel, users })
            }
            state::ServerEvent::Privmsg { channel, message } => Self::Privmsg(ChannelMessage {
                channel,
                message: message.into(),
            }),
            state::ServerEvent::React {
                target_message,
                user,
                text,
                is_unreact,
            } => Self::React(React {
                target_message,
                user,
                text,
                is_unreact,
            }),
        }
    }
}

#[derive(Debug, Clone, Tsify)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct ChannelMessage {
    pub channel: String,
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct UserList {
    pub channel: String,
    pub users: Vec<ChannelUser>,
}

#[derive(Debug, Clone, Tsify)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct TextMessage {
    pub content: String,
    #[tsify(type = "Map<string, string[]>")]
    pub reactions: Map<JsString, JsValue>,
    pub reply: Option<MessageReference>,
    pub redacted: bool,
    pub edited: bool,
    pub relayed_by: Option<String>,
}

impl From<state::TextMessage> for TextMessage {
    fn from(message: state::TextMessage) -> Self {
        let mut reactions = js_sys::Map::new_typed();
        for (k, v) in message.reactions {
            reactions = reactions.set(&JsString::from(k), &JsValue::from(v));
        }

        Self {
            content: message.content,
            reactions,
            reply: message.reply,
            redacted: message.redacted,
            edited: message.edited,
            relayed_by: message.relayed_by,
        }
    }
}

#[derive(Debug, Clone, Tsify)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct Message {
    pub text: Option<TextMessage>,
    pub metadata: MessageMetadata,
}

impl From<state::Message> for Message {
    fn from(message: state::Message) -> Self {
        Self {
            text: message.text.map(Into::into),
            metadata: message.metadata,
        }
    }
}

#[derive(Debug, Tsify)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct OrbitError {
    pub kind: OrbitErrorKind,
    pub description: String,
}

impl From<state::OrbitError> for OrbitError {
    fn from(error: state::OrbitError) -> Self {
        let kind = match error {
            state::OrbitError::NickTaken => OrbitErrorKind::NickTaken,
            state::OrbitError::SaslFailed(_) => OrbitErrorKind::SaslFailed,
            state::OrbitError::Generic(_) => OrbitErrorKind::Generic,
            state::OrbitError::Unknown(_) => OrbitErrorKind::Unknown,
        };

        Self {
            kind,
            description: error.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub enum OrbitErrorKind {
    NickTaken,
    SaslFailed,
    Generic,
    Unknown,
}

impl From<anyhow::Error> for OrbitError {
    fn from(error: anyhow::Error) -> Self {
        Self {
            kind: OrbitErrorKind::Unknown,
            description: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct React {
    pub target_message: String,
    pub user: String,
    pub text: String,
    pub is_unreact: bool,
}

#[derive(Debug, Clone, Tsify)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct History {
    pub channel: String,
    pub messages: Vec<Message>,
}

impl From<state::History> for History {
    fn from(history: state::History) -> Self {
        Self {
            channel: history.channel,
            messages: history.messages.into_iter().map(Into::into).collect(),
        }
    }
}
