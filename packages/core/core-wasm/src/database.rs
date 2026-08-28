use std::fmt;

use anyhow::{Context, anyhow};
use core_shared::{database::Database as ActorDatabase, state::OrbitError};
use indexed_db_futures::prelude::QuerySource;
use indexed_db_futures::transaction::Transaction;
use indexed_db_futures::{
    Build, KeyPath, KeyPathSeq, database::Database as InnerDb, transaction::TransactionMode,
};
use indexed_db_futures::{BuildSerde, KeyRange};
use serde::{Deserialize, Serialize};

use crate::dbg;

const MESSAGE_STORE: &str = "messages";
const CHANNEL_TIME_INDEX: &str = "server-channel-timestamp";

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
                                    "server_id",
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

    #[tracing::instrument(err, skip(self))]
    async fn message_tx(
        &self,
        tx: &Transaction<'_>,
        msgid: &str,
    ) -> Result<Option<(i32, String, core_shared::state::Message)>, OrbitError> {
        let store = tx
            .object_store(MESSAGE_STORE)
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to open messages object store")?;

        let message: Option<DbMessage> = store
            .get(msgid)
            .serde()
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to serialize msgid")?
            .await
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to get message")?;

        Ok(message.map(|m| (m.server_id, m.channel, m.message)))
    }

    #[tracing::instrument(err, skip(self))]
    async fn insert_message_tx(
        &self,
        tx: &Transaction<'_>,
        server_id: i32,
        channel: &str,
        message: core_shared::state::Message,
    ) -> Result<(), OrbitError> {
        let store = tx
            .object_store(MESSAGE_STORE)
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to open messages object store")?;

        let msgid = message.metadata.msgid.clone();
        let message = DbMessage {
            server_id,
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
            .context("Failed to serialize msgid")?
            .await
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to put message")?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMessage {
    pub message: core_shared::state::Message,
    pub server_id: i32,
    pub channel: String,
    pub timestamp: f64,
}

impl ActorDatabase for IndexedDb {
    #[tracing::instrument(err, skip(self))]
    async fn insert_message(
        &mut self,
        server_id: i32,
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

        self.insert_message_tx(&tx, server_id, channel, message)
            .await?;

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
    ) -> Result<Option<(i32, String, core_shared::state::Message)>, OrbitError> {
        let tx = self
            .inner
            .transaction(MESSAGE_STORE)
            .build()
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to create messages transaction")?;

        self.message_tx(&tx, msgid).await
    }

    #[tracing::instrument(err, skip(self))]
    async fn messages(
        &mut self,
        server_id: i32,
        channel: &str,
    ) -> Result<Vec<core_shared::state::Message>, OrbitError> {
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
        let index = store
            .index(CHANNEL_TIME_INDEX)
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to access index")?;

        // false = inclusive bound
        let range = KeyRange::Bound(
            (server_id, channel, 0.0),
            false,
            (server_id, channel, f64::MAX),
            false,
        );

        let mut messages = Vec::new();
        for data in index
            .get_all::<DbMessage>()
            .with_query(range)
            .serde()
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to serialize message range")?
            .await
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to get messages")?
        {
            let data = data
                .map_err(|e| anyhow!(e.to_string()))
                .context("Failed to deserialize message")?;
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
        let tx = self
            .inner
            .transaction(MESSAGE_STORE)
            .with_mode(TransactionMode::Readwrite)
            .build()
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to create messages transaction")?;

        let Some((server_id, channel, mut message)) = self.message_tx(&tx, msgid).await? else {
            return Ok(());
        };
        let Some(text) = &mut message.text else {
            return Ok(());
        };
        let reactors = text.reactions.entry(react.to_string()).or_default();
        reactors.push(reactor.to_string());
        reactors.sort();
        reactors.dedup();

        self.insert_message_tx(&tx, server_id, &channel, message)
            .await?;

        tx.commit()
            .await
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to commit transaction")?;

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    async fn remove_reaction(
        &mut self,
        msgid: &str,
        react: &str,
        reactor: &str,
    ) -> Result<(), OrbitError> {
        let tx = self
            .inner
            .transaction(MESSAGE_STORE)
            .with_mode(TransactionMode::Readwrite)
            .build()
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to create messages transaction")?;

        let Some((server_id, channel, mut message)) = self.message_tx(&tx, msgid).await? else {
            return Ok(());
        };
        let Some(text) = &mut message.text else {
            return Ok(());
        };

        let mut changed = false;
        if let Some(reactors) = text.reactions.get_mut(react)
            && let Some(pos) = reactors.iter().position(|r| *r == reactor)
        {
            reactors.remove(pos);
            if reactors.is_empty() {
                text.reactions.remove(react);
                changed = true;
            }
        }

        if changed {
            return Ok(());
        }

        self.insert_message_tx(&tx, server_id, &channel, message)
            .await?;

        tx.commit()
            .await
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to commit transaction")?;

        Ok(())
    }
}
