use std::fmt;

#[cfg(feature = "web")]
use crate::dbg;
use crate::{
    SendCommand,
    state::{
        Channel, Message, MessageMetadata, MessageType, Server, ServerError, ServerEvent,
        TextMessage, User, UserList,
    },
};
use anyhow::anyhow;
use base64::prelude::*;
use futures::{
    SinkExt, StreamExt,
    channel::{
        mpsc::{self, UnboundedSender},
        oneshot,
    },
    stream::FusedStream,
};
use irc_proto::{CapSubCommand, Command::*, Message as IrcMessage, Response, message::Tag};
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use tracing::{debug, error, trace, warn};

#[derive(Debug, Default)]
pub struct ResponseChannels(Vec<(CommandKey, oneshot::Sender<CommandResponse>)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKey {
    RequestCaps,
    SignIn,
    Join(String),
    Privmsg { target: String, text: String },
}

#[derive(Debug)]
pub enum CommandResponse {
    GetState(Server),
    Capabilities,
    SignIn(anyhow::Result<()>),
    Join(String),
    Privmsg(Message),
}

impl ResponseChannels {
    pub fn register(&mut self, key: CommandKey, os_tx: oneshot::Sender<CommandResponse>) {
        self.0.push((key, os_tx));
    }

    #[tracing::instrument]
    pub async fn reply(
        &mut self,
        key: &CommandKey,
        response: CommandResponse,
    ) -> Result<(), CommandResponse> {
        if let Some(idx) = self.0.iter().position(|(rk, _)| rk == key) {
            let (_, ch) = self.0.remove(idx);
            ch.send(response)?;
        } else {
            trace!("Failed to find response channel");
        }

        Ok(())
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
        handler: UnboundedSender<ServerError>,
    },
    AddDisconectHandler {
        handler: UnboundedSender<String>,
    },
}

pub trait IrcConnection: fmt::Debug {
    type Incoming: FusedStream<Item = anyhow::Result<IrcMessage>> + Unpin;
    type Outgoing: SendCommand;

    fn in_out(self) -> (Self::Incoming, Self::Outgoing);
    fn address(&self) -> &str;
}

#[derive(Debug)]
pub struct IrcActor<C: IrcConnection> {
    cmd_rx: mpsc::UnboundedReceiver<ActorMessage>,
    incoming: C::Incoming,
    outgoing: C::Outgoing,
    state: Server,
    response_channels: ResponseChannels,
    event_handlers: Vec<UnboundedSender<ServerEvent>>,
    error_handlers: Vec<UnboundedSender<ServerError>>,
    disconnect_handlers: Vec<UnboundedSender<String>>,
}

impl<C: IrcConnection> IrcActor<C> {
    #[tracing::instrument]
    pub async fn new(
        name: String,
        connection: C,
        spawn: fn(IrcActor<C>) -> (),
    ) -> anyhow::Result<UnboundedSender<ActorMessage>> {
        let address = connection.address().to_string();
        let (incoming, outgoing) = connection.in_out();

        let (cmd_tx, cmd_rx) = mpsc::unbounded();
        let mut actor = Self {
            cmd_rx,
            incoming,
            outgoing,
            state: Server::new(name, address),
            response_channels: ResponseChannels::default(),
            event_handlers: Vec::new(),
            error_handlers: Vec::new(),
            disconnect_handlers: Vec::new(),
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
            }
        }
    }

    #[tracing::instrument(err, skip(self, message))]
    pub async fn handle_incoming(&mut self, message: IrcMessage) -> anyhow::Result<()> {
        match message.command {
            CAP(_, sub, param, caps) => {
                self.handle_caps(sub, param, caps).await?;
            }
            PING(server1, server2) => {
                self.outgoing.pong(server1, server2).await?;
            }
            Response(rpl, params) => {
                self.handle_response(rpl, params).await?;
            }
            JOIN(channel_name, _, _) => {
                let channel = Channel::new(channel_name.clone());
                self.state
                    .channels
                    .insert(channel_name.clone(), channel.clone());
                self.response_channels
                    .reply(
                        &CommandKey::Join(channel_name.clone()),
                        CommandResponse::Join(channel_name),
                    )
                    .await
                    .map_err(|e| anyhow!("Failed to reply to JOIN command {e:?}"))?;

                self.on_event(ServerEvent::Joined(channel)).await?;
            }
            PRIVMSG(ref target, ref text) => {
                let mut msgid = None;
                let mut server_time = None;
                let mut username = None;
                if let Some(ref tags) = message.tags {
                    for Tag(key, value) in tags {
                        match key.as_str() {
                            "msgid" => msgid = value.clone(),
                            "account" => username = value.clone(),
                            "time" => {
                                server_time = value
                                    .as_ref()
                                    .and_then(|v| OffsetDateTime::parse(&v, &Iso8601::DEFAULT).ok())
                            }
                            _ => {
                                warn!("unhandled tag: {key:?}: {value:?}");
                            }
                        }
                    }
                }

                let server_time = server_time
                    .unwrap_or_else(|| OffsetDateTime::now_utc())
                    .unix_timestamp();
                let msgid = msgid.unwrap_or_else(|| {
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(&server_time.to_ne_bytes());
                    hasher.update(&target.as_bytes());
                    hasher.update(&text.as_bytes());

                    hasher.finalize().to_string()
                });

                let nickname = message.source_nickname().unwrap();
                let user = self
                    .state
                    .users
                    .entry(nickname.to_string())
                    .or_insert_with(|| User::new(nickname.to_string()));
                if username.is_some() {
                    user.username = username;
                }

                let state_message = Message {
                    text: Some(TextMessage {
                        content: text.clone(),
                        reactions: Vec::new(),
                        reply: None,
                        redacted: false,
                        edited: false,
                    }),
                    metadata: MessageMetadata {
                        msgid,
                        server_time: server_time as f64,
                        message_type: MessageType::Privmsg,
                        user: nickname.to_string(),
                    },
                };

                if nickname == self.state.me.as_ref().unwrap().nickname {
                    if let Err(e) = self
                        .response_channels
                        .reply(
                            &CommandKey::Privmsg {
                                target: target.clone(),
                                text: text.clone(),
                            },
                            CommandResponse::Privmsg(state_message.clone()),
                        )
                        .await
                    {
                        error!("Failed to reply to PRIVMSG command {e:?}");
                    }
                }

                self.on_event(ServerEvent::Privmsg(state_message)).await?;
            }
            ERROR(msg) => self.on_error(ServerError::Generic(msg)).await?,
            AUTHENTICATE(_) => (),
            _ => {
                warn!("unhandled message, {message:?}");
            }
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn handle_caps(
        &mut self,
        sub: CapSubCommand,
        param: Option<String>,
        caps: Option<String>,
    ) -> anyhow::Result<()> {
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
                    .await
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
    ) -> anyhow::Result<()> {
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
            Response::RPL_SASLSUCCESS | Response::RPL_WELCOME => {
                self.response_channels
                    .reply(&CommandKey::SignIn, CommandResponse::SignIn(Ok(())))
                    .await
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;
            }
            Response::RPL_LOGGEDIN => {
                self.state.me.as_mut().unwrap().username = Some(params[2].clone());
            }
            Response::ERR_SASLFAIL => {
                self.response_channels
                    .reply(
                        &CommandKey::SignIn,
                        CommandResponse::SignIn(Err(anyhow!("{}", &params[1]))),
                    )
                    .await
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;
            }
            Response::ERR_NICKNAMEINUSE => {
                self.response_channels
                    .reply(
                        &CommandKey::SignIn,
                        CommandResponse::SignIn(Err(anyhow!("{}", &params[2]))),
                    )
                    .await
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;
            }
            Response::RPL_TOPIC => {
                let channel_name = params[1].to_string();
                let topic = params[2].to_string();
                let channel = self
                    .state
                    .channels
                    .entry(channel_name.clone())
                    .or_insert_with(|| Channel::new(channel_name));

                channel.metadata.topic = Some(topic);

                let metadata = channel.metadata.clone();
                self.on_event(ServerEvent::ChannelUpdated(metadata))
                    .await
                    .unwrap();
            }
            Response::RPL_NAMREPLY => {
                let channel_name = params[2].to_string();
                let users: Vec<_> = params[3].split_whitespace().map(|u| u.to_owned()).collect();
                let channel = self
                    .state
                    .channels
                    .entry(channel_name.clone())
                    .or_insert_with(|| Channel::new(channel_name));

                channel.users = users;
            }
            Response::RPL_ENDOFNAMES => self
                .on_event(ServerEvent::UserList(UserList {
                    channel: params[1].to_string(),
                    users: self.state.channels.get(&params[1]).unwrap().users.clone(),
                }))
                .await
                .map_err(|e| anyhow!("Failed to send server event {e:?}"))?,
            Response::RPL_ISUPPORT => {
                for option in &params[1..(params.len() - 1)] {
                    let (key, value) = option.split_once('=').unzip();
                    self.state.support.set(key.unwrap_or(option), value);
                }
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
    pub async fn handle_command(&mut self, cmd: ActorMessage) -> anyhow::Result<()> {
        match cmd.command {
            ActorCommand::GetState => cmd
                .reply_tx
                .unwrap()
                .send(CommandResponse::GetState(self.state.clone()))
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
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn on_event(&mut self, event: ServerEvent) -> anyhow::Result<()> {
        for handler in &mut self.event_handlers {
            handler.send(event.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn on_error(&mut self, error: ServerError) -> anyhow::Result<()> {
        for handler in &mut self.error_handlers {
            handler.send(error.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn on_disconnect(&mut self, reason: String) -> anyhow::Result<()> {
        for handler in &mut self.disconnect_handlers {
            handler.send(reason.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    async fn request_caps(&mut self) -> anyhow::Result<()> {
        let irc_version = String::from("302");
        self.ls_caps(irc_version).await?;
        self.req_caps(&[
            "echo-message",
            "message-tags",
            "sasl",
            "draft/message-redaction",
            "draft/metadata-2",
            "draft/chathistory",
            "draft/event-playback",
            "draft/account-registration",
            "draft/multiline",
            "server-time",
        ])
        .await?;

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    async fn sign_in_anonymous(
        &mut self,
        nick: String,
        user: String,
        realname: String,
    ) -> anyhow::Result<()> {
        self.end_caps().await?;
        self.nick(nick.clone()).await?;
        self.user(user.clone(), String::from("0"), realname.clone())
            .await?;

        self.state.me = Some(User {
            nickname: nick,
            username: Some(user),
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
    ) -> anyhow::Result<()> {
        if self.state.capabilities.sasl.enabled {
            self.sasl_plain().await?;
            let credentials =
                BASE64_STANDARD.encode(format!("\0{}\0{}", username, password).as_bytes());

            // Chunk overly long credentials
            let mut sending = credentials.as_str();
            while !sending.is_empty() {
                let (chunk, rest) = sending.split_at(400.min(credentials.len()));
                self.sasl(chunk.to_string()).await?;

                if rest.is_empty() && chunk.len() == 400 {
                    self.sasl("+".to_string()).await?;
                }
                sending = rest;
            }
        } else {
            unimplemented!("sasl cap not enabled")
        }
        self.end_caps().await?;
        self.nick(nickname.clone()).await?;
        self.user(username.clone(), String::from("0"), realname.clone())
            .await?;

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
}

impl<C: IrcConnection> SendCommand for IrcActor<C> {
    type Error = <C::Outgoing as SendCommand>::Error;
    async fn message(&mut self, command: IrcMessage) -> Result<(), Self::Error> {
        self.outgoing.message(command).await
    }
}
