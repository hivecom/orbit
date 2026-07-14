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

    fn ls_caps(
        &mut self,
        version: String,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async {
            self.command(CAP(None, CapSubCommand::LS, Some(version), None))
                .await
        }
    }

    fn req_caps(
        &mut self,
        caps: &[&str],
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async {
            self.command(CAP(None, CapSubCommand::REQ, None, Some(caps.join(" "))))
                .await
        }
    }

    fn end_caps(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> {
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
}

impl SendCommand for UnboundedSender<IrcMessage> {
    type Error = mpsc::SendError;
    async fn message(&mut self, message: IrcMessage) -> Result<(), Self::Error> {
        self.send(message).await?;

        Ok(())
    }
}
