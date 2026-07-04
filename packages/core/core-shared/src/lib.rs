use std::sync::Arc;

use futures::{
    SinkExt,
    channel::{
        mpsc::{self, UnboundedSender},
        oneshot,
    },
    lock::Mutex,
};
use irc_proto::{CapSubCommand, Command::*, Message as IrcMessage, Response, message::Tag};

use time::{OffsetDateTime, format_description::well_known::Iso8601};

#[cfg(feature = "web")]
use gloo_console::warn;
#[cfg(feature = "web")]
use tsify::Tsify;
#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "web")]
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

pub mod state;

#[derive(Debug, PartialEq, Eq)]
pub enum CommandKey {
    Connect,
    Join(String),
    Privmsg { channel: String, text: String },
}

#[derive(Debug)]
pub enum CommandResponse {
    Connect,
    Join(String),
    Privmsg(Message),
}

#[derive(Default, Clone)]
pub struct ResponseChannels(Arc<Mutex<Vec<(CommandKey, oneshot::Sender<CommandResponse>)>>>);

impl ResponseChannels {
    pub async fn create(&self, key: CommandKey) -> oneshot::Receiver<CommandResponse> {
        let (os_tx, os_rx) = oneshot::channel();
        self.0.lock().await.push((key, os_tx));

        os_rx
    }

