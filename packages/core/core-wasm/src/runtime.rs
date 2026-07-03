use std::{str::FromStr, sync::Arc};

use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    lock::Mutex,
};
use gloo_console::{debug, error, log};
use gloo_net::websocket::{Message as WsMessage, futures::WebSocket};
use indexed_db_futures::{
    Build, BuildSerde, database::Database, prelude::QuerySource, transaction::TransactionMode,
};
use irc_proto::Message as IrcMessage;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, spawn_local};

use core_shared::{
    SendCommand, handle_message,
    state::{Capabilities, Server, ServerEvent, ServerMetadata},
};

async fn open_db() -> Database {
    Database::open("orbit-connector")
        .with_on_upgrade_needed_fut(|event, db| async move {
            let old_version = event.old_version() as u64;
            let new_version = event.new_version().map(|v| v as u64);

            match (old_version, new_version) {
                (0, Some(1)) => {
                    db.create_object_store("servers").build()?;
                }
                _ => {}
            }

            Ok(())
        })
        .await
        .unwrap()
}

#[derive(Default)]
#[wasm_bindgen]
pub struct ServerList {
    servers: Vec<IrcServer>,
}

#[wasm_bindgen]
impl ServerList {
    #[wasm_bindgen]
    pub async fn new() -> Result<Self, JsError> {
        let db = open_db().await;
        let servers = {
            let transaction = db
                .transaction("servers")
                .with_mode(TransactionMode::Readwrite)
                .build()
                .unwrap();

            let store = transaction.object_store("servers").unwrap();

            let mut servers = Vec::new();

            for data in store.get_all::<ServerData>().serde()?.await? {
                debug!(format!("Loaded server: {:?}", &data));
                let data = data.unwrap();
                let server = IrcServer::new(IrcRuntime::new(data).await);
                server.connect().await?;
                servers.push(server);
            }

            servers
        };

        Ok(Self { servers })
    }

    #[wasm_bindgen]
    pub async fn create_server(
        &mut self,
        server_name: String,
        address: String,
        username: String,
    ) -> Result<IrcServer, JsError> {
        debug!("Adding server");
        let id = self.servers.len();

        let server = IrcServer::new(
            IrcRuntime::new(ServerData {
                id: id as i64,
                name: server_name,
                address,
                username: username.clone(),
                nickname: username.clone(),
                realname: username,
                password: None,
            })
            .await,
        );
        server.connect().await?;
        debug!("Connected");
        self.servers.push(server.clone());

        Ok(server)
    }

    #[wasm_bindgen]
    pub async fn get_servers(&self) -> Vec<IrcServer> {
        self.servers.clone()
    }

    #[wasm_bindgen]
    pub async fn get_server_by_id(&self, id: i64) -> IrcServer {
        debug_assert_eq!(self.servers[id as usize].0.lock().await.my_id, id);
        self.servers[id as usize].clone()
    }
}

// import { getServers, getServerById } from "connector"
//
// // Fetch all servers
// const servers = getServers()
//
// for (const server of servers) {
//   // Listen for messages on all servers
//   server.on("message", (payload) => {
//     payload.channel
//     payload.message
//   })
//
//   // Show error in eu
//   server.on("error", () => {})
//   // Some clean up
//   server.on("disconnect", () => {})
// }
//
// // Other place - probably just a method to return part of existing state on rust side. Low cost method (I hope)
// const specificServer = getServerById(12)
// specificServer.disconnect()
//
// // Some other place
//
// const channel = specificServer.getChannelById(4)
// channel.sendMessage("ok what do you think about this @Jokler?")

#[derive(Clone)]
#[wasm_bindgen]
pub struct IrcServer(Arc<Mutex<IrcRuntime>>);

#[wasm_bindgen]
impl IrcServer {
    fn new(inner: IrcRuntime) -> Self {
        Self(Arc::new(Mutex::new(inner)))
    }

    #[wasm_bindgen]
    pub async fn connect(&self) -> Result<(), JsError> {
        self.0.lock().await.connect().await
    }

    #[wasm_bindgen]
    pub async fn disconnect(&self) {
        self.0.lock().await.disconnect().await;
    }

    #[wasm_bindgen]
    pub async fn join_channel(&self, channel: String, password: Option<String>) {
        self.0.lock().await.join(channel, password).await.unwrap();
    }

    #[wasm_bindgen]
    pub async fn on_message(&self, f: js_sys::Function) {
        self.0.lock().await.on_message(f).await;
    }

    #[wasm_bindgen]
    pub async fn on_event(&self, f: js_sys::Function) {
        self.0.lock().await.on_event(f).await;
    }

    #[wasm_bindgen]
    pub async fn on_error(&self, f: js_sys::Function) {
        self.0.lock().await.on_error(f).await;
    }

    #[wasm_bindgen]
    pub async fn on_disconnect(&self, f: js_sys::Function) {
        self.0.lock().await.on_disconnect(f).await;
    }

