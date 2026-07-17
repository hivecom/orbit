use core_shared::actor::{CommandResponse, IrcConnection};
use core_shared::state::{Capabilities, Capability, User};
use futures::SinkExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use irc_proto::{CapSubCommand, Command, Message as IrcMessage};

use std::str::FromStr;
use std::time::Duration;

use core_shared::actor::{ActorCommand, ActorMessage, IrcActor};
use futures::channel::oneshot;

const LONG_PASSWORD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[derive(Debug, Default)]
struct MockConn;

impl IrcConnection for MockConn {
    type Incoming = UnboundedReceiver<anyhow::Result<IrcMessage>>;
    type Outgoing = UnboundedSender<IrcMessage>;

    fn address(&self) -> &str {
        "mock-conn"
    }

    fn in_out(self) -> (Self::Incoming, Self::Outgoing) {
        let (outgoing_tx, mut outgoing_rx) = unbounded::<IrcMessage>();
        let (mut incoming_tx, incoming_rx) = unbounded::<anyhow::Result<IrcMessage>>();

        tokio::spawn(async move {
            while let Ok(msg) = outgoing_rx.recv().await {
                dbg!(msg.to_string());
                match msg.command {
                    Command::CAP(None, CapSubCommand::LS, ref version, None) => {
                        if *version == Some("302".into()) {
                            incoming_tx.send(Ok(IrcMessage::from_str(":irc.hivecom.net CAP * LS * :account-notify account-tag away-notify batch cap-notify chghost draft/account-registration=before-connect,email-required draft/channel-rename draft/chathistory draft/event-playback draft/extended-isupport draft/languages=1,en draft/message-redaction draft/metadata-2=before-connect,max-subs=10,max-keys=10 draft/multiline=max-bytes=4096,max-lines=100 draft/no-implicit-names draft/persistence draft/pre-away draft/read-marker draft/relaymsg=/ draft/webpush echo-message").unwrap())).await.unwrap();
                            incoming_tx.send(Ok(IrcMessage::from_str(":irc.hivecom.net CAP * LS :ergo.chat/nope extended-join extended-monitor invite-notify labeled-response message-tags multi-prefix sasl=PLAIN,EXTERNAL server-time setname soju.im/webpush standard-replies userhost-in-names znc.in/playback znc.in/self-message").unwrap())).await.unwrap();
                        } else {
                            incoming_tx.send(Ok(msg)).await.unwrap();
                        }
                    }
                    Command::CAP(None, CapSubCommand::REQ, None, Some(_)) => {
                        incoming_tx.send(Ok(IrcMessage::from_str("@time=2026-07-13T11:01:09.182Z :irc.hivecom.net CAP * ACK :echo-message message-tags sasl draft/message-redaction draft/metadata-2 draft/chathistory draft/event-playback draft/account-registration draft/multiline server-time").unwrap())).await.unwrap();
                    }
                    Command::USER(_, _, _) => {
                        incoming_tx.send(Ok(IrcMessage::from_str(":irc.hivecom.net 001 testnick :Welcome to the Hivecom IRC Network testnick").unwrap())).await.unwrap();
                    }
                    _ => {
                        dbg!(&msg);
                    }
                }
            }
        });

        (incoming_rx, outgoing_tx)
    }
}

#[tokio::test]
async fn test_irc_register_flow() {
    let addr = IrcActor::<MockConn>::start(0, MockConn, |actor: IrcActor<MockConn>| {
        tokio::spawn(actor.run());
    })
    .await
    .unwrap();

    let (tx, rx) = oneshot::channel();

    let requested_nickname = String::from("testnick");
    let requested_username = String::from("testuser");
    let requested_realname = String::from("testreal");

    addr.unbounded_send(ActorMessage {
        command: ActorCommand::SignIn {
            nick: requested_nickname.clone(),
            user: requested_username.clone(),
            realname: requested_realname.clone(),
            password: LONG_PASSWORD.to_string(),
        },
        reply_tx: Some(tx),
    })
    .unwrap();

    // 5. Use tokio::time::timeout to prevent hanging tests
    let response = tokio::time::timeout(Duration::from_secs(1), rx)
        .await
        .unwrap()
        .unwrap();

    let CommandResponse::SignIn(_) = response else {
        panic!("Expected CommandResponse::SignIn but got: {:?}", response);
    };

    let (tx, rx) = oneshot::channel();
    addr.unbounded_send(ActorMessage {
        command: ActorCommand::GetState,
        reply_tx: Some(tx),
    })
    .unwrap();
    let response = tokio::time::timeout(Duration::from_secs(1), rx)
        .await
        .unwrap()
        .unwrap();
    let CommandResponse::GetState(state) = response else {
        panic!("Expected CommandResponse::GetState but got: {:?}", response);
    };

    assert_matches::assert_matches!(
        state.capabilities,
        Capabilities {
            message_tags: Capability {
                has: true,
                enabled: true
            },
            message_redaction: Capability {
                has: true,
                enabled: true
            },
            message_edit: Capability {
                has: false,
                enabled: false
            },
            multiline: Capability {
                has: true,
                enabled: true
            },
            metadata: Capability {
                has: true,
                enabled: true
            },
            webpush: Capability {
                has: true,
                enabled: false
            },
            ..
        }
    );

    assert_matches::assert_matches!(
        state.me,
        Some(User {
            nickname: nick,
            username: user,
            realname,
            ..
        }) if nick == requested_nickname && user == Some(requested_username) && realname == Some(requested_realname)
    );

    // TODO: cleanup
}
