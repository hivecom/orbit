use std::{fmt, time::Duration};

use futures::{FutureExt, future::FusedFuture};
use rand::{SeedableRng, rngs::SmallRng};
#[cfg(not(feature = "web"))]
use std::time::Instant;
#[cfg(feature = "web")]
use web_time::Instant;

#[cfg(feature = "web")]
use crate::dbg;
use crate::{
    SendCommand,
    database::Database,
    response_channels::{CommandKey, CommandResponse, ResponseChannels},
    state::{Channel, Message, OrbitError, Server, ServerEvent, User},
};
use anyhow::Context;
use futures::{
    SinkExt, StreamExt,
    channel::{
        mpsc::{self, UnboundedSender},
        oneshot,
    },
    stream::FusedStream,
};
use irc_proto::Message as IrcMessage;
use tracing::warn;

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

pub(crate) struct RequestedHistory {
    pub target: String,
    pub label: Option<String>,
}

#[derive(Debug)]
pub(crate) struct CurrentBatch {
    pub id: String,
    pub data: BatchData,
}

#[derive(Debug)]
pub(crate) enum BatchData {
    History {
        label: Option<String>,
        channel: String,
        messages: Vec<Message>,
    },
    Multiline {
        target: String,
        message: Message,
    },
    Unhandled,
}

impl CurrentBatch {
    pub fn is_chathistory(&self) -> bool {
        matches!(self.data, BatchData::History { .. })
    }
}

#[derive(Default, Clone)]
pub(crate) enum SaslState {
    #[default]
    Unauthed,
    Requested {
        nickname: String,
        realname: String,
        username: String,
        password: String,
    },
}

pub struct IrcActor<C: IrcConnection, DB: Database> {
    pub(crate) cmd_rx: mpsc::UnboundedReceiver<ActorMessage>,
    pub(crate) incoming: C::Incoming,
    pub(crate) outgoing: C::Outgoing,
    pub(crate) state: Server,
    pub(crate) database: DB,
    pub(crate) response_channels: ResponseChannels,
    pub(crate) event_handlers: Vec<UnboundedSender<ServerEvent>>,
    pub(crate) error_handlers: Vec<UnboundedSender<OrbitError>>,
    pub(crate) disconnect_handlers: Vec<UnboundedSender<String>>,

    pub(crate) current_batches: Vec<CurrentBatch>,
    pub(crate) requested_history_batches: Vec<(RequestedHistory, Instant)>,
    pub(crate) sasl_state: SaslState,
    pub(crate) rng: SmallRng,
}

impl<C: IrcConnection, DB: Database> IrcActor<C, DB> {
    #[tracing::instrument]
    pub async fn start(
        id: i32,
        connection: C,
        database: DB,
        spawn: fn(IrcActor<C, DB>) -> (),
    ) -> Result<UnboundedSender<ActorMessage>, OrbitError> {
        let address = connection.address().to_string();
        let (incoming, outgoing) = connection.in_out();

        let (cmd_tx, cmd_rx) = mpsc::unbounded();
        let mut actor = Self {
            cmd_rx,
            incoming,
            outgoing,
            state: Server::new(id, address),
            database,
            response_channels: ResponseChannels::default(),
            event_handlers: Default::default(),
            error_handlers: Default::default(),
            disconnect_handlers: Default::default(),
            current_batches: Default::default(),
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
            let timeout = gloo_timers::future::TimeoutFuture::new(1000).fuse();
            #[cfg(not(feature = "web"))]
            let timeout = Box::pin(tokio::time::sleep(Duration::from_secs(1)).fuse());

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


                    assert!(
                        self.requested_history_batches
                            .iter()
                            .all(|(_, creation)| creation.elapsed() < Duration::from_secs(5))
                    );

                    timeout = create_timeout();
                }
            }
        }
    }

    pub(crate) async fn push_batch(&mut self, target: String, state_message: Message) -> bool {
        if let Some(batch) = self.current_batches.iter_mut().find(|b| b.is_chathistory())
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
    pub(crate) async fn on_event(&mut self, event: ServerEvent) -> Result<(), OrbitError> {
        for handler in &mut self.event_handlers {
            handler.send(event.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn on_error(&mut self, error: OrbitError) -> Result<(), OrbitError> {
        for handler in &mut self.error_handlers {
            handler.send(error.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn on_disconnect(&mut self, reason: String) -> Result<(), OrbitError> {
        for handler in &mut self.disconnect_handlers {
            handler.send(reason.clone()).await.unwrap();
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn request_caps(&mut self) -> Result<(), OrbitError> {
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
    pub(crate) async fn sign_in_anonymous(
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
    pub(crate) async fn sign_in(
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

    pub(crate) async fn channel_mut(&mut self, name: String) -> &mut Channel {
        self.state
            .channels
            .entry(name.clone())
            .or_insert_with(|| Channel::new(name))
    }

    pub(crate) async fn user_mut(&mut self, nickname: String) -> &mut User {
        self.state
            .users
            .entry(nickname.clone())
            .or_insert_with(|| User::new(nickname))
    }
}

impl<C: IrcConnection, DB: Database> SendCommand for IrcActor<C, DB> {
    type Error = <C::Outgoing as SendCommand>::Error;
    async fn message(&mut self, command: IrcMessage) -> Result<(), Self::Error> {
        self.outgoing.message(command).await
    }
}
