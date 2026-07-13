use core_shared::actor::{CommandResponse, IrcConnection};
use futures::SinkExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use irc_proto::Message as IrcMessage;

use std::time::Duration;

use core_shared::actor::{ActorCommand, ActorMessage, IrcActor};
use futures::channel::oneshot;

#[derive(Debug, Default)]
struct MockConn;

impl IrcConnection for MockConn {
    type Incoming = UnboundedReceiver<IrcMessage>;
    type Outgoing = UnboundedSender<IrcMessage>;

    fn in_out(self) -> (Self::Incoming, Self::Outgoing) {
        let (outgoing_tx, mut outgoing_rx) = unbounded::<IrcMessage>();
        let (mut incoming_tx, incoming_rx) = unbounded::<IrcMessage>();

        tokio::spawn(async move {
            while let Ok(msg) = outgoing_rx.recv().await {
                dbg!(&msg);
                incoming_tx.send(msg).await.unwrap();
            }
        });

        (incoming_rx, outgoing_tx)
    }
}

#[tokio::test]
async fn test_irc_join_flow() {
    let addr = IrcActor::<MockConn>::new(MockConn::default(), |actor: IrcActor<MockConn>| {
        tokio::spawn(actor.run());
    })
    .await;

    let (tx, rx) = oneshot::channel();

    addr.unbounded_send(ActorMessage {
        command: ActorCommand::Join {
            channel: "channel".into(),
            password: None,
        },
        reply_tx: tx,
    })
    .unwrap();

    // 5. Use tokio::time::timeout to prevent hanging tests
    let response = tokio::time::timeout(Duration::from_secs(1), rx).await;
    assert!(response.is_ok());

    assert_eq!(
        response.unwrap().unwrap(),
        CommandResponse::Join(String::from("channel"))
    );

    // TODO: cleanup
}