    pub async fn reply(
        &self,
        key: &CommandKey,
        response: CommandResponse,
    ) -> Result<(), CommandResponse> {
        let mut channels = self.0.lock().await;
        if let Some(idx) = channels.iter().position(|(rk, _)| rk == key) {
            let (_, ch) = channels.remove(idx);
            ch.send(response)?;
        } else {
            warn!("Failed to find response channel for key: {key}, response: {response}");
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen)]
pub enum ServerError {
    Placeholder,
}

pub async fn handle_message(
    message: IrcMessage,
    out: &mut UnboundedSender<irc_proto::Message>,
    server_events: &mut UnboundedSender<ServerEvent>,
    errors: &mut UnboundedSender<ServerError>,
    response_channels: &ResponseChannels,
    state: &Arc<Mutex<Server>>,
) {
    match message.command {
        CAP(_, sub, param, caps) => {
            handle_caps(sub, param, caps, &state).await;
        }
        PING(server1, server2) => {
            out.pong(server1, server2).await.unwrap();
        }
        Response(rpl, params) => {
            handle_response(rpl, params, &state, response_channels).await;
        }
        JOIN(channel, _, _) => {
            response_channels
                .reply(
                    &CommandKey::Join(channel.clone()),
                    CommandResponse::Join(channel),
                )
                .await
                .unwrap();
        }
        PRIVMSG(channel, text) => {
            let mut msgid = None;
            let mut server_time = None;
            if let Some(tags) = message.tags {
                for Tag(key, value) in tags {
                    match key.as_str() {
                        "msgid" => msgid = value,
                        "time" => {
                            server_time = value
                                .and_then(|v| OffsetDateTime::parse(&v, &Iso8601::DEFAULT).ok())
                        }
                        _ => {
                            dbg!("unhandled tag", key, value);
                        }
                    }
                }
            }

            let message = Message {
                text: Some(TextMessage {
                    content: text.clone(),
                    reactions: Vec::new(),
                    reply: None,
                    redacted: false,
                    edited: false,
                }),
                metadata: MessageMetadata {
                    msgid: msgid.unwrap(),
                    server_time: server_time.unwrap().unix_timestamp(),
                    message_type: MessageType::Privmsg,
                    user_id: 0,
                },
            };

            response_channels
                .reply(
                    &CommandKey::Privmsg { channel, text },
                    CommandResponse::Privmsg(message.clone()),
                )
                .await
                .unwrap();

            server_events
                .send(ServerEvent::Privmsg(message))
                .await
                .unwrap();
        }
        _ => {
            dbg!(message);
        }
    }
}

pub async fn handle_caps(
    sub: CapSubCommand,
    param: Option<String>,
    caps: Option<String>,
    state: &Arc<Mutex<Server>>,
) {
    match sub {
        CapSubCommand::LS if let Some(caps) = caps => {
            for cap in caps.split_whitespace() {
                let cap = cap.split('=').next().unwrap();
                state.lock().await.capabilities.set_from_name(cap, None);
            }
        }
        CapSubCommand::LS if let Some(param) = param => {
            if param == "*" {
                return;
            }
            for cap in param.split_whitespace() {
                let cap = cap.split('=').next().unwrap();
                state.lock().await.capabilities.set_from_name(cap, None);
            }
        }
        CapSubCommand::ACK if let Some(param) = param => {
            for cap in param.split_whitespace() {
                state
                    .lock()
                    .await
                    .capabilities
                    .set_from_name(cap, Some(true));
            }
        }
        _ => {
            dbg!(sub, &param, caps);
        }
    }
}

pub async fn handle_response(
    rpl: Response,
    params: Vec<String>,
    state: &Arc<Mutex<Server>>,
    response_channels: &ResponseChannels,
) {
    match rpl {
        Response::RPL_WELCOME => state.lock().await.me = Some(User::new(0, params[0].clone())),
        Response::RPL_MOTDSTART => {
            state.lock().await.metadata.reset_motd();
        }
        Response::RPL_MOTD => {
            state.lock().await.metadata.add_motd(&params[1]);
        }
        Response::RPL_ENDOFMOTD => {
            if !state.lock().await.connected {
                response_channels
                    .reply(&CommandKey::Connect, CommandResponse::Connect)
                    .await
                    .unwrap();
                // server_events.send(ServerEvent::Connected).await.unwrap();
                state.lock().await.connected = true;
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
}

pub trait SendCommand {
    type Error;

    fn message(&mut self, command: IrcMessage) -> impl Future<Output = Result<(), Self::Error>>;

    fn command(
        &mut self,
        command: irc_proto::Command,
    ) -> impl Future<Output = Result<(), Self::Error>>
    where
        Self: Send,
    {
        async {
            self.message(IrcMessage {
                tags: None,
                prefix: None,
                command,
            })
            .await?;

            Ok(())
        }
    }

    fn pong(
        &mut self,
        server1: String,
        server2: Option<String>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>>
    where
        Self: Send,
    {
        async { self.command(PONG(server1, server2)).await }
    }

    fn ls_caps(
        &mut self,
        version: String,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>>
    where
        Self: Send,
    {
        async {
            self.command(CAP(None, CapSubCommand::LS, Some(version), None))
                .await
        }
    }

    fn req_caps(
        &mut self,
        caps: &[&str],
    ) -> impl std::future::Future<Output = Result<(), Self::Error>>
    where
        Self: Send,
    {
        async {
            self.command(CAP(None, CapSubCommand::REQ, None, Some(caps.join(" "))))
                .await
        }
    }

    fn end_caps(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>>
    where
        Self: Send,
    {
        async {
            self.command(CAP(None, CapSubCommand::END, None, None))
                .await
        }
    }

    fn nick(&mut self, nick: String) -> impl std::future::Future<Output = Result<(), Self::Error>>
    where
        Self: Send,
    {
        async { self.command(NICK(nick)).await }
    }

    fn user(
        &mut self,
        user: String,
        mode: String,
        realname: String,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>>
    where
        Self: Send,
    {
        async { self.command(USER(user, mode, realname)).await }
    }

    fn join(
        &mut self,
        channel: String,
        password: Option<String>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>>
    where
        Self: Send,
    {
        async { self.command(JOIN(channel, password, None)).await }
    }

    fn privmsg(
        &mut self,
        target: String,
        message: String,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>>
    where
        Self: Send,
    {
        async { self.command(PRIVMSG(target, message)).await }
    }
}

impl SendCommand for UnboundedSender<IrcMessage> {
    type Error = mpsc::SendError;
    async fn message(&mut self, message: IrcMessage) -> Result<(), Self::Error> {
        self.send(message).await?;

        Ok(())
    }
}

#[cfg(feature = "web")]
use gloo_net::websocket::{self, Message as WsMessage, futures::WebSocket};

use crate::state::{Message, MessageMetadata, MessageType, Server, ServerEvent, TextMessage, User};

#[cfg(feature = "web")]
impl SendCommand for futures::stream::SplitSink<WebSocket, WsMessage> {
    type Error = websocket::WebSocketError;
    async fn message(&mut self, message: IrcMessage) -> Result<(), Self::Error> {
        self.send(WsMessage::Text(dbg!(message.to_string())))
            .await?;

        Ok(())
    }
}
