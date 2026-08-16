use std::fmt;

use anyhow::{Context, anyhow};
use core_shared::{database::Database as ActorDatabase, state::OrbitError};
use indexed_db_futures::prelude::QuerySource;
use indexed_db_futures::{
    Build, KeyPath, KeyPathSeq, database::Database as InnerDb, transaction::TransactionMode,
};
use indexed_db_futures::{BuildSerde, KeyRange};
use serde::{Deserialize, Serialize};

use crate::dbg;

const MESSAGE_STORE: &str = "messages";
const CHANNEL_TIME_INDEX: &str = "channel-timestamp";

pub struct IndexedDb {
    inner: InnerDb,
}

impl fmt::Debug for IndexedDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IndexedDb").finish_non_exhaustive()
    }
}

impl IndexedDb {
    pub async fn new(name: &str) -> Result<Self, OrbitError> {
        let inner = InnerDb::open(name)
            .with_on_upgrade_needed_fut(|event, db| async move {
                let old_version = event.old_version() as u64;
                let new_version = event.new_version().map(|v| v as u64);

                #[allow(clippy::single_match)]
                match (old_version, new_version) {
                    (0, Some(1)) => {
                        let store = db.create_object_store(MESSAGE_STORE).build()?;
                        store
                            .create_index(
                                CHANNEL_TIME_INDEX,
                                KeyPath::Sequence(KeyPathSeq::from_slice(&[
                                    "channel",
                                    "timestamp",
                                ])),
                            )
                            .build()?;
                    }
                    _ => {}
                }

                Ok(())
            })
            .await
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to open IndexedDb")?;

        Ok(IndexedDb { inner })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMessage {
    pub message: core_shared::state::Message,
    pub channel: String,
    pub timestamp: f64,
}

impl ActorDatabase for IndexedDb {
    #[tracing::instrument(err, skip(self))]
    async fn insert_message(
        &mut self,
        channel: &str,
        message: core_shared::state::Message,
    ) -> Result<(), OrbitError> {
        let tx = self
            .inner
            .transaction(MESSAGE_STORE)
            .with_mode(TransactionMode::Readwrite)
            .build()
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to create messages transaction")?;
        let store = tx
            .object_store(MESSAGE_STORE)
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to open messages object store")?;

        let msgid = message.metadata.msgid.clone();
        let message = DbMessage {
            timestamp: message.metadata.server_time,
            channel: channel.to_string(),
            message,
        };

        store
            .put(message)
            .with_key(msgid)
            .with_key_type::<String>()
            .serde()
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to put message")?;

        tx.commit()
            .await
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to commit transaction")?;

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    async fn message(
        &mut self,
        msgid: &str,
    ) -> Result<Option<(String, core_shared::state::Message)>, OrbitError> {
        let transaction = self.inner.transaction(MESSAGE_STORE).build().unwrap();
        let store = transaction.object_store(MESSAGE_STORE).unwrap();

        let message: Option<DbMessage> = store.get(msgid).serde().unwrap().await.unwrap();

        Ok(message.map(|m| (m.channel, m.message)))
    }

    #[tracing::instrument(err, skip(self))]
    async fn messages(
        &mut self,
        channel: &str,
    ) -> Result<Vec<core_shared::state::Message>, OrbitError> {
        let transaction = self.inner.transaction(MESSAGE_STORE).build().unwrap();
        let store = transaction.object_store(MESSAGE_STORE).unwrap();
        let index = store.index(CHANNEL_TIME_INDEX).unwrap();

        // false = inclusive bound
        let range = KeyRange::Bound((channel, 0.0), false, (channel, f64::MAX), false);

        let mut messages = Vec::new();
        for data in index
            .get_all::<DbMessage>()
            .with_query(range)
            .serde()
            .unwrap()
            .await
            .unwrap()
        {
            let data = data.unwrap();
            messages.push(data.message);
        }

        Ok(messages)
    }

    #[tracing::instrument(err, skip(self))]
    async fn add_reaction(
        &mut self,
        msgid: &str,
        react: &str,
        reactor: &str,
    ) -> Result<(), OrbitError> {
        let Some((channel, mut message)) = self.message(msgid).await? else {
            return Ok(());
        };
        let Some(text) = &mut message.text else {
            return Ok(());
        };
        let reactors = text.reactions.entry(react.to_string()).or_default();
        reactors.push(reactor.to_string());
        reactors.sort();
        reactors.dedup();

        self.insert_message(&channel, message).await?;

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    async fn remove_reaction(
        &mut self,
        msgid: &str,
        react: &str,
        reactor: &str,
    ) -> Result<(), OrbitError> {
        let Some((channel, mut message)) = self.message(msgid).await? else {
            return Ok(());
        };
        let Some(text) = &mut message.text else {
            return Ok(());
        };
        if let Some(reactors) = text.reactions.get_mut(react)
            && let Some(pos) = reactors.iter().position(|r| *r == reactor)
        {
            reactors.remove(pos);
            if reactors.is_empty() {
                text.reactions.remove(react);
            }
        }

        self.insert_message(&channel, message).await?;

        Ok(())
    }
}
