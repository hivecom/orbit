use core_shared::error::OrbitError as ActorOrbitError;
use serde::{Deserialize, Serialize};
use tsify::{Ts, Tsify};
use wasm_bindgen::JsValue;

#[derive(Debug, Tsify, Deserialize, Serialize)]
pub struct OrbitError {
    pub kind: OrbitErrorKind,
    pub description: String,
}

impl OrbitError {
    pub fn unknown_server() -> Self {
        Self {
            kind: OrbitErrorKind::UnknownServer,
            description: String::from("No server with the provided ID is known"),
        }
    }
}

impl From<ActorOrbitError> for OrbitError {
    fn from(error: ActorOrbitError) -> Self {
        let kind = match error {
            ActorOrbitError::NickTaken => OrbitErrorKind::NickTaken,
            ActorOrbitError::SaslFailed(_) => OrbitErrorKind::SaslFailed,
            ActorOrbitError::CapabilityDisabled(_) => OrbitErrorKind::CapabilityDisabled,
            ActorOrbitError::NotFound => OrbitErrorKind::NotFound,
            ActorOrbitError::Generic(_) => OrbitErrorKind::Generic,
            ActorOrbitError::Unknown(_) => OrbitErrorKind::Unknown,
        };

        Self {
            kind,
            description: error.to_string(),
        }
    }
}

impl From<OrbitError> for JsValue {
    fn from(e: OrbitError) -> Self {
        Ts::from_rust(&e)
            .map(JsValue::from)
            .unwrap_or_else(|e| e.to_string().into())
    }
}

#[derive(Debug, Tsify, Clone, Deserialize, Serialize)]
pub enum OrbitErrorKind {
    NickTaken,
    SaslFailed,
    CapabilityDisabled,
    NotFound,
    Generic,
    UnknownServer,
    Unknown,
}

impl From<anyhow::Error> for OrbitError {
    fn from(error: anyhow::Error) -> Self {
        Self {
            kind: OrbitErrorKind::Unknown,
            description: error.to_string(),
        }
    }
}
