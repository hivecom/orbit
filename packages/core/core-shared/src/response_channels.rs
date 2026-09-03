use std::time::Duration;

use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};
#[cfg(not(feature = "web"))]
use std::time::Instant;
#[cfg(feature = "web")]
use web_time::Instant;

use futures::channel::oneshot;
use tracing::warn;

use crate::state::{Channel, History, Message, OrbitError, Server, SignedIn};

#[cfg(feature = "web")]
#[allow(unused_imports)]
use crate::dbg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CommandKey {
    SignIn,
    Join(String),
    Privmsg { target: String, text: String },
    History(String),
    Label(String),
}

#[derive(Debug)]
pub enum CommandResponse {
    GetState(Box<Server>),
    GetChannelState(Box<Option<Channel>>),
    Capabilities,
    SignIn(SignedIn),
    Join(Box<Channel>),
    Privmsg(Box<Message>),
    History(History),
    Error(OrbitError),
}

const LABEL_CHARSET: &str = "abcdefghijklmnopqrstuvwxyz\
                               ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                               1234567890";

pub(crate) fn generate_label(rng: &mut SmallRng) -> String {
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
pub(crate) struct ResponseChannels {
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
        if let Some(idx) = self.channels.iter().position(|(rk, _, _)| rk == key) {
            let (_, _, ch) = self.channels.remove(idx);
            ch.send(response)?;

            Ok(true)
        } else {
            if let CommandKey::Label(label) = key {
                warn!("Failed to find response channel for label {label:?}");
            }
            // warn!("Failed to find response channel");
            Ok(false)
        }
    }

    pub fn check_timeouts(&mut self) {
        self.channels
            .retain(|(_, creation, _)| creation.elapsed() < Duration::from_secs(5));
    }
}
