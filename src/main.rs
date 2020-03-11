use ripe_live_stream_actuator::{ipset_action, on_update, parse_message};
use tungstenite::{connect, Message};
use url::Url;

// FIXME: shoulbe be parameterized
const CONNECTION: &str = r#"wss://ris-live.ripe.net/v1/ws/?client=rust-workshop-1299"#;
const SUBSCRIBE: &str = r#"{"type": "ris_subscribe", "data": {"host": "rrc21"}}"#;

fn main() {
    env_logger::init();

    let (mut socket, _response) = connect(Url::parse(CONNECTION).unwrap()).expect("Can't connect");

    socket
        .write_message(Message::Text(SUBSCRIBE.to_string()))
        .unwrap();

    loop {
        let msg = socket.read_message().expect("Error reading message");
        if msg.is_text() {
            let msg = parse_message(&msg.into_text().unwrap());
            if msg.data.r#type != "UPDATE" {
                continue;
            }
            on_update(msg.data, ipset_action);
        }
    }
}
