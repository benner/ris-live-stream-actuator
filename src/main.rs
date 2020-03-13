use log::debug;
use ripe_live_stream_actuator::{ipset_action, on_update, parse_message};
use structopt::StructOpt;
use tungstenite::{connect, Message};
use url::Url;

const SUBSCRIBE: &str = r#"{"type": "ris_subscribe", "data": {"host": "rrc21"}}"#;

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(
        default_value = r#"wss://ris-live.ripe.net/v1/ws/?client=rust-workshop-1299"#,
        long
    )]
    ris_url: Url,
}

fn main() {
    env_logger::init();
    let opts = Opts::from_args();
    debug!("options: #{:?}", opts);

    let (mut socket, _response) = connect(opts.ris_url).expect("Can't connect");

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