    #[wasm_bindgen]
    pub async fn state(&self) -> Server {
        self.0.lock().await.state.lock().await.clone()
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct IrcChannel {
    name: String,
    outgoing: UnboundedSender<IrcMessage>,
}

#[wasm_bindgen]
impl IrcChannel {
    #[wasm_bindgen]
    pub async fn send_message(&mut self, message: String) {
        self.outgoing
            .privmsg(self.name.clone(), message)
            .await
            .unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct ServerData {
    pub id: i64,
    pub name: String,
    pub address: String, // separate port?
    pub username: String,
    pub nickname: String,
    pub realname: String,
    pub password: Option<String>,
}

pub struct IrcRuntime {
    db: Database,
    my_id: i64,
    state: Arc<Mutex<Server>>,
    outgoing: Option<UnboundedSender<IrcMessage>>,
    server_events: Option<UnboundedReceiver<ServerEvent>>,
}

impl IrcRuntime {
    pub async fn new(server_data: ServerData) -> Self {
        let db = open_db().await;

        let id = server_data.id;
        let name = server_data.name.clone();
        let address = server_data.address.clone();
        {
            let transaction = db
                .transaction("servers")
                .with_mode(TransactionMode::Readwrite)
                .build()
                .unwrap();
            let store = transaction.object_store("servers").unwrap();
            store
                .put(server_data)
                .with_key(id)
                .with_key_type::<i64>()
                .serde()
                .unwrap();

            transaction.commit().await.unwrap();
        }

        Self {
            db,
            my_id: id,
            state: Arc::new(Mutex::new(Server {
                id,
                metadata: ServerMetadata {
                    name,
                    motd: None,
                    address,
                    transponder_url: None,
                    satellite_url: None,
                    depot_url: None,
                },
                channels: Vec::new(),
                capabilities: Capabilities::default(),
                users: Vec::new(),
                me: None,
                connected: false,
            })),
            outgoing: None,
            server_events: None,
        }
    }

    async fn register(&mut self) {
        let irc_version = String::from("302");
        self.ls_caps(irc_version).await.unwrap();
        self.req_caps(&[
            "echo-message",
            "message-tags",
            "sasl",
            "draft/message-redaction",
            "draft/metadata-2",
            "draft/chathistory",
            "draft/event-playback",
            "draft/account-registration",
            "draft/multiline",
            "server-time",
        ])
        .await
        .unwrap();
        self.end_caps().await.unwrap();

        let name = String::from("orbit-testj");
        self.nick(name.clone()).await.unwrap();
        self.user(name, String::from("0"), String::from("Jokler"))
            .await
            .unwrap();
    }

    /// Connect to server by id
    pub async fn connect(&mut self) -> Result<(), JsError> {
        dbg!("connect");
        log!("connect");
        let server_data = {
            let transaction = self.db.transaction("servers").build().unwrap();
            let store = transaction.object_store("servers").unwrap();
            dbg!(
                store
                    .get::<ServerData, i64, i64>(self.my_id)
                    .serde()
                    .unwrap()
                    .await
            )
        }?
        .ok_or(JsError::new("can't connect to server, invalid id"))?;

        let ws = WebSocket::open(&server_data.address)?;
        let (mut write, mut read) = ws.split();

        let (mut outgoing_tx, mut outgoing_rx) = mpsc::unbounded();
        self.outgoing = Some(outgoing_tx.clone());
        let (mut server_event_tx, server_event_rx) = mpsc::unbounded();
        self.server_events = Some(server_event_rx);

        spawn_local(async move {
            while let Ok(msg) = outgoing_rx.recv().await {
                write.message(msg).await.unwrap();
            }
        });

        self.register().await;

        let state = self.state.clone();
        spawn_local(async move {
            while let Some(msg) = read.next().await {
                let WsMessage::Text(text) = msg.unwrap() else {
                    error!("unexpected binary message on websocket");
                    return;
                };
                let message = IrcMessage::from_str(&text).unwrap();
                handle_message(message, &mut outgoing_tx, &mut server_event_tx, &state).await;
            }
            log!("WebSocket Closed");
        });

        Ok(())
    }

    pub async fn disconnect(&mut self) {
        unimplemented!()
    }

    pub async fn on_message(&self, f: js_sys::Function) {
        unimplemented!()
    }

    pub async fn on_event(&mut self, f: js_sys::Function) {
        let mut server_events = self.server_events.take().unwrap();

        spawn_local(async move {
            while let Some(event) = server_events.next().await {
                f.call1(&JsValue::null(), &event.into()).unwrap();
            }
        });
    }

    pub async fn on_error(&self, f: js_sys::Function) {
        unimplemented!()
    }

    pub async fn on_disconnect(&self, f: js_sys::Function) {
        unimplemented!()
    }
}

impl SendCommand for IrcRuntime {
    type Error = mpsc::SendError;
    async fn message(&mut self, message: IrcMessage) -> Result<(), Self::Error> {
        if let Some(outgoing) = &mut self.outgoing {
            outgoing.send(message).await?;
        }

        Ok(())
    }
}
