use core::fmt;

use futures::{
    SinkExt,
    channel::mpsc::{self, UnboundedSender},
};
use irc_proto::{CapSubCommand, Command::*, Message as IrcMessage};

pub trait SendCommand {
    type Error: std::error::Error + Send + Sync + 'static;

    fn message(&mut self, command: IrcMessage) -> impl Future<Output = Result<(), Self::Error>>;

    fn command(
        &mut self,
        command: irc_proto::Command,
    ) -> impl Future<Output = Result<(), Self::Error>> {
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
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async { self.command(PONG(server1, server2)).await }
    }

    fn cap_ls(
        &mut self,
        version: String,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async {
            self.command(CAP(None, CapSubCommand::LS, Some(version), None))
                .await
        }
    }

    fn cap_req(
        &mut self,
        caps: &[&str],
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async {
            self.command(CAP(None, CapSubCommand::REQ, None, Some(caps.join(" "))))
                .await
        }
    }

    fn cap_end(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async {
            self.command(CAP(None, CapSubCommand::END, None, None))
                .await
        }
    }

    fn nick(&mut self, nick: String) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async { self.command(NICK(nick)).await }
    }

    fn sasl(&mut self, req: String) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async { self.command(AUTHENTICATE(req)).await }
    }

    fn sasl_plain(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async { self.sasl("PLAIN".into()).await }
    }

    fn sasl_abort(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async { self.sasl("*".into()).await }
    }

    fn user(
        &mut self,
        user: String,
        mode: String,
        realname: String,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async { self.command(USER(user, mode, realname)).await }
    }

    fn join(
        &mut self,
        channel: String,
        password: Option<String>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async { self.command(JOIN(channel, password, None)).await }
    }

    fn privmsg(
        &mut self,
        target: String,
        message: String,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async { self.command(PRIVMSG(target, message)).await }
    }

    fn whois(
        &mut self,
        server: Option<String>,
        user: String,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async { self.command(WHOIS(server, user)).await }
    }

    fn history(
        &mut self,
        subcommand: ChatHistorySubCommand,
        mut args: Vec<String>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async move {
            args.insert(0, subcommand.to_string());
            self.command(Raw("CHATHISTORY".to_string(), args)).await
        }
    }

    fn history_before(
        &mut self,
        target: String,
        before: String,
        limit: i32,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async move {
            self.history(
                ChatHistorySubCommand::Before,
                vec![target, before, limit.to_string()],
            )
            .await
        }
    }

    fn history_latest(
        &mut self,
        target: String,
        since: Option<String>,
        limit: i32,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async move {
            self.history(
                ChatHistorySubCommand::Latest,
                vec![
                    target,
                    since.unwrap_or_else(|| String::from("*")),
                    limit.to_string(),
                ],
            )
            .await
        }
    }
}

impl SendCommand for UnboundedSender<IrcMessage> {
    type Error = mpsc::SendError;
    async fn message(&mut self, message: IrcMessage) -> Result<(), Self::Error> {
        self.send(message).await?;

        Ok(())
    }
}

pub enum ChatHistorySubCommand {
    Before,
    After,
    Latest,
    Around,
    Between,
    Targets,
}

impl fmt::Display for ChatHistorySubCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Before => write!(f, "BEFORE"),
            Self::After => write!(f, "AFTER"),
            Self::Latest => write!(f, "LATEST"),
            Self::Around => write!(f, "AROUND"),
            Self::Between => write!(f, "BETWEEN"),
            Self::Targets => write!(f, "TARGETS"),
        }
    }
}
