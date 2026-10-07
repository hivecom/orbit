use core_shared::error::OrbitError as ActorOrbitError;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::JsValue;

#[derive(Debug, Tsify, Deserialize, Serialize)]
pub struct OrbitError {
    pub kind: OrbitErrorKind,
    pub description: String,
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
        serde_wasm_bindgen::to_value(&e).unwrap_or_else(|e| e.to_string().into())
    }
}

#[derive(Debug, Tsify, Clone, Deserialize, Serialize)]
pub enum OrbitErrorKind {
    NickTaken,
    SaslFailed,
    CapabilityDisabled,
    NotFound,
    Generic,
    Serialize,
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

impl From<serde_wasm_bindgen::Error> for OrbitError {
    fn from(error: serde_wasm_bindgen::Error) -> Self {
        Self {
            kind: OrbitErrorKind::Serialize,
            description: error.to_string(),
        }
    }
}
