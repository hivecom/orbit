use std::fmt;

#[cfg(feature = "web")]
use crate::dbg;
use crate::{
    SendCommand,
    state::{
        Channel, ChannelMetadata, Message, MessageMetadata, MessageType, Server, ServerError,
        ServerEvent, TextMessage, User,
    },
};
use anyhow::anyhow;
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
use tracing::{debug, error};

#[derive(Debug, Default)]
pub struct ResponseChannels(Vec<(CommandKey, oneshot::Sender<CommandResponse>)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKey {
    Register,
    Join(String),
    Privmsg { target: String, text: String },
}

#[derive(Debug)]
pub enum CommandResponse {
    Register(Server),
    Join(String),
    Privmsg(Message),
}

impl ResponseChannels {
    pub fn register(&mut self, key: CommandKey, os_tx: oneshot::Sender<CommandResponse>) {
        self.0.push((key, os_tx));
    }

    pub async fn reply(
        &mut self,
        key: &CommandKey,
        response: CommandResponse,
    ) -> Result<(), CommandResponse> {
        if let Some(idx) = self.0.iter().position(|(rk, _)| rk == key) {
            let (_, ch) = self.0.remove(idx);
            ch.send(response)?;
        } else {
            debug!(
                "Failed to find response channel for key: {key:?}, response: {response:?}, list: {:?}",
                self.0
            );
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
    Register {
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
    type Incoming: FusedStream<Item = IrcMessage> + Unpin;
    type Outgoing: SendCommand;

    fn in_out(self) -> (Self::Incoming, Self::Outgoing);
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
    #[tracing::instrument(ret)]
    pub async fn new(connection: C, spawn: fn(IrcActor<C>) -> ()) -> UnboundedSender<ActorMessage> {
        let (incoming, outgoing) = connection.in_out();

        let (cmd_tx, cmd_rx) = mpsc::unbounded();
        let actor = Self {
            cmd_rx,
            incoming,
            outgoing,
            state: Server::default(),
            response_channels: ResponseChannels::default(),
            event_handlers: Vec::new(),
            error_handlers: Vec::new(),
            disconnect_handlers: Vec::new(),
        };

        spawn(actor);

        cmd_tx
    }

    #[tracing::instrument(ret, skip(self))]
    pub async fn run(mut self) {
        loop {
            futures::select! {
                msg = self.incoming.next() => {
                    match msg {
                        Some(m) => self.handle_incoming(m).await.unwrap(),
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

    #[tracing::instrument(err, skip(self))]
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
                let channel = Channel {
                    id: self.state.channels.iter().map(|c| c.id).max().unwrap_or(0),
                    metadata: ChannelMetadata {
                        name: channel_name.clone(),
                        display_name: None,
                        description: None,
                        icon: None,
                    },
                    messages: Vec::new(),
                    users: Vec::new(),
                };
                self.state.channels.push(channel.clone());
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
                if let Some(ref tags) = message.tags {
                    for Tag(key, value) in tags {
                        match key.as_str() {
                            "msgid" => msgid = value.clone(),
                            "time" => {
                                server_time = value
                                    .as_ref()
                                    .and_then(|v| OffsetDateTime::parse(&v, &Iso8601::DEFAULT).ok())
                            }
                            _ => {
                                dbg!("unhandled tag", key, value);
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
                        server_time,
                        message_type: MessageType::Privmsg,
                        user_id: 0,
                    },
                };

                if message.source_nickname() == Some(&self.state.me.as_ref().unwrap().nickname) {
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
            _ => {
                dbg!(message);
            }
        }

        Ok(())
    }

    #[tracing::instrument(ret, err, skip(self))]
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
            }
            _ => {
                dbg!(sub, &param, caps);
            }
        }

        Ok(())
    }

    #[tracing::instrument(ret, err, skip(self))]
    pub async fn handle_response(
        &mut self,
        rpl: Response,
        params: Vec<String>,
    ) -> anyhow::Result<()> {
        match rpl {
            Response::RPL_WELCOME => self.state.me = Some(User::new(0, params[0].clone())),
            Response::RPL_MOTDSTART => {
                self.state.metadata.reset_motd();
            }
            Response::RPL_MOTD => {
                self.state.metadata.add_motd(&params[1]);
            }
            Response::RPL_ENDOFMOTD => {
                if !self.state.connected {
                    self.response_channels
                        .reply(
                            &CommandKey::Register,
                            CommandResponse::Register(self.state.clone()),
                        )
                        .await
                        .map_err(|e| anyhow!("Failed to reply to connetion command {e:?}"))?;
                    self.state.connected = true;
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
            | Response::RPL_LOCALUSERS
            | Response::RPL_GLOBALUSERS => (),
            _ => {
                dbg!(rpl, params);
            }
        }

        Ok(())
    }

    #[tracing::instrument(ret, err, skip(self))]
    pub async fn handle_command(&mut self, cmd: ActorMessage) -> anyhow::Result<()> {
        match cmd.command {
            ActorCommand::Register {
                nick,
                user,
                realname,
            } => {
                self.response_channels
                    .register(CommandKey::Register, cmd.reply_tx.unwrap());
                self.register(nick, user, realname).await?;
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

    #[tracing::instrument(ret, err, skip(self))]
    pub async fn on_event(&mut self, event: ServerEvent) -> anyhow::Result<()> {
        for handler in &mut self.event_handlers {
            handler.send(event.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(ret, err, skip(self))]
    pub async fn on_error(&mut self, error: ServerError) -> anyhow::Result<()> {
        for handler in &mut self.error_handlers {
            handler.send(error.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(ret, err, skip(self))]
    pub async fn on_disconnect(&mut self, reason: String) -> anyhow::Result<()> {
        for handler in &mut self.disconnect_handlers {
            handler.send(reason.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(ret, err, skip(self))]
    async fn register(
        &mut self,
        nick: String,
        user: String,
        realname: String,
    ) -> anyhow::Result<()> {
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
        self.end_caps().await?;

        self.nick(nick).await?;
        self.user(user, String::from("0"), realname).await?;

        Ok(())
    }
}

impl<C: IrcConnection> SendCommand for IrcActor<C> {
    type Error = <C::Outgoing as SendCommand>::Error;
    async fn message(&mut self, command: IrcMessage) -> Result<(), Self::Error> {
        self.outgoing.message(command).await
    }
}
