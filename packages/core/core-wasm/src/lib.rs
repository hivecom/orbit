use gloo_console::debug;
use wasm_bindgen::prelude::*;

use crate::runtime::ServerList;

pub mod runtime;

#[macro_export]
macro_rules! dbg {
    () => {
        ::gloo_console::debug!(&format!("[{}:{}:{}]", file!(), line!(), column!()));
    };

    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                ::gloo_console::debug!(&format!("[{}:{}:{}] {} = {:#?}",
                    file!(),
                    line!(),
                    column!(),
                    stringify!($val),
                    &&tmp as &dyn std::fmt::Debug,
                ));
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[wasm_bindgen(start)]

pub fn init() {
    console_error_panic_hook::set_once();
    debug!("WASM panic hook initialized");
}

#[wasm_bindgen]
pub async fn initialize_orbit() -> Result<ServerList, JsError> {
    ServerList::new().await
}
