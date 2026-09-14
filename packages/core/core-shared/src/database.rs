use std::fmt;

use crate::state::{Message, OrbitError};

pub trait Database: fmt::Debug + Sized {
    fn insert_message(
        &mut self,
        server_id: i32,
        channel: &str,
        message: Message,
    ) -> impl Future<Output = Result<(), OrbitError>>;

    fn message(
        &mut self,
        msgid: &str,
    ) -> impl Future<Output = Result<Option<(i32, String, Message)>, OrbitError>>;

    fn messages(
        &mut self,
        server_id: i32,
        channel: &str,
    ) -> impl Future<Output = Result<Vec<Message>, OrbitError>>;

    fn add_reaction(
        &mut self,
        msgid: &str,
        react: &str,
        reactor: &str,
    ) -> impl Future<Output = Result<(), OrbitError>>;

    fn remove_reaction(
        &mut self,
        msgid: &str,
        react: &str,
        reactor: &str,
    ) -> impl Future<Output = Result<(), OrbitError>>;
}
