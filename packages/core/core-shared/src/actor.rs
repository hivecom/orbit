#[cfg(feature = "default")]
use std::pin::pin;
use std::{fmt, time::Duration};

use futures::{FutureExt, future::FusedFuture};
use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};
#[cfg(not(feature = "web"))]
use std::time::Instant;
#[cfg(feature = "web")]
use web_time::Instant;

#[cfg(feature = "web")]
use crate::dbg;
use crate::{
    SendCommand,
    state::{
        Channel, ChannelRole, ChannelUser, History, Message, MessageMetadata, MessageReference,
        MessageType, OrbitError, Server, ServerEvent, SignedIn, Tags, TextMessage, User,
    },
};
use anyhow::{Context, anyhow};
use base64::prelude::*;
use futures::{
    SinkExt, StreamExt,
    channel::{
        mpsc::{self, UnboundedSender},
        oneshot,
    },
    stream::FusedStream,
};
use irc_proto::{BatchSubCommand, CapSubCommand, Command::*, Message as IrcMessage, Response};
use ordermap::OrderMap;
use tracing::{debug, error, warn};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKey {
    RequestCaps,
    SignIn,
    Join(String),
    Privmsg { target: String, text: String },
    History,
    Label(String),
}

#[derive(Debug)]
pub enum CommandResponse {
    GetState(Box<Server>),
    GetChannelState(Box<Option<Channel>>),
    Capabilities,
    SignIn(Result<SignedIn, OrbitError>),
    Join(String),
    Privmsg(Box<Message>),
    History(History),
}

const LABEL_CHARSET: &str = "abcdefghijklmnopqrstuvwxyz\
                               ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                               1234567890";

fn generate_label(rng: &mut SmallRng) -> String {
    let char_vec = LABEL_CHARSET
        .split("")
        .filter(|c| !c.is_empty())
        .collect::<Vec<&str>>();

    std::iter::repeat_with(|| char_vec.choose(rng).expect("CHARSET is not empty"))
        .take(10)
        .copied()
        .collect::<Vec<_>>()
        .join("")
}

#[derive(Debug)]
pub struct ResponseChannels {
    channels: Vec<(CommandKey, Instant, oneshot::Sender<CommandResponse>)>,
    rng: SmallRng,
}

impl Default for ResponseChannels {
    #[tracing::instrument]
    fn default() -> Self {
        Self {
            channels: Vec::new(),
            rng: SmallRng::from_seed([0; 32]),
        }
    }
}

impl ResponseChannels {
    #[tracing::instrument]
    pub fn register(&mut self, key: CommandKey, os_tx: oneshot::Sender<CommandResponse>) {
        self.channels.push((key, Instant::now(), os_tx));
    }

    #[tracing::instrument]
    pub fn register_labeled(&mut self, os_tx: oneshot::Sender<CommandResponse>) -> String {
        let label = generate_label(&mut self.rng);

        self.channels
            .push((CommandKey::Label(label.clone()), Instant::now(), os_tx));

        label
    }

    #[tracing::instrument]
    pub fn reply(
        &mut self,
        key: &CommandKey,
        response: CommandResponse,
    ) -> Result<bool, CommandResponse> {
        if let CommandKey::Label(label) = key {
            dbg!(label);
        }

        if let Some(idx) = self.channels.iter().position(|(rk, _, _)| rk == key) {
            let (_, _, ch) = self.channels.remove(idx);
            ch.send(response)?;

            Ok(true)
        } else {
            // warn!("Failed to find response channel");
            Ok(false)
        }
    }

    pub fn check_timeouts(&mut self) {
        self.channels
            .retain(|(_, creation, _)| creation.elapsed() < Duration::from_secs(1));
    }
}

#[derive(Debug)]
pub struct ActorMessage {
    pub command: ActorCommand,
    pub reply_tx: Option<oneshot::Sender<CommandResponse>>,
}

