use gloo_console::log;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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

static MOTD: &'static str = "
======================================================================
                            ....''''''....
            .xdc,     ...',,;,,,,,,,,,,,,;,,'...
            .XXXX0;.,,,,,,,''............'',,,,,,,..
            .XKKKKK:,'..........................',,,,'.
            cXKKKKKd..............  ............. dc,,,,'.
          .'oXKKKKKd.....  ................  .... KK0dc,,,'.
        .,''oKKKKKKd   ........................   0KKKKOl''''.
      .''''.cK00000d. ..........................  0000000'.'''.
     .'''...:K00000d............................. 0000000' ..'''.
    .'''....:000000d............................. 0000000'....'''.
   .'''.... :0OOOOOd............................. 0OOOOOO' ....'''.
  ......... :0OOOOOo............................. OOOOOOO' .........
  ........  :OOOOOOo............................. OOOOOOO'  ........
 ........   ;Okkkkko............''''''........... Okkkkkk.   ........
 .......    ;kkkkkko...........',;;;;,'.......... kkkkkkk.    .......
 ... ...    ;kxxkkkd;;;;;;::::codkkkkxolc:::;;;;;,kkkkkxk.    ... ...
.... ...    ;kxxxxxkkkkkkkkkkO0XK0NNKKXKOkkkkkkkkkkkxxxxx.    ... ...
.... ..     ;xxxxxxxkkkkkOOOO0XN0NMMW0XX0OOOkkkkkkkkxxxxx.    ... ...
 ... ...    ,xdddxxxxxxxxxxxkkOXK0KX00X0kkkkkkxxxxxxxdddd.    ... ...
 ... ...    ,ddddddl''''',,,,;:cldxddoc:;,,,,''''.xdddddd.    ... ...
 ...  ..    ,doodddc............',,,,'........... dddddod.    ..  ...
  ... ...   ,ooooooc. ........................... doooooo.   ... ...
  ...  ..   ,oooooo:. ..........................  ooooooo.  ...  ...
   ...  ..  'olllll:   .........................  ollllll.  ..  ...
          . 'llllll:      ....................    ollllll. .
            'llllll;           .........          lllllll.
            'lccccc;                              lcccccc.
            'cccccc;                              ccccccc.
             .;cccc,                              ccccccc.
               .';:,                              c::::::.
        __  ___   ..     .                  .     :::::::.
       / / / (_)   _____  _________  ____ ___     .::::::.
      / /_/ / / | / / _ \\/ ___/ __ \\/ __ `__ \\      .::::
     / __  / /| |/ /  __/ /__/ /_/ / / / / / /
irc./_/ /_/_/ |___/\\___/\\___/\\____/_/ /_/ /_/.net (Frankfurt, Germany)
======================================================================
          Port:        6697 (TLS only), 8097 (Websocket)
          Admins:      Jokler, kilmanio & zealsprince
";

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub async fn initialize_orbit() -> Vec<Server> {
    let me = User {
        id: 0,
        display_name: Some(String::from("Bokler")),
        // username@host
        username: String::from("Jokler"),
        realname: String::from("Jokler"),
        nickname: String::from("Jokler"),
        account: None,
        description: Some(String::from("what is the syntax for a multiline string")),
        profile_picture_url: Some(String::from("https://avatars.githubusercontent.com/u/5204415")),
        bot: false,
    };

    vec![
        Server {
            id: 0,
            metadata: ServerMetadata {
                name: String::from("Hivecom"),
                motd: String::from(MOTD),
                address: String::from("ws://irc.hivecom.net:8097"),
                transponder_url: None,
                satellite_url: None,
                depot_url: None,
            },
            channels: vec![
                Channel {
                    id: 0,
                    metadata: ChannelMetadata {
                        name: String::from("#orbit"),
                        display_name: Some(String::from("Orbit")),
                        description: Some(String::from("Channel for this app.")),
                        icon: None,
                    },
                    // messages: vec![
                    //     Message::Join(EventMessage{
                    //         msgid: String::from("msgidmsgidmsgidmsgid"),
                    //         server_time: 1782601862,
                    //         user_id: 0,
                    //     }),
                    //     Message::Privmsg(TextMessage{
                    //         msgid: String::from("msgidmsgidmsgidmsgid"),
                    //         server_time: 1782601862,
                    //         user_id: 0,
                    //         text: String::from("Hey there!"),
                    //         reactions: vec![
                    //             Reaction{user_id: vec![0], text: String::from("🫪")}
                    //         ],
                    //         reply: None,
                    //         redacted: false, // tombstone overlay; original text is NOT retained when set
                    //         edited: false,   // set when text was edited in place (post-MVP)
                    //     }),
                    // ]
                    // .into_iter()
                    // .map(|m| serde_wasm_bindgen::to_value(&m).unwrap())
                    // .collect(),
                    users: vec![0],
                },
            ],
            capabilities: Capabilities {
                sasl: true,
                message_tags: true,
                message_redaction: true,
                message_edit: false,
                multiline: true,
                metadata: true,
                webpush: true,
            },
            users: vec![me.clone()],
            me,
        }
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct Server {
    pub id: i64,
    pub metadata: ServerMetadata,
    pub channels: Vec<Channel>,
    pub capabilities: Capabilities,
    pub users: Vec<User>,
    pub me: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct ServerMetadata {
    pub name: String,
    pub motd: String,
    pub address: String,

    pub transponder_url: Option<String>,
    pub satellite_url: Option<String>,
    pub depot_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct Channel {
    pub id: i64,
    pub metadata: ChannelMetadata,
    // pub messages: Vec<JsValue>,
    pub users: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct ChannelMetadata {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Capabilities {
    // irc
    pub sasl: bool,
    // reacts / replies
    pub message_tags: bool,
    pub message_redaction: bool,
    pub message_edit: bool,
    pub multiline: bool,
    pub metadata: bool,
    pub webpush: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct User {
    id: i64,
    username: String,
    realname: String,
    nickname: String,
    account: Option<String>,
    display_name: Option<String>,
    description: Option<String>,
    profile_picture_url: Option<String>,
    bot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Privmsg(TextMessage),
    Notice(TextMessage),
    Action(TextMessage),
    Join(EventMessage),
    Part(EventMessage),
    Quit(EventMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct TextMessage {
    msgid: String,                   // primary key: server msgid, or a synthetic evt:* key for keyless lines
    server_time: i64,                // sort key, from server-time (epoch ms)
    user_id: i64,                    // nick at send time (display only; account is authoritative)
    text: String,
    reactions: Vec<Reaction>,
    reply: Option<MessageReference>,
    redacted: bool,                  // tombstone overlay; original text is NOT retained when set
    edited: bool,                    // set when text was edited in place (post-MVP)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct EventMessage {
    msgid: String,                   // primary key: server msgid, or a synthetic evt:* key for keyless lines
    server_time: i64,                // sort key, from server-time (epoch ms)
    user_id: i64,                    // nick at send time (display only; account is authoritative)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct MessageReference {
    user_id: i64,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct Reaction {
    user_id: Vec<i64>,
    text: String,
}
