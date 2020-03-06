use serde::Deserialize;
use std::process::Command;
use std::vec::Vec;

#[derive(Deserialize, Debug)]
pub struct Annoucment {
    next_hop: String,
    pub prefixes: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    timestamp: f64,
    pub r#type: String,
    #[serde(default)]
    pub announcements: Vec<Annoucment>,
    #[serde(default)]
    pub withdrawals: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct MSG {
    pub r#type: String,
    pub data: Data,
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

pub fn on_withdrawals(prefixes: &[String]) {
    for prefix in prefixes {
        println!("w {:#?}", prefix);
        ipset_action("add", prefix);
    }
}

pub fn on_announcements(annoucments: &[Annoucment]) {
    for a in annoucments {
        for prefix in &a.prefixes {
            println!("a {:#?}", prefix);
            ipset_action("del", prefix);
        }
    }
}

pub fn parse_message(message: &str) -> MSG {
    let json = message;
    let msg: MSG = serde_json::from_str(&json).unwrap();
    msg
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
