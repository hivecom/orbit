use std::sync::Arc;

use futures::{
    SinkExt,
    channel::mpsc::{self, UnboundedSender},
    lock::Mutex,
};
use irc_proto::{CapSubCommand, Command::*, Message as IrcMessage, Response};

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

pub async fn handle_message(
    message: IrcMessage,
    out: &mut UnboundedSender<irc_proto::Message>,
    server_events: &mut UnboundedSender<ServerEvent>,
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
            handle_response(rpl, params, &state, server_events).await;
        }
        _ => {
            // dbg!(message);
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
    server_events: &mut UnboundedSender<ServerEvent>,
) {
    match rpl {
        Response::RPL_MOTDSTART => state.lock().await.metadata.reset_motd(),
        Response::RPL_MOTD => state.lock().await.metadata.add_motd(&params[1]),
        Response::RPL_WELCOME
        | Response::RPL_YOURHOST
        | Response::RPL_CREATED
        | Response::RPL_MYINFO
        | Response::RPL_LUSERCLIENT
        | Response::RPL_LUSEROP
        | Response::RPL_LUSERUNKNOWN
        | Response::RPL_LUSERCHANNELS
        | Response::RPL_LUSERME
        | Response::RPL_LOCALUSERS
        | Response::RPL_GLOBALUSERS
        | Response::RPL_ENDOFMOTD => {
            if !state.lock().await.connected {
                server_events.send(ServerEvent::Connected).await.unwrap();
                state.lock().await.connected = true;
            }
        }
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

use crate::state::{Server, ServerEvent};

#[cfg(feature = "web")]
impl SendCommand for futures::stream::SplitSink<WebSocket, WsMessage> {
    type Error = websocket::WebSocketError;
    async fn message(&mut self, message: IrcMessage) -> Result<(), Self::Error> {
        self.send(WsMessage::Text(dbg!(message.to_string())))
            .await?;

        Ok(())
    }
}
