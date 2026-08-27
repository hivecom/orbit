use std::collections::HashMap;
#[cfg(not(feature = "web"))]
use std::time::Instant;
#[cfg(feature = "web")]
use web_time::Instant;

#[cfg(feature = "web")]
use crate::dbg;
use crate::{
    SendCommand,
    actor::{
        ActorCommand, ActorMessage, BatchData, CurrentBatch, IrcActor, IrcConnection,
        RequestedHistory, SaslState,
    },
    database::Database,
    response_channels::{CommandKey, CommandResponse, generate_label},
    state::{
        Channel, ChannelRole, ChannelUser, History, Message, MessageMetadata, MessageReference,
        MessageType, OrbitError, ServerEvent, SignedIn, Tags, TextMessage, User,
    },
};
use anyhow::{Context, anyhow};
use base64::prelude::*;
use irc_proto::{BatchSubCommand, CapSubCommand, Command::*, Message as IrcMessage, Response};
use tracing::{debug, error, warn};

impl<C: IrcConnection, DB: Database> IrcActor<C, DB> {
    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn handle_incoming(
        &mut self,
        mut message: IrcMessage,
    ) -> Result<(), OrbitError> {
        match message.command {
            CAP(_, sub, param, caps) => self.handle_caps(sub, param, caps).await?,
            PING(server1, server2) => {
                self.outgoing
                    .pong(server1, server2)
                    .await
                    .context("Failed to send pong")?;
            }
            JOIN(ref channel_name, _, _) => self.handle_join(&message, channel_name).await?,
            PART(ref channel_name, ref comment) => {
                self.handle_part(&message, channel_name, comment).await?
            }
            QUIT(ref comment) => self.handle_quit(&message, comment).await?,
            PRIVMSG(ref target, ref text) => self.handle_privmsg(&message, target, text).await?,
            BATCH(ref reference, ref typ, ref param) => {
                self.handle_batch(&message, reference, typ, param).await?
            }
            ChannelMODE(ref channel_name, ref mode) => {
                let target = message.response_target().unwrap();
                let source = message.source_nickname().unwrap();

                if !self.current_batches.iter().any(|b| b.is_chathistory()) {
                    dbg!(target, source, channel_name, mode);
                }
            }
            TOPIC(ref channel_name, ref text) => {
                let target = message.response_target().unwrap();
                let source = message.source_nickname().unwrap();

                if !self.current_batches.iter().any(|b| b.is_chathistory()) {
                    dbg!(target, source, channel_name, text);
                }
            }
            AUTHENTICATE(ref param) if param == "+" => self.handle_authenticate(&message).await?,
            Raw(ref cmd, ref mut target) if cmd == "TAGMSG" => {
                let target = target.remove(0);
                self.handle_tagmsg(&message, target).await?;
            }
            Response(rpl, params) => self.handle_response(rpl, params).await?,
            ERROR(msg) => self.on_error(OrbitError::Generic(msg)).await?,
            _ => {
                warn!("unhandled message, {message:?}");
            }
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn handle_caps(
        &mut self,
        sub: CapSubCommand,
        param: Option<String>,
        caps: Option<String>,
    ) -> Result<(), OrbitError> {
        match sub {
            CapSubCommand::LS if let Some(caps) = caps => {
                for cap in caps.split_whitespace() {
                    let cap = cap
                        .split('=')
                        .next()
                        .ok_or_else(|| anyhow!("Cap is empty: \"{}\"", cap))?;
                    self.state.capabilities.set_from_name(cap, None);
                }
            }
            CapSubCommand::LS if let Some(param) = param => {
                if param == "*" {
                    return Ok(());
                }
                for cap in param.split_whitespace() {
                    let cap = cap
                        .split('=')
                        .next()
                        .ok_or_else(|| anyhow!("Cap is empty: \"{}\"", cap))?;
                    self.state.capabilities.set_from_name(cap, None);
                }
            }
            CapSubCommand::ACK if let Some(param) = param => {
                for cap in param.split_whitespace() {
                    self.state.capabilities.set_from_name(cap, Some(true));
                }
                self.response_channels
                    .reply(&CommandKey::RequestCaps, CommandResponse::Capabilities)
                    .unwrap();
            }
            _ => {
                debug!("unhandled caps message");
            }
        }

        Ok(())
    }

    pub(crate) async fn handle_join(
        &mut self,
        message: &IrcMessage,
        target: &str,
    ) -> Result<(), OrbitError> {
        let source = message.source_nickname().unwrap();

        let mut tags = Tags::default();
        if let Some(ref t) = message.tags {
            tags = Tags::parse(t);
        }

        if self.current_batches.iter().any(|b| b.is_chathistory()) {
            assert!(tags.server_time.is_some());
        }

        let state_message = Message {
            text: None,
            metadata: MessageMetadata {
                msgid: tags.msgid_with_fallback(&["JOIN", source, target]),
                server_time: tags.server_time_with_fallback() as f64,
                message_type: MessageType::Join,
                user: source.to_string(),
            },
        };

        if self
            .push_batch(target.to_string(), state_message.clone())
            .await
        {
            return Ok(());
        }

        if source == self.state.me.as_ref().unwrap().nickname {
            let channel = Channel::new(target.to_string());
            self.state
                .channels
                .insert(target.to_string(), channel.clone());

            if !self.state.capabilities.history.enabled {
                self.response_channels
                    .reply(
                        &CommandKey::Join(target.to_string()),
                        CommandResponse::Join(target.to_string()),
                    )
                    .map_err(|e| anyhow!("Failed to reply to JOIN command {e:?}"))?;
            }

            self.on_event(ServerEvent::Joined(channel)).await?;

            if self.state.capabilities.history.enabled {
                let label = if self.state.capabilities.labeled_response.enabled {
                    Some(generate_label(&mut self.rng))
                } else {
                    None
                };

                self.requested_history_batches.push((
                    RequestedHistory {
                        target: target.to_string(),
                        label: label.clone(),
                    },
                    Instant::now(),
                ));
                self.history_latest(target.to_string(), None, 5, label)
                    .await
                    .context("Failed to request latest history")?;
            }
        } else {
            let channel = self.channel_mut(target.to_string()).await;

            channel.users.push(ChannelUser {
                nickname: source.to_string(),
                role: ChannelRole::Regular,
            });
        }

        self.database
            .insert_message(self.state.id, target, state_message.clone())
            .await?;

        self.on_event(ServerEvent::Privmsg {
            channel: target.to_string(),
            message: state_message,
        })
        .await?;

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn handle_part(
        &mut self,
        message: &IrcMessage,
        target: &str,
        comment: &Option<String>,
    ) -> Result<(), OrbitError> {
        let mut tags = Tags::default();
        if let Some(ref t) = message.tags {
            tags = Tags::parse(t);
        }
        let source = message.source_nickname().unwrap();

        let state_message = Message {
            text: None,
            metadata: MessageMetadata {
                msgid: tags.msgid_with_fallback(&["PART", source, target]),
                server_time: tags.server_time_with_fallback() as f64,
                message_type: MessageType::Part,
                user: source.to_string(),
            },
        };

        if self
            .push_batch(target.to_string(), state_message.clone())
            .await
        {
            return Ok(());
        }

        self.database
            .insert_message(self.state.id, target, state_message.clone())
            .await?;

        self.on_event(ServerEvent::Privmsg {
            channel: target.to_string(),
            message: state_message,
        })
        .await?;

        let channel = self.channel_mut(target.to_string()).await;

        channel.users.retain(|u| u.nickname != source);

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn handle_quit(
        &mut self,
        message: &IrcMessage,
        comment: &Option<String>,
    ) -> Result<(), OrbitError> {
        let mut tags = Tags::default();
        if let Some(ref t) = message.tags {
            tags = Tags::parse(t);
        }
        let source = message.source_nickname().unwrap();

        for batch in &mut self.current_batches {
            if let BatchData::History {
                messages, target, ..
            } = &mut batch.data
            {
                let state_message = Message {
                    text: None,
                    metadata: MessageMetadata {
                        msgid: tags.msgid_with_fallback(&["QUIT", source, target]),
                        server_time: tags.server_time_with_fallback() as f64,
                        message_type: MessageType::Quit,
                        user: source.to_string(),
                    },
                };

                messages.push(state_message);
                return Ok(());
            }
        }

        let mut channel_quits = Vec::new();

        self.state.users.remove(source);
        for channel in self.state.channels.values_mut() {
            let before = channel.users.len();
            channel.users.retain(|u| u.nickname != source);

            if before > channel.users.len() {
                let state_message = Message {
                    text: None,
                    metadata: MessageMetadata {
                        msgid: tags.msgid_with_fallback(&["QUIT", source, &channel.metadata.name]),
                        server_time: tags.server_time_with_fallback() as f64,
                        message_type: MessageType::Quit,
                        user: source.to_string(),
                    },
                };

                self.database
                    .insert_message(self.state.id, &channel.metadata.name, state_message.clone())
                    .await?;
                channel_quits.push(state_message);
            }
        }

        for msg in channel_quits {
            self.on_event(ServerEvent::Privmsg {
                channel: String::new(),
                message: msg,
            })
            .await?;
        }
        Ok(())
    }

    pub(crate) async fn handle_privmsg(
        &mut self,
        message: &IrcMessage,
        target: &String,
        text: &str,
    ) -> Result<(), OrbitError> {
        let mut tags = Tags::default();
        if let Some(ref t) = message.tags {
            tags = Tags::parse(t);
        }
        assert_eq!(
            self.current_batches.iter().last().map(|b| b.id.as_str()),
            tags.batch.as_deref(),
        );

        if let Some(batch) = self.current_batches.iter_mut().last()
            && let BatchData::Multiline {
                message,
                target: channel,
            } = &mut batch.data
            && let Some(t) = message.text.as_mut()
        {
            if t.content.is_empty() {
                *channel = target.to_string();
                t.content = text.to_string();
            } else {
                t.content = format!("{}\n{}", t.content, text);
            }
            return Ok(());
        }

        let source = message.source_nickname().unwrap();
        let msgid = tags.msgid_with_fallback(&["PRIVMSG", source, target, text]);
        let reply = self.reply_reference(&tags.reply).await?;

        let state_message = Message {
            metadata: MessageMetadata {
                msgid: msgid.clone(),
                server_time: tags.server_time_with_fallback() as f64,
                message_type: MessageType::Privmsg,
                user: source.to_string(),
            },
            text: Some(TextMessage {
                content: text.to_string(),
                reactions: HashMap::new(),
                reply,
                redacted: false,
                edited: false,
                relayed_by: tags.relayed_by,
            }),
        };

        if self.push_batch(target.clone(), state_message.clone()).await {
            return Ok(());
        }

        if let Some(username) = tags.account.clone() {
            let user = self.user_mut(source.to_string()).await;
            user.username = Some(username);
        }

        self.database
            .insert_message(self.state.id, target, state_message.clone())
            .await?;

        if source == self.state.me.as_ref().unwrap().nickname
            && let Err(e) = self.response_channels.reply(
                &CommandKey::Privmsg {
                    target: target.clone(),
                    text: text.to_string(),
                },
                CommandResponse::Privmsg(Box::new(state_message.clone())),
            )
        {
            error!("Failed to reply to PRIVMSG command {e:?}");
        }

        self.on_event(ServerEvent::Privmsg {
            channel: target.clone(),
            message: state_message,
        })
        .await?;

        Ok(())
    }

    async fn reply_reference(
        &mut self,
        reply: &Option<String>,
    ) -> Result<Option<MessageReference>, OrbitError> {
        if let Some(r) = reply {
            let rmsg = self.database.message(r).await?.map(|(_, _, m)| m);

            Ok(Some(MessageReference {
                text: rmsg
                    .as_ref()
                    .and_then(|m| m.text.clone().map(|t| t.content)),
                username: rmsg.map(|m| m.metadata.user),
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) async fn handle_batch(
        &mut self,
        message: &IrcMessage,
        reference: &str,
        typ: &Option<BatchSubCommand>,
        param: &Option<Vec<String>>,
    ) -> Result<(), OrbitError> {
        let mut tags = Tags::default();
        if let Some(ref t) = message.tags {
            tags = Tags::parse(t);
        }
        if let Some(id) = reference.strip_prefix('+') {
            match typ {
                Some(BatchSubCommand::CUSTOM(c)) if c.as_str() == "CHATHISTORY" => {
                    let idx = self
                        .requested_history_batches
                        .iter()
                        .position(|b| b.0.label == tags.label)
                        .expect("Chat history was requested");
                    let channel = self.requested_history_batches.remove(idx).0.target;

                    self.current_batches.push(CurrentBatch {
                        id: id.to_string(),
                        data: BatchData::History {
                            label: tags.label,
                            target: channel,
                            messages: Vec::new(),
                        },
                    });
                }
                Some(BatchSubCommand::CUSTOM(c)) if c.as_str() == "DRAFT/MULTILINE" => {
                    let source = message.source_nickname().unwrap();
                    let target = message.response_target().unwrap();
                    let msgid = tags.msgid_with_fallback(&["MULTILINE", source, target]);

                    let reply = self.reply_reference(&tags.reply).await?;

                    self.current_batches.push(CurrentBatch {
                        id: id.to_string(),
                        data: BatchData::Multiline {
                            target: String::new(),
                            message: Message {
                                metadata: MessageMetadata {
                                    msgid,
                                    message_type: MessageType::Privmsg,
                                    server_time: tags.server_time_with_fallback() as f64,
                                    user: source.to_string(),
                                },
                                text: Some(TextMessage {
                                    content: Default::default(),
                                    reactions: Default::default(),
                                    reply,
                                    redacted: false,
                                    edited: false,
                                    relayed_by: tags.relayed_by,
                                }),
                            },
                        },
                    });
                }
                _ => {
                    self.current_batches.push(CurrentBatch {
                        id: id.to_string(),
                        data: BatchData::Unhandled,
                    });
                    warn!(?typ, ?param, "unhandled BATCH type");
                }
            }
        } else {
            assert_eq!(
                self.current_batches.iter().last().map(|b| b.id.as_str()),
                Some(&reference[1..])
            );

            if let Some(batch) = self.current_batches.pop() {
                match batch.data {
                    BatchData::History {
                        label,
                        target: channel_name,
                        messages,
                    } => {
                        for message in &messages {
                            self.database
                                .insert_message(self.state.id, &channel_name, message.clone())
                                .await?;
                        }

                        let history = History {
                            channel: channel_name.clone(),
                            messages,
                        };

                        let key = if let Some(label) = label {
                            CommandKey::Label(label)
                        } else {
                            CommandKey::History
                        };

                        self.response_channels
                            .reply(
                                &CommandKey::Join(channel_name.clone()),
                                CommandResponse::Join(channel_name.clone()),
                            )
                            .map_err(|e| anyhow!("Failed to reply to JOIN command {e:?}"))?;

                        self.response_channels
                            .reply(&key, CommandResponse::History(history.clone()))
                            .unwrap();
                    }

                    BatchData::Multiline {
                        target,
                        message: state_message,
                    } => {
                        let source = message.source_nickname().unwrap();

                        if self
                            .push_batch(target.to_string(), state_message.clone())
                            .await
                        {
                            return Ok(());
                        }

                        if let Some(username) = tags.account.clone() {
                            let user = self.user_mut(source.to_string()).await;
                            user.username = Some(username);
                        }

                        self.database
                            .insert_message(self.state.id, target.as_str(), state_message.clone())
                            .await?;

                        if source == self.state.me.as_ref().unwrap().nickname
                            && let Err(e) = self.response_channels.reply(
                                &CommandKey::Privmsg {
                                    target: target.to_string(),
                                    text: state_message.text.as_ref().unwrap().content.clone(),
                                },
                                CommandResponse::Privmsg(Box::new(state_message.clone())),
                            )
                        {
                            error!("Failed to reply to PRIVMSG command {e:?}");
                        }

                        self.on_event(ServerEvent::Privmsg {
                            channel: target.to_string(),
                            message: state_message,
                        })
                        .await?;
                    }
                    BatchData::Unhandled => (),
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn handle_authenticate(
        &mut self,
        message: &IrcMessage,
    ) -> Result<(), OrbitError> {
        if let SaslState::Requested {
            nickname,
            realname,
            username,
            password,
        } = self.sasl_state.clone()
        {
            let credentials =
                BASE64_STANDARD.encode(format!("\0{}\0{}", username, password).as_bytes());

            // Chunk overly long credentials
            let mut sending = credentials.as_str();
            while !sending.is_empty() {
                let (chunk, rest) = sending.split_at(400.min(sending.len()));
                self.sasl(chunk.to_string())
                    .await
                    .context("Failed to send SASL chunk")?;

                if rest.is_empty() && chunk.len() == 400 {
                    self.sasl("+".to_string())
                        .await
                        .context("Failed to send SASL end")?;
                }
                sending = rest;
            }
            self.nick(nickname.clone())
                .await
                .context("Failed to send NICK")?;
            self.user(username.clone(), String::from("0"), realname.clone())
                .await
                .context("Failed to send USER")?;

            self.state.me = Some(User {
                nickname,
                username: Some(username),
                realname: Some(realname),
                display_name: None,
                description: None,
                profile_picture_url: None,
                bot: false,
            });
        }

        Ok(())
    }

    pub(crate) async fn handle_tagmsg(
        &mut self,
        message: &IrcMessage,
        target: String,
    ) -> Result<(), OrbitError> {
        let mut tags = Tags::default();
        if let Some(ref t) = message.tags {
            tags = Tags::parse(t);
        }

        let is_unreact = tags.unreact.is_some();
        if let Some(react) = tags.react.or(tags.unreact)
            && let Some(reply) = tags.reply
        {
            let reactor = message.source_nickname().unwrap().to_string();
            if is_unreact {
                self.database
                    .remove_reaction(&reply, &react, &reactor)
                    .await?;
            } else {
                self.database.add_reaction(&reply, &react, &reactor).await?;
            }

            self.on_event(ServerEvent::React {
                target_message: reply,
                user: reactor,
                text: react,
                is_unreact,
            })
            .await?;
        }
        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn handle_response(
        &mut self,
        rpl: Response,
        params: Vec<String>,
    ) -> Result<(), OrbitError> {
        match rpl {
            Response::RPL_MOTDSTART => {
                self.state.metadata.reset_motd();
            }
            Response::RPL_MOTD => {
                self.state.metadata.add_motd(&params[1]);
            }
            Response::RPL_ENDOFMOTD => self
                .on_event(ServerEvent::ServerInfo(self.state.metadata.clone()))
                .await
                .map_err(|e| anyhow!("Failed to send server event {e:?}"))?,
            Response::RPL_SASLSUCCESS => {
                self.response_channels
                    .reply(
                        &CommandKey::SignIn,
                        CommandResponse::SignIn(Ok(SignedIn::User)),
                    )
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;

                self.cap_end().await.context("Failed to send CAP END")?;
            }
            Response::RPL_WELCOME => {
                self.response_channels
                    .reply(
                        &CommandKey::SignIn,
                        CommandResponse::SignIn(Ok(SignedIn::Guest)),
                    )
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;
            }
            Response::RPL_LOGGEDIN => {
                self.state.me.as_mut().unwrap().username = Some(params[2].clone());
            }
            Response::ERR_SASLFAIL => {
                self.response_channels
                    .reply(
                        &CommandKey::SignIn,
                        CommandResponse::SignIn(Err(OrbitError::SaslFailed(params[1].to_string()))),
                    )
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;
            }
            Response::ERR_NICKNAMEINUSE => {
                self.response_channels
                    .reply(
                        &CommandKey::SignIn,
                        CommandResponse::SignIn(Err(OrbitError::NickTaken)),
                    )
                    .map_err(|e| anyhow!("Failed to reply to sign in command {e:?}"))?;
            }
            Response::RPL_TOPIC => {
                let channel_name = params[1].to_string();
                let topic = params[2].to_string();
                let channel = self.channel_mut(channel_name).await;

                channel.metadata.topic = Some(topic);

                let metadata = channel.metadata.clone();
                self.on_event(ServerEvent::ChannelUpdated(metadata))
                    .await
                    .unwrap();
            }
            Response::RPL_NAMREPLY => {
                let channel_name = params[2].to_string();
                let users: Vec<_> = params[3].split_whitespace().map(|u| u.to_owned()).collect();

                let mut channel_users = Vec::new();
                for mut user in users {
                    if let Some(prefix) = &self.state.support.prefix
                        && let Some((role, _)) =
                            prefix.iter().find(|(_, p)| Some(*p) == user.chars().nth(0))
                    {
                        user.remove(0);
                        let role = ChannelRole::from(*role);
                        channel_users.push(ChannelUser {
                            role,
                            nickname: user.clone(),
                        });
                    } else {
                        channel_users.push(ChannelUser {
                            role: ChannelRole::Regular,
                            nickname: user.clone(),
                        });
                    }

                    self.state
                        .users
                        .entry(user.clone())
                        .or_insert_with(|| User::new(user));
                }

                let channel = self.channel_mut(channel_name).await;

                channel.users = channel_users;
            }
            Response::RPL_ENDOFNAMES => self
                .on_event(ServerEvent::UserList {
                    channel: params[1].to_string(),
                    users: self.state.channels.get(&params[1]).unwrap().users.clone(),
                })
                .await
                .map_err(|e| anyhow!("Failed to send server event {e:?}"))?,
            Response::RPL_ISUPPORT => {
                for option in &params[1..(params.len() - 1)] {
                    let (key, value) = option.split_once('=').unzip();
                    self.state.support.set(key.unwrap_or(option), value);
                }
                self.state.metadata.name = self.state.support.network.clone();
            }
            Response::RPL_YOURHOST
            | Response::RPL_CREATED
            | Response::RPL_MYINFO
            | Response::RPL_LUSERCLIENT
            | Response::RPL_LUSEROP
            | Response::RPL_LUSERUNKNOWN
            | Response::RPL_LUSERCHANNELS
            | Response::RPL_LUSERME
            | Response::RPL_TOPICWHOTIME
            | Response::RPL_LOCALUSERS
            | Response::RPL_UMODEIS
            | Response::RPL_GLOBALUSERS => (),
            _ => {
                warn!("unhandled response");
            }
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn handle_command(&mut self, cmd: ActorMessage) -> Result<(), OrbitError> {
        match cmd.command {
            ActorCommand::GetState => {
                let mut state = self.state.clone();
                for (name, channel) in &mut state.channels {
                    channel.messages = self.database.messages(self.state.id, name).await?;
                }

                cmd.reply_tx
                    .unwrap()
                    .send(CommandResponse::GetState(Box::new(state)))
                    .unwrap();
            }
            ActorCommand::GetChannelState(channel_name) => {
                let Some(mut channel) = self.state.channels.get(&channel_name).cloned() else {
                    cmd.reply_tx
                        .unwrap()
                        .send(CommandResponse::GetChannelState(Box::new(None)))
                        .unwrap();
                    return Ok(());
                };
                channel.messages = self.database.messages(self.state.id, &channel_name).await?;
                cmd.reply_tx
                    .unwrap()
                    .send(CommandResponse::GetChannelState(Box::new(Some(channel))))
                    .unwrap();
            }
            ActorCommand::SignIn {
                nick,
                user,
                realname,
                password,
            } => {
                self.response_channels
                    .register(CommandKey::SignIn, cmd.reply_tx.unwrap());
                self.sign_in(nick, user, realname, password).await?;
            }
            ActorCommand::SignInAnonymous {
                nick,
                user,
                realname,
            } => {
                self.response_channels
                    .register(CommandKey::SignIn, cmd.reply_tx.unwrap());
                self.sign_in_anonymous(nick, user, realname).await?;
            }
            ActorCommand::Join { channel, password } => {
                self.response_channels
                    .register(CommandKey::Join(channel.clone()), cmd.reply_tx.unwrap());
                self.join(channel, password).await.unwrap();
            }
            ActorCommand::Privmsg { text, target } => {
                self.response_channels.register(
                    CommandKey::Privmsg {
                        target: target.clone(),
                        text: text.clone(),
                    },
                    cmd.reply_tx.unwrap(),
                );
                self.privmsg(target, text).await.unwrap();
            }
            ActorCommand::AddEventHandler { handler } => {
                self.event_handlers.push(handler);
            }
            ActorCommand::AddErrorHandler { handler } => {
                self.error_handlers.push(handler);
            }
            ActorCommand::AddDisconectHandler { handler } => {
                self.disconnect_handlers.push(handler);
            }
            ActorCommand::RequestHistory {
                channel,
                before_msgid,
            } => {
                let label = if self.state.capabilities.labeled_response.enabled {
                    Some(
                        self.response_channels
                            .register_labeled(cmd.reply_tx.unwrap()),
                    )
                } else {
                    self.response_channels
                        .register(CommandKey::History, cmd.reply_tx.unwrap());

                    None
                };

                self.requested_history_batches.push((
                    RequestedHistory {
                        target: channel.clone(),
                        label: label.clone(),
                    },
                    Instant::now(),
                ));
                self.history_before(channel, format!("msgid={before_msgid}"), 5, label)
                    .await
                    .context("Failed to send history before")?;
            }
        }

        Ok(())
    }
}