#[derive(Debug)]
pub enum ActorCommand {
    GetState,
    GetChannelState(String),
    SignIn {
        nick: String,
        user: String,
        realname: String,
        password: String,
    },
    SignInAnonymous {
        nick: String,
        user: String,
        realname: String,
    },
    Join {
        channel: String,
        password: Option<String>,
    },
    Privmsg {
        text: String,
        target: String,
    },
    AddEventHandler {
        handler: UnboundedSender<ServerEvent>,
    },
    AddErrorHandler {
        handler: UnboundedSender<OrbitError>,
    },
    AddDisconectHandler {
        handler: UnboundedSender<String>,
    },
    RequestHistory {
        channel: String,
        before_msgid: String,
    },
}

pub trait IrcConnection: fmt::Debug {
    type Incoming: FusedStream<Item = anyhow::Result<IrcMessage>> + Unpin;
    type Outgoing: SendCommand;

    fn in_out(self) -> (Self::Incoming, Self::Outgoing);
    fn address(&self) -> &str;
}

struct RequestedHistory {
    channel: String,
    label: Option<String>,
}

#[derive(Debug)]
struct CurrentBatch {
    id: String,
    data: BatchData,
}

#[derive(Debug)]
enum BatchData {
    History {
        label: Option<String>,
        channel: String,
        messages: Vec<Message>,
    },
    Unhandled,
}

impl CurrentBatch {
    fn is_chathistory(&self) -> bool {
        matches!(self.data, BatchData::History { .. })
    }
}

#[derive(Default, Clone)]
enum SaslState {
    #[default]
    Unauthed,
    Requested {
        nickname: String,
        realname: String,
        username: String,
        password: String,
    },
}

pub struct IrcActor<C: IrcConnection> {
    cmd_rx: mpsc::UnboundedReceiver<ActorMessage>,
    incoming: C::Incoming,
    outgoing: C::Outgoing,
    state: Server,
    response_channels: ResponseChannels,
    event_handlers: Vec<UnboundedSender<ServerEvent>>,
    error_handlers: Vec<UnboundedSender<OrbitError>>,
    disconnect_handlers: Vec<UnboundedSender<String>>,

    current_batch: Option<CurrentBatch>,
    requested_history_batches: Vec<RequestedHistory>,
    sasl_state: SaslState,
    rng: SmallRng,
}

impl<C: IrcConnection> IrcActor<C> {
    #[tracing::instrument]
    pub async fn start(
        id: i32,
        connection: C,
        spawn: fn(IrcActor<C>) -> (),
    ) -> Result<UnboundedSender<ActorMessage>, OrbitError> {
        let address = connection.address().to_string();
        let (incoming, outgoing) = connection.in_out();

        let (cmd_tx, cmd_rx) = mpsc::unbounded();
        let mut actor = Self {
            cmd_rx,
            incoming,
            outgoing,
            state: Server::new(id, address),
            response_channels: ResponseChannels::default(),
            event_handlers: Default::default(),
            error_handlers: Default::default(),
            disconnect_handlers: Default::default(),
            current_batch: Default::default(),
            requested_history_batches: Default::default(),
            sasl_state: Default::default(),
            rng: SmallRng::from_seed([1; 32]),
        };

        let (tx, rx) = oneshot::channel();
        actor
            .response_channels
            .register(CommandKey::RequestCaps, tx);
        actor.request_caps().await?;

        spawn(actor);

        rx.await.unwrap();

        Ok(cmd_tx)
    }

