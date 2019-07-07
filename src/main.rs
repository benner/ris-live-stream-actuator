extern crate env_logger;
extern crate serde;
extern crate serde_json;
extern crate tungstenite;
extern crate url;

use serde::Deserialize;
use std::process::Command;
use std::vec::Vec;
use std::{thread, time};
use tungstenite::{connect, Message};
use url::Url;

// FIXME: shoulbe be parameterized
const CONNECTION: &str = r#"wss://ris-live.ripe.net/v1/ws/?client=rust-workshop-1299"#;
const SUBSCRIBE: &str = r#"{"type": "ris_subscribe", "data": {"host": "rrc21"}}"#;

#[derive(Deserialize, Debug)]
struct Annoucment {
    next_hop: String,
    prefixes: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Data {
    timestamp: f64,
    r#type: String,
    #[serde(default)]
    announcements: Vec<Annoucment>,
    #[serde(default)]
    withdrawals: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct MSG {
    r#type: String,
    data: Data,
}

fn is_v6(network: &str) -> bool {
    network.contains(':')
}

fn ipset_table(prefix: &str) -> &str {
    if is_v6(prefix) {
        "ris-ipv6"
    } else {
        "ris-ipv4"
    }
}

fn ipset_action(action: &str, prefix: &str) {
    let output = Command::new("ipset")
        .arg(action)
        .arg("-exist")
        .arg(ipset_table(prefix))
        .arg(prefix)
        .output()
        .expect("failed to execute ipset");

    if !output.status.success() {
        panic!("ipset failed to add prefix");
    }
}

fn on_withdrawals(prefixes: &[String]) {
    for prefix in prefixes {
        println!("w {:#?}", prefix);
        ipset_action("add", prefix);
    }
}

fn on_announcements(annoucments: &[Annoucment]) {
    for a in annoucments {
        for prefix in &a.prefixes {
            println!("a {:#?}", prefix);
            ipset_action("del", prefix);
        }
    }
}

fn parse_message(message: &str) -> MSG {
    let json = message;
    let msg: MSG = serde_json::from_str(&json).unwrap();
    msg
}

fn main() {
    env_logger::init();

    let (mut socket, _response) = connect(Url::parse(CONNECTION).unwrap()).expect("Can't connect");

    thread::sleep(time::Duration::from_millis(100));
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

            if msg.data.withdrawals.is_empty() {
                on_announcements(&msg.data.announcements)
            } else {
                on_withdrawals(&msg.data.withdrawals)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_is_v6() {
        assert_eq!(
            super::is_v6("2001:0db8:85a3:0000:0000:8a2e:0370:7334/64"),
            true
        );
    }

    #[test]
    fn test_not_v6() {
        assert_eq!(super::is_v6("172.16.254.1/24"), false);
    }

    #[test]
    fn test_ipset_table_v6() {
        assert_eq!(
            super::ipset_table("2001:0db8:85a3:0000:0000:8a2e:0370:7334/64"),
            "ris-ipv6"
        );
    }

    #[test]
    fn test_ipset_table_v4() {
        assert_eq!(super::ipset_table("172.16.254.1/24"), "ris-ipv4");
    }

    #[test]
    #[should_panic]
    fn test_parse_message_empty() {
        let _message = super::parse_message("{}");
    }

    #[test]
    fn test_parse_message_defaul_values() {
        let message: super::MSG = super::parse_message(
            r#"
            {
                "type": "ris_data",
                "data": {
                            "timestamp": 1561021440.88,
                            "type": "UPDATE"
                        }
            }
            "#,
        );
        assert_eq!(message.data.announcements.len(), 0);
        assert_eq!(message.data.withdrawals.len(), 0);
    }

    #[test]
    fn test_parse_message_withdrawals() {
        let message: super::MSG = super::parse_message(
            r#"
            {
                "type": "ris_data",
                "data": {
                            "timestamp": 1561021440.88,
                            "type": "UPDATE",
                            "withdrawals": ["192.168.0.1/24"]
                        }
            }
            "#,
        );
        assert_eq!(message.data.withdrawals.len(), 1);
        assert_eq!(message.data.withdrawals, vec!["192.168.0.1/24"]);
    }

    #[test]
    fn test_parse_message_announcements() {
        let message: super::MSG = super::parse_message(
            r#"
            {
                "type": "ris_data",
                "data": {
                            "timestamp": 1561021440.88,
                            "type": "UPDATE",
                            "announcements": [{
                                "next_hop": "192.168.0.1",
                                "prefixes": ["192.168.2.0/24"]
                            }]
                        }
            }
            "#,
        );
        assert_eq!(message.data.announcements.len(), 1);
        assert_eq!(message.data.announcements[0].next_hop, "192.168.0.1");
        assert_eq!(
            message.data.announcements[0].prefixes,
            vec!["192.168.2.0/24"]
        );
    }
}
