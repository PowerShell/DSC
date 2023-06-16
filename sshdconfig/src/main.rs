use config::sshd::SshdConfig;
use std::{io::{self, Read}, process::exit};

pub mod config;

fn main() {
}

// mainly an example at this point
#[test]
fn test_config() {
    let input_json: &str = r#"
    {
        "persourcemaxstartups": "none",
        "ipqos": {
            "allSessions": "none"
        },
        "passwordAuthentication": "yes",
        "syslogFacility": "AUTH",
        "authorizedKeysFile": [{
            "value": "test"
        }],
        "channeltimeout": [{
            "type": "agent-connection",
            "interval": "5m"
        }],
        "port": [
            { "value": 24 },
            { "value": 23 }
        ],
        "match": [
            {
                "conditionalKey": "group",
                "conditionalValue": "administrator",
                "data": {
                    "PasswordAuthentication": "yes",
                    "authorizedKeysFile": [{
                        "value": "test.txt",
                        "_ensure": "Absent"
                    }]
                }
            },
            {
                "conditionalKey": "user",
                "conditionalValue": "anoncvs",
                "data": {
                    "passwordAuthentication": {
                        "value": "no",
                        "_ensure": "Absent"
                    },
                    "authorizedKeysFile": ["test.txt"]
                }
            }
        ]
    }
    "#;
    let config: SshdConfig = serde_json::from_str(input_json).unwrap();
    //println!("{:?}", &config);
    let json = config.to_json();
    println!("{}", &json);
}