    #[tracing::instrument(skip(self))]
    pub async fn run(mut self) {
        fn create_timeout() -> impl FusedFuture {
            #[cfg(feature = "web")]
            let mut timeout = gloo_timers::future::TimeoutFuture::new(1000).fuse();
            #[cfg(not(feature = "web"))]
            let mut timeout = pin!(tokio::time::sleep(Duration::from_secs(1)).fuse());

            timeout
        }

        let mut timeout = create_timeout();

        loop {
            futures::select! {
                msg = self.incoming.next() => {
                        match msg {
                            Some(m) => {
                            match m {
                                Ok(m) => self.handle_incoming(m).await.unwrap(),
                                Err(e) => {
                                    self.on_disconnect(e.to_string()).await.unwrap();
                                    break;
                                    }
                            };
                        }
                        None => {
                            self.on_disconnect("Connection Closed".into()).await.unwrap();
                            break;
                        }
                    }
                }
                cmd = self.cmd_rx.select_next_some() => {
                    self.handle_command(cmd).await.unwrap();
                }
                _ = timeout => {
                    self.response_channels.check_timeouts();

                    timeout = create_timeout();
                }
            }
        }
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn handle_incoming(&mut self, mut message: IrcMessage) -> Result<(), OrbitError> {
        match message.command {
            CAP(_, sub, param, caps) => {
                self.handle_caps(sub, param, caps).await?;
            }
            PING(server1, server2) => {
                self.outgoing
                    .pong(server1, server2)
                    .await
                    .context("Failed to send pong")?;
            }
            Response(rpl, params) => {
                self.handle_response(rpl, params).await?;
            }
            JOIN(ref channel_name, _, _) => {
                let source = message.source_nickname().unwrap();

                let mut tags = Tags::default();
                if let Some(ref t) = message.tags {
                    tags = Tags::parse(t);
                }

                if self.current_batch.as_ref().map(|b| b.is_chathistory()) == Some(true) {
                    assert!(tags.server_time.is_some());
                }

                let state_message = Message {
                    text: None,
                    metadata: MessageMetadata {
                        msgid: tags.msgid_with_fallback(&["JOIN", source]),
                        server_time: tags.server_time_with_fallback() as f64,
                        message_type: MessageType::Join,
                        user: source.to_string(),
                    },
                };

                if self
                    .push_batch(channel_name.clone(), state_message.clone())
                    .await
                {
                    return Ok(());
                }

                if source == self.state.me.as_ref().unwrap().nickname {
                    let channel = Channel::new(channel_name.clone());
                    self.state
                        .channels
                        .insert(channel_name.clone(), channel.clone());

                    if !self.state.capabilities.history.enabled {
                        self.response_channels
                            .reply(
                                &CommandKey::Join(channel_name.clone()),
                                CommandResponse::Join(channel_name.clone()),
                            )
                            .map_err(|e| anyhow!("Failed to reply to JOIN command {e:?}"))?;
                    }

                    self.on_event(ServerEvent::Joined(channel)).await?;

                    if self.state.capabilities.history.enabled {
                        let label = if self.state.capabilities.labeled_response.enabled {
                            Some(generate_label(&mut self.rng))
                        } else {
                            None
                        };

                        self.requested_history_batches.push(RequestedHistory {
                            channel: channel_name.clone(),
                            label: label.clone(),
                        });
                        self.history_latest(channel_name.clone(), None, 5, label)
                            .await
                            .context("Failed to request latest history")?;
                    }
                } else {
                    let channel = self.channel_mut(channel_name.clone()).await;

                    channel.users.push(ChannelUser {
                        nickname: source.to_string(),
                        role: ChannelRole::Regular,
                    });
                }

                self.on_event(ServerEvent::Privmsg {
                    channel: channel_name.to_string(),
                    message: state_message,
                })
                .await?;
            }
            PART(ref channel_name, ref comment) => {
                let mut tags = Tags::default();
                if let Some(ref t) = message.tags {
                    tags = Tags::parse(t);
                }
                let source = message.source_nickname().unwrap();

                let state_message = Message {
                    text: None,
                    metadata: MessageMetadata {
                        msgid: tags.msgid_with_fallback(&["PART", source]),
                        server_time: tags.server_time_with_fallback() as f64,
                        message_type: MessageType::Part,
                        user: source.to_string(),
                    },
                };

                if self
                    .push_batch(channel_name.clone(), state_message.clone())
                    .await
                {
                    return Ok(());
                }

                self.on_event(ServerEvent::Privmsg {
                    channel: channel_name.to_string(),
                    message: state_message,
                })
                .await?;

                let channel = self.channel_mut(channel_name.clone()).await;

                channel.users.retain(|u| u.nickname != source);
            }
            QUIT(ref comment) => {
                let mut tags = Tags::default();
                if let Some(ref t) = message.tags {
                    tags = Tags::parse(t);
                }
                let source = message.source_nickname().unwrap();

                let state_message = Message {
                    text: None,
                    metadata: MessageMetadata {
                        msgid: tags.msgid_with_fallback(&["QUIT", source]),
                        server_time: tags.server_time_with_fallback() as f64,
                        message_type: MessageType::Quit,
                        user: source.to_string(),
                    },
                };

                if let Some(batch) = self.current_batch.as_mut()
                    && let BatchData::History { messages, .. } = &mut batch.data
                {
                    messages.push(state_message);
                    return Ok(());
                }

                self.on_event(ServerEvent::Privmsg {
                    channel: String::new(),
                    message: state_message,
                })
                .await?;

                self.state.users.remove(source);
                for channel in self.state.channels.values_mut() {
                    channel.users.retain(|u| u.nickname != source);
                }
            }
            PRIVMSG(ref target, ref text) => {
                let mut tags = Tags::default();
                if let Some(ref t) = message.tags {
                    tags = Tags::parse(t);
                }
                assert_eq!(
                    self.current_batch.as_ref().map(|s| s.id.as_str()),
                    tags.batch.as_deref(),
                );

                let source = message.source_nickname().unwrap();

                let msgid = tags.msgid_with_fallback(&["PRIVMSG", source, target, text]);

                let reply = tags
                    .reply
                    .as_ref()
                    .map(|r| {
                        self.state
                            .channels
                            .get(target)
                            .and_then(|c| c.messages.get(r))
                    })
                    .map(|m| MessageReference {
                        text: m.and_then(|m| m.text.clone().map(|t| t.content)),
                        username: m.map(|m| m.metadata.user.clone()),
                    });

                let state_message = Message {
                    metadata: MessageMetadata {
                        msgid: msgid.clone(),
                        server_time: tags.server_time_with_fallback() as f64,
                        message_type: MessageType::Privmsg,
                        user: source.to_string(),
                    },
                    text: Some(TextMessage {
                        content: text.clone(),
                        reactions: OrderMap::new(),
                        reply,
                        redacted: false,
                        edited: false,
                        relayed_by: tags.relayed_by,
                    }),
                };

                if self.push_batch(target.clone(), state_message.clone()).await {
                    return Ok(());
                }

                if let Some(username) = tags.account.clone() {
                    let user = self.user_mut(source.to_string()).await;
                    user.username = Some(username);
                }

                let channel = self.channel_mut(target.clone()).await;
                channel.messages.insert(msgid, state_message.clone());

                if source == self.state.me.as_ref().unwrap().nickname
                    && let Err(e) = self.response_channels.reply(
                        &CommandKey::Privmsg {
                            target: target.clone(),
                            text: text.clone(),
                        },
                        CommandResponse::Privmsg(Box::new(state_message.clone())),
                    )
                {
                    error!("Failed to reply to PRIVMSG command {e:?}");
                }

                self.on_event(ServerEvent::Privmsg {
                    channel: target.clone(),
                    message: state_message,
                })
                .await?;
            }
            BATCH(reference, typ, param) => {
                if let Some(id) = reference.strip_prefix('+') {
                    let mut tags = Tags::default();
                    if let Some(ref t) = message.tags {
                        tags = Tags::parse(t);
                    }

                    match typ {
                        Some(BatchSubCommand::CUSTOM(c)) if &c == "CHATHISTORY" => {
                            let idx = self
                                .requested_history_batches
                                .iter()
                                .position(|b| b.label == tags.label)
                                .expect("Chat history was requested");
                            let channel = self.requested_history_batches.remove(idx).channel;

                            self.current_batch = Some(CurrentBatch {
                                id: id.to_string(),
                                data: BatchData::History {
                                    label: tags.label,
                                    channel,
                                    messages: Vec::new(),
                                },
                            });
                        }
                        _ => {
                            self.current_batch = Some(CurrentBatch {
                                id: id.to_string(),
                                data: BatchData::Unhandled,
                            });
                            warn!(?typ, ?param, "unhandled BATCH type");
                        }
                    }
                } else {
                    assert_eq!(
                        self.current_batch.as_ref().map(|b| b.id.as_str()),
                        Some(&reference[1..])
                    );

                    if let Some(batch) = self.current_batch.take()
                        && let BatchData::History {
                            label,
                            channel: channel_name,
                            messages,
                        } = batch.data
                    {
                        let channel = self.channel_mut(channel_name.clone()).await;
                        for message in &messages {
                            channel
                                .messages
                                .insert(message.metadata.msgid.clone(), message.clone());
                        }

                        let history = History {
                            channel: channel_name.clone(),
                            messages,
                        };

                        let key = if let Some(label) = label {
                            CommandKey::Label(label)
                        } else {
                            CommandKey::History
                        };

                        self.response_channels
                            .reply(
                                &CommandKey::Join(channel_name.clone()),
                                CommandResponse::Join(channel_name.clone()),
                            )
                            .map_err(|e| anyhow!("Failed to reply to JOIN command {e:?}"))?;

                        self.response_channels
                            .reply(&key, CommandResponse::History(history.clone()))
                            .unwrap();
                    }

                    self.current_batch = None;
                }
            }
            ChannelMODE(ref channel_name, ref mode) => {
                let target = message.response_target().unwrap();
                let source = message.source_nickname().unwrap();

                if self.current_batch.as_ref().map(|b| b.is_chathistory()) != Some(true) {
                    dbg!(target, source, channel_name, mode);
                }
            }
            TOPIC(ref channel_name, ref text) => {
                let target = message.response_target().unwrap();
                let source = message.source_nickname().unwrap();

                if self.current_batch.as_ref().map(|b| b.is_chathistory()) != Some(true) {
                    dbg!(target, source, channel_name, text);
                }
            }
            Raw(ref cmd, ref mut target) if cmd == "TAGMSG" => {
                let target = target.remove(0);

                let mut tags = Tags::default();
                if let Some(ref t) = message.tags {
                    tags = Tags::parse(t);
                }

                let is_unreact = tags.unreact.is_some();
                if let Some(react) = tags.react.or(tags.unreact)
                    && let Some(reply) = tags.reply
                {
                    let channel = self.channel_mut(target.clone()).await;

                    let nickname = message.source_nickname().unwrap().to_string();
                    if let Some(message) = channel.messages.get_mut(&reply) {
                        let reactors = message
                            .text
                            .as_mut()
                            .unwrap()
                            .reactions
                            .entry(react.clone())
                            .or_insert_with(Vec::new);

                        if is_unreact {
                            reactors.push(nickname.clone());
                        } else {
                            reactors.retain(|v| *v != nickname);
                        }

                        // TODO: should it be sent if the message wasn't found?
                        self.on_event(ServerEvent::React {
                            target_message: reply,
                            user: nickname,
                            text: react,
                            is_unreact,
                        })
                        .await?;
                    }
                }
            }
            AUTHENTICATE(param) if param == "+" => {
                if let SaslState::Requested {
                    nickname,
                    realname,
                    username,
                    password,
                } = self.sasl_state.clone()
                {
                    let credentials =
                        BASE64_STANDARD.encode(format!("\0{}\0{}", username, password).as_bytes());

                    // Chunk overly long credentials
                    let mut sending = credentials.as_str();
                    while !sending.is_empty() {
                        let (chunk, rest) = sending.split_at(400.min(sending.len()));
                        self.sasl(chunk.to_string())
                            .await
                            .context("Failed to send SASL chunk")?;

                        if rest.is_empty() && chunk.len() == 400 {
                            self.sasl("+".to_string())
                                .await
                                .context("Failed to send SASL end")?;
                        }
                        sending = rest;
                    }
                    self.nick(nickname.clone())
                        .await
                        .context("Failed to send NICK")?;
                    self.user(username.clone(), String::from("0"), realname.clone())
                        .await
                        .context("Failed to send USER")?;

                    self.state.me = Some(User {
                        nickname,
                        username: Some(username),
                        realname: Some(realname),
                        display_name: None,
                        description: None,
                        profile_picture_url: None,
                        bot: false,
                    });
                }
            }
            ERROR(msg) => self.on_error(OrbitError::Generic(msg)).await?,
            _ => {
                warn!("unhandled message, {message:?}");
            }
        }

        Ok(())
    }

    pub async fn push_batch(&mut self, target: String, state_message: Message) -> bool {
        if let Some(batch) = self.current_batch.as_mut()
            && let BatchData::History {
                channel, messages, ..
            } = &mut batch.data
        {
            if channel.is_empty() {
                *channel = target.clone();
            } else {
                assert_eq!(*channel, target)
            }
            messages.push(state_message);

            return true;
        }

        false
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn handle_caps(
        &mut self,
        sub: CapSubCommand,
        param: Option<String>,
        caps: Option<String>,
    ) -> Result<(), OrbitError> {
        match sub {
            CapSubCommand::LS if let Some(caps) = caps => {
                for cap in caps.split_whitespace() {
                    let cap = cap
                        .split('=')
                        .next()
                        .ok_or_else(|| anyhow!("Cap is empty: \"{}\"", cap))?;
                    self.state.capabilities.set_from_name(cap, None);
                }
            }
            CapSubCommand::LS if let Some(param) = param => {
                if param == "*" {
                    return Ok(());
                }
                for cap in param.split_whitespace() {
                    let cap = cap
                        .split('=')
                        .next()
                        .ok_or_else(|| anyhow!("Cap is empty: \"{}\"", cap))?;
                    self.state.capabilities.set_from_name(cap, None);
                }
            }
            CapSubCommand::ACK if let Some(param) = param => {
                for cap in param.split_whitespace() {
                    self.state.capabilities.set_from_name(cap, Some(true));
                }
                self.response_channels
                    .reply(&CommandKey::RequestCaps, CommandResponse::Capabilities)
                    .unwrap();
            }
            _ => {
                debug!("unhandled caps message");
            }
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn handle_response(
        &mut self,
        rpl: Response,
        params: Vec<String>,
    ) -> Result<(), OrbitError> {
        match rpl {
            Response::RPL_MOTDSTART => {
                self.state.metadata.reset_motd();
            }
            Response::RPL_MOTD => {
                self.state.metadata.add_motd(&params[1]);
            }
            Response::RPL_ENDOFMOTD => self
                .on_event(ServerEvent::ServerInfo(self.state.metadata.clone()))
                .await
                .map_err(|e| anyhow!("Failed to send server event {e:?}"))?,
            Response::RPL_SASLSUCCESS => {
                self.response_channels
                    .reply(
                        &CommandKey::SignIn,
                        CommandResponse::SignIn(Ok(SignedIn::User)),
                    )
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;

                self.cap_end().await.context("Failed to send CAP END")?;
            }
            Response::RPL_WELCOME => {
                self.response_channels
                    .reply(
                        &CommandKey::SignIn,
                        CommandResponse::SignIn(Ok(SignedIn::Guest)),
                    )
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;
            }
            Response::RPL_LOGGEDIN => {
                self.state.me.as_mut().unwrap().username = Some(params[2].clone());
            }
            Response::ERR_SASLFAIL => {
                self.response_channels
                    .reply(
                        &CommandKey::SignIn,
                        CommandResponse::SignIn(Err(OrbitError::SaslFailed(params[1].to_string()))),
                    )
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;
            }
            Response::ERR_NICKNAMEINUSE => {
                self.response_channels
                    .reply(
                        &CommandKey::SignIn,
                        CommandResponse::SignIn(Err(OrbitError::NickTaken)),
                    )
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;
            }
            Response::RPL_TOPIC => {
                let channel_name = params[1].to_string();
                let topic = params[2].to_string();
                let channel = self.channel_mut(channel_name).await;

                channel.metadata.topic = Some(topic);

                let metadata = channel.metadata.clone();
                self.on_event(ServerEvent::ChannelUpdated(metadata))
                    .await
                    .unwrap();
            }
            Response::RPL_NAMREPLY => {
                let channel_name = params[2].to_string();
                let users: Vec<_> = params[3].split_whitespace().map(|u| u.to_owned()).collect();

                let mut channel_users = Vec::new();
                for mut user in users {
                    if let Some(prefix) = &self.state.support.prefix
                        && let Some((role, _)) =
                            prefix.iter().find(|(_, p)| Some(*p) == user.chars().nth(0))
                    {
                        user.remove(0);
                        let role = ChannelRole::from(*role);
                        channel_users.push(ChannelUser {
                            role,
                            nickname: user.clone(),
                        });
                    } else {
                        channel_users.push(ChannelUser {
                            role: ChannelRole::Regular,
                            nickname: user.clone(),
                        });
                    }

                    self.state
                        .users
                        .entry(user.clone())
                        .or_insert_with(|| User::new(user));
                }

                let channel = self.channel_mut(channel_name).await;

                channel.users = channel_users;
            }
            Response::RPL_ENDOFNAMES => self
                .on_event(ServerEvent::UserList {
                    channel: params[1].to_string(),
                    users: self.state.channels.get(&params[1]).unwrap().users.clone(),
                })
                .await
                .map_err(|e| anyhow!("Failed to send server event {e:?}"))?,
            Response::RPL_ISUPPORT => {
                for option in &params[1..(params.len() - 1)] {
                    let (key, value) = option.split_once('=').unzip();
                    self.state.support.set(key.unwrap_or(option), value);
                }
                self.state.metadata.name = self.state.support.network.clone();
            }
            Response::RPL_YOURHOST
            | Response::RPL_CREATED
            | Response::RPL_MYINFO
            | Response::RPL_LUSERCLIENT
            | Response::RPL_LUSEROP
            | Response::RPL_LUSERUNKNOWN
            | Response::RPL_LUSERCHANNELS
            | Response::RPL_LUSERME
            | Response::RPL_TOPICWHOTIME
            | Response::RPL_LOCALUSERS
            | Response::RPL_UMODEIS
            | Response::RPL_GLOBALUSERS => (),
            _ => {
                warn!("unhandled response");
            }
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn handle_command(&mut self, cmd: ActorMessage) -> Result<(), OrbitError> {
        match cmd.command {
            ActorCommand::GetState => cmd
                .reply_tx
                .unwrap()
                .send(CommandResponse::GetState(Box::new(self.state.clone())))
                .unwrap(),
            ActorCommand::GetChannelState(channel_name) => cmd
                .reply_tx
                .unwrap()
                .send(CommandResponse::GetChannelState(Box::new(
                    self.state.channels.get(&channel_name).cloned(),
                )))
                .unwrap(),
            ActorCommand::SignIn {
                nick,
                user,
                realname,
                password,
            } => {
                self.response_channels
                    .register(CommandKey::SignIn, cmd.reply_tx.unwrap());
                self.sign_in(nick, user, realname, password).await?;
            }
            ActorCommand::SignInAnonymous {
                nick,
                user,
                realname,
            } => {
                self.response_channels
                    .register(CommandKey::SignIn, cmd.reply_tx.unwrap());
                self.sign_in_anonymous(nick, user, realname).await?;
            }
            ActorCommand::Join { channel, password } => {
                self.response_channels
                    .register(CommandKey::Join(channel.clone()), cmd.reply_tx.unwrap());
                self.join(channel, password).await.unwrap();
            }
            ActorCommand::Privmsg { text, target } => {
                self.response_channels.register(
                    CommandKey::Privmsg {
                        target: target.clone(),
                        text: text.clone(),
                    },
                    cmd.reply_tx.unwrap(),
                );
                self.privmsg(target, text).await.unwrap();
            }
            ActorCommand::AddEventHandler { handler } => {
                self.event_handlers.push(handler);
            }
            ActorCommand::AddErrorHandler { handler } => {
                self.error_handlers.push(handler);
            }
            ActorCommand::AddDisconectHandler { handler } => {
                self.disconnect_handlers.push(handler);
            }
            ActorCommand::RequestHistory {
                channel,
                before_msgid,
            } => {
                let label = if self.state.capabilities.labeled_response.enabled {
                    Some(
                        self.response_channels
                            .register_labeled(cmd.reply_tx.unwrap()),
                    )
                } else {
                    self.response_channels
                        .register(CommandKey::History, cmd.reply_tx.unwrap());

                    None
                };

                self.requested_history_batches.push(RequestedHistory {
                    channel: channel.clone(),
                    label: label.clone(),
                });
                self.history_before(channel, format!("msgid={before_msgid}"), 5, label)
                    .await
                    .context("Failed to send history before")?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn on_event(&mut self, event: ServerEvent) -> Result<(), OrbitError> {
        for handler in &mut self.event_handlers {
            handler.send(event.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn on_error(&mut self, error: OrbitError) -> Result<(), OrbitError> {
        for handler in &mut self.error_handlers {
            handler.send(error.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn on_disconnect(&mut self, reason: String) -> Result<(), OrbitError> {
        for handler in &mut self.disconnect_handlers {
            handler.send(reason.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    async fn request_caps(&mut self) -> Result<(), OrbitError> {
        let irc_version = String::from("302");
        self.cap_ls(irc_version)
            .await
            .context("Failed to send CAPS LS")?;
        self.cap_req(&[
            "echo-message",
            "labeled-response",
            "message-tags",
            "sasl",
            "draft/message-redaction",
            "draft/metadata-2",
            "draft/chathistory",
            "draft/event-playback",
            "draft/account-registration",
            "draft/multiline",
            "server-time",
            "batch",
        ])
        .await
        .context("Failed to send CAP REQ")?;

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    async fn sign_in_anonymous(
        &mut self,
        nickname: String,
        username: String,
        realname: String,
    ) -> Result<(), OrbitError> {
        self.cap_end().await.context("Failed to send CAP END")?;
        self.nick(nickname.clone())
            .await
            .context("Failed to send NICK")?;
        self.user(username.clone(), String::from("0"), realname.clone())
            .await
            .context("Failed to send USER")?;

        self.state.me = Some(User {
            nickname,
            username: Some(username),
            realname: Some(realname),
            display_name: None,
            description: None,
            profile_picture_url: None,
            bot: false,
        });

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    async fn sign_in(
        &mut self,
        nickname: String,
        username: String,
        realname: String,
        password: String,
    ) -> Result<(), OrbitError> {
        if self.state.capabilities.sasl.enabled {
            self.sasl_plain()
                .await
                .context("Failed to send SASL PLAIN")?;
            self.sasl_state = SaslState::Requested {
                nickname,
                realname,
                username,
                password,
            };
        } else {
            warn!("SASL capability not enabled, falling back to anonymous sign in");
            self.sign_in_anonymous(nickname, username, realname).await?;
        }

        Ok(())
    }

    async fn channel_mut(&mut self, name: String) -> &mut Channel {
        self.state
            .channels
            .entry(name.clone())
            .or_insert_with(|| Channel::new(name))
    }

    async fn user_mut(&mut self, nickname: String) -> &mut User {
        self.state
            .users
            .entry(nickname.clone())
            .or_insert_with(|| User::new(nickname))
    }
}

impl<C: IrcConnection> SendCommand for IrcActor<C> {
    type Error = <C::Outgoing as SendCommand>::Error;
    async fn message(&mut self, command: IrcMessage) -> Result<(), Self::Error> {
        self.outgoing.message(command).await
    }
}
