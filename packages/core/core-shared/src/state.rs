use std::collections::HashMap;
use std::str::FromStr;

#[cfg(feature = "web")]
use crate::dbg;
#[cfg(feature = "web")]
use tsify::Tsify;
#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Server {
    pub id: u32,
    pub metadata: ServerMetadata,
    pub channels: HashMap<String, Channel>,
    pub capabilities: Capabilities,
    pub support: Support,
    pub users: HashMap<String, User>,
    pub me: Option<User>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct ServerMetadata {
    pub name: String,
    pub motd: Option<String>,
    pub address: String,

    pub transponder_url: Option<String>,
    pub satellite_url: Option<String>,
    pub depot_url: Option<String>,
}

impl ServerMetadata {
    pub fn reset_motd(&mut self) {
        self.motd = None;
    }
    pub fn add_motd(&mut self, line: &str) {
        match self.motd {
            Some(ref mut motd) => {
                motd.push_str("\n");
                motd.push_str(&line);
            }
            None => self.motd = Some(line.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct Channel {
    pub metadata: ChannelMetadata,
    pub messages: Vec<Message>,
    pub users: Vec<String>,
}

impl Channel {
    pub fn new(name: String) -> Self {
        Self {
            metadata: ChannelMetadata {
                name,
                display_name: Default::default(),
                topic: Default::default(),
                description: Default::default(),
                icon: Default::default(),
            },
            messages: Default::default(),
            users: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct ChannelMetadata {
    pub name: String,
    pub display_name: Option<String>,
    pub topic: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen)]
pub struct Capabilities {
    // reacts / replies
    pub message_tags: Capability,
    pub message_redaction: Capability,
    pub message_edit: Capability,
    pub multiline: Capability,
    pub metadata: Capability,
    pub webpush: Capability,

    pub(crate) echo_messages: Capability,
    pub(crate) sasl: Capability,
    pub(crate) history: Capability,
    pub(crate) event_playback: Capability,
    pub(crate) account_registration: Capability,
    pub(crate) server_time: Capability,

    pub(crate) account_notify: Capability,
    pub(crate) account_tag: Capability,
    pub(crate) away_notify: Capability,
    pub(crate) batch: Capability,
    pub(crate) cap_notify: Capability,
    pub(crate) chghost: Capability,
    pub(crate) channel_rename: Capability,
    pub(crate) extended_isupport: Capability,
    pub(crate) languages: Capability,
    pub(crate) no_implicit_names: Capability,
    pub(crate) persistence: Capability,
    pub(crate) pre_away: Capability,
    pub(crate) read_marker: Capability,
    pub(crate) relaymsg: Capability,
    pub(crate) ergo_nope: Capability,
    pub(crate) extended_join: Capability,
    pub(crate) extended_monitor: Capability,
    pub(crate) invite_notify: Capability,
    pub(crate) labeled_response: Capability,
    pub(crate) multi_prefix: Capability,
    pub(crate) setname: Capability,
    pub(crate) standard_replies: Capability,
    pub(crate) userhost_in_names: Capability,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen)]
pub struct Capability {
    pub has: bool,
    pub enabled: bool,
}

impl Capabilities {
    pub fn set_from_name(&mut self, cap: &str, enabled: Option<bool>) {
        match cap {
            "message-tags" => {
                self.message_tags.has = true;
                enabled.map(|e| self.message_tags.enabled = e);
            }
            "draft/message-redaction" => {
                self.message_redaction.has = true;
                enabled.map(|e| self.message_redaction.enabled = e);
            }
            "draft/multiline" => {
                self.multiline.has = true;
                enabled.map(|e| self.multiline.enabled = e);
            }
            "draft/metadata-2" => {
                self.metadata.has = true;
                enabled.map(|e| self.metadata.enabled = e);
            }
            "draft/webpush" => {
                self.webpush.has = true;
                enabled.map(|e| self.webpush.enabled = e);
            }

            "echo-message" => {
                self.echo_messages.has = true;
                enabled.map(|e| self.echo_messages.enabled = e);
            }
            "sasl" => {
                self.sasl.has = true;
                enabled.map(|e| self.sasl.enabled = e);
            }
            "draft/chathistory" => {
                self.history.has = true;
                enabled.map(|e| self.history.enabled = e);
            }
            "draft/event-playback" => {
                self.event_playback.has = true;
                enabled.map(|e| self.event_playback.enabled = e);
            }
            "draft/account-registration" => {
                self.account_registration.has = true;
                enabled.map(|e| self.account_registration.enabled = e);
            }
            "server-time" => {
                self.server_time.has = true;
                enabled.map(|e| self.server_time.enabled = e);
            }

            "account-notify" => {
                self.account_notify.has = true;
                enabled.map(|e| self.account_notify.enabled = e);
            }
            "account-tag" => {
                self.account_tag.has = true;
                enabled.map(|e| self.account_tag.enabled = e);
            }
            "away-notify" => {
                self.away_notify.has = true;
                enabled.map(|e| self.away_notify.enabled = e);
            }
            "batch" => {
                self.batch.has = true;
                enabled.map(|e| self.batch.enabled = e);
            }
            "cap-notify" => {
                self.cap_notify.has = true;
                enabled.map(|e| self.cap_notify.enabled = e);
            }
            "chghost" => {
                self.chghost.has = true;
                enabled.map(|e| self.chghost.enabled = e);
            }
            "draft/channel-rename" => {
                self.channel_rename.has = true;
                enabled.map(|e| self.channel_rename.enabled = e);
            }
            "draft/extended-isupport" => {
                self.extended_isupport.has = true;
                enabled.map(|e| self.extended_isupport.enabled = e);
            }
            "draft/languages" => {
                self.languages.has = true;
                enabled.map(|e| self.languages.enabled = e);
            }
            "draft/no-implicit-names" => {
                self.no_implicit_names.has = true;
                enabled.map(|e| self.no_implicit_names.enabled = e);
            }
            "draft/persistence" => {
                self.persistence.has = true;
                enabled.map(|e| self.persistence.enabled = e);
            }
            "draft/pre-away" => {
                self.pre_away.has = true;
                enabled.map(|e| self.pre_away.enabled = e);
            }
            "draft/read-marker" => {
                self.read_marker.has = true;
                enabled.map(|e| self.read_marker.enabled = e);
            }
            "draft/relaymsg" => {
                self.relaymsg.has = true;
                enabled.map(|e| self.relaymsg.enabled = e);
            }
            "ergo.chat/nope" => {
                self.ergo_nope.has = true;
                enabled.map(|e| self.ergo_nope.enabled = e);
            }
            "extended-join" => {
                self.extended_join.has = true;
                enabled.map(|e| self.extended_join.enabled = e);
            }
            "extended-monitor" => {
                self.extended_monitor.has = true;
                enabled.map(|e| self.extended_monitor.enabled = e);
            }
            "invite-notify" => {
                self.invite_notify.has = true;
                enabled.map(|e| self.invite_notify.enabled = e);
            }
            "labeled-response" => {
                self.labeled_response.has = true;
                enabled.map(|e| self.labeled_response.enabled = e);
            }
            "multi-prefix" => {
                self.multi_prefix.has = true;
                enabled.map(|e| self.multi_prefix.enabled = e);
            }
            "setname" => {
                self.setname.has = true;
                enabled.map(|e| self.setname.enabled = e);
            }
            "standard-replies" => {
                self.standard_replies.has = true;
                enabled.map(|e| self.standard_replies.enabled = e);
            }
            "userhost-in-names" => {
                self.userhost_in_names.has = true;
                enabled.map(|e| self.userhost_in_names.enabled = e);
            }
            _ if cap.starts_with("soju.im") || cap.starts_with("znc.in") => (),
            _ => unimplemented!("cap: {cap}"),
        };
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
pub struct Support {
    pub away_length: Option<i64>,
    pub bot: Option<char>,
    pub case_mapping: Option<String>,
    pub channel_limit: Option<HashMap<char, i64>>,
    pub channel_modes: Option<Vec<String>>,
    pub channel_length: Option<i64>,
    pub channel_types: Option<String>,
    pub chat_history: Option<i64>,
    // search extensions for list command
    pub elist: Option<String>,
    pub excepts: bool,
    pub extban: Option<String>,
    pub forward: Option<String>,
    pub invex: bool,
    pub kick_length: Option<i64>,
    pub max_list: Option<HashMap<String, i64>>,
    pub max_targets: Option<i64>,
    pub modes: bool,
    pub monitor: Option<i64>,
    pub message_ref_types: Option<Vec<String>>,
    pub network: Option<String>,
    pub nick_length: Option<i64>,
    pub prefix: Option<Vec<(char, char)>>,
    pub rp_channel: Option<char>,
    pub rp_user: Option<char>,
    pub safe_list: bool,
    pub safe_rate: bool,
    pub status_message: Option<String>,
    pub target_max: Option<Vec<(String, Option<i64>)>>,
    pub topic_length: Option<i64>,
    pub utf8_mapping: Option<String>,
    pub utf8_only: bool,
    pub vapid: Option<String>,
    pub whox: bool,
}

impl Support {
    pub fn set(&mut self, key: &str, value: Option<&str>) {
        match key.trim() {
            "AWAYLEN" => self.away_length = value.map(|v| i64::from_str(v).unwrap()),
            "BOT" => self.bot = value.map(|v| v.chars().nth(0).unwrap()),
            "CASEMAPPING" => self.case_mapping = value.map(ToOwned::to_owned),
            "CHANLIMIT" => {
                self.channel_limit = value.map(|v| {
                    v.split(',')
                        .map(|ml| {
                            ml.split_once(':')
                                .map(|(a, b)| {
                                    (a.chars().nth(0).unwrap(), i64::from_str(b).unwrap())
                                })
                                .unwrap()
                        })
                        .collect()
                })
            }
            "CHANMODES" => {
                self.channel_modes = value.map(|v| v.split(',').map(ToOwned::to_owned).collect())
            }
            "CHANNELLEN" => self.channel_length = value.map(|v| i64::from_str(v).unwrap()),
            "CHANTYPES" => self.channel_types = value.map(ToOwned::to_owned),
            "draft/CHATHISTORY" | "CHATHISTORY" => {
                self.chat_history = value.map(|v| i64::from_str(v).unwrap())
            }
            "ELIST" => self.elist = value.map(ToOwned::to_owned),
            "EXCEPTS" => self.excepts = true,
            "EXTBAN" => self.extban = value.map(ToOwned::to_owned),
            "FORWARD" => self.forward = value.map(ToOwned::to_owned),
            "INVEX" => self.invex = true,
            "KICKLEN" => self.kick_length = value.map(|v| i64::from_str(v).unwrap()),
            "MAXLIST" => {
                self.max_list = value.map(|v| {
                    v.split(',')
                        .map(|ml| {
                            ml.split_once(':')
                                .map(|(a, b)| (a.to_owned(), i64::from_str(b).unwrap()))
                                .unwrap()
                        })
                        .collect()
                })
            }
            "MAXTARGETS" => self.max_targets = value.map(|v| i64::from_str(v).unwrap()),
            "MODES" => self.modes = true,
            "MONITOR" => self.monitor = value.map(|v| i64::from_str(v).unwrap()),
            "MSGREFTYPES" => {
                self.message_ref_types =
                    value.map(|v| v.split(',').map(ToOwned::to_owned).collect())
            }
            "NETWORK" => self.network = value.map(ToOwned::to_owned),
            "NICKLEN" => self.nick_length = value.map(|v| i64::from_str(v).unwrap()),
            "PREFIX" => {
                self.prefix = value.map(|v| {
                    v[1..]
                        .split_once(')')
                        .map(|(a, b)| a.chars().zip(b.chars()).collect())
                        .unwrap()
                });
            }
            "RPCHAN" => self.rp_channel = value.map(|v| v.chars().nth(0).unwrap()),
            "RPUSER" => self.rp_user = value.map(|v| v.chars().nth(0).unwrap()),
            "SAFELIST" => self.safe_list = true,
            "SAFERATE" => self.safe_rate = true,
            "STATUSMSG" => self.status_message = value.map(ToOwned::to_owned),
            "TARGMAX" => {
                self.target_max = value.map(|v| {
                    v.split(',')
                        .map(|ml| {
                            ml.split_once(':')
                                .map(|(a, b)| (a.to_owned(), i64::from_str(b).ok()))
                                .unwrap()
                        })
                        .collect()
                })
            }
            "TOPICLEN" => self.topic_length = value.map(|v| i64::from_str(v).unwrap()),
            "UTF8MAPPING" => self.utf8_mapping = value.map(ToOwned::to_owned),
            "UTF8ONLY" => self.utf8_only = true,
            "VAPID" => self.vapid = value.map(ToOwned::to_owned),
            "WHOX" => self.whox = true,
            _ => unimplemented!("isupport: {key}, {value:?}"),
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct User {
    pub nickname: String,
    pub username: Option<String>,
    pub realname: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub profile_picture_url: Option<String>,
    pub bot: bool,
}

impl User {
    pub fn new(nickname: String) -> Self {
        Self {
            nickname,
            realname: None,
            username: None,
            display_name: None,
            description: None,
            profile_picture_url: None,
            bot: false,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct Message {
    pub text: Option<TextMessage>,
    pub metadata: MessageMetadata,
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.metadata == other.metadata
    }
}

impl Eq for Message {}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "web", wasm_bindgen)]
pub enum MessageType {
    Privmsg,
    Notice,
    Action,
    Join,
    Part,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct TextMessage {
    pub content: String,
    pub reactions: Vec<Reaction>,
    pub reply: Option<MessageReference>,
    pub redacted: bool,
    pub edited: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct MessageMetadata {
    pub msgid: String,
    pub server_time: i64,
    pub message_type: MessageType,
    pub user: String,
}

impl PartialEq for MessageMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.msgid == other.msgid
    }
}

impl Eq for MessageMetadata {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct MessageReference {
    pub user_id: i64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct Reaction {
    pub user_id: Vec<i64>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen)]
#[cfg_attr(feature = "web", serde(untagged))]
pub enum ServerEvent {
    Joined(Channel),
    ChannelUpdated(ChannelMetadata),
    ServerInfo(ServerMetadata),
    UserList(UserList),
    Privmsg(Message),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen(getter_with_clone))]
pub struct UserList {
    pub channel: String,
    pub users: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "web", derive(Tsify))]
#[cfg_attr(feature = "web", wasm_bindgen)]
#[cfg_attr(feature = "web", serde(untagged))]
pub enum ServerError {
    Generic(String),
}
