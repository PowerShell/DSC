use config::sshd::SshdConfig;

pub mod config;

fn main() {
}

// mainly an example at this point
#[test]
fn test_config() {
    let input_json: &str = r#"
    {
        "channeltimeout": [
            {
                "type": "agent-connection",
                "interval": "1h30m"
            },
            {
                "type": {
                    "subsystem": "sftp"
                },
                "interval": "10s"
            }
        ],
        "permitopen": [
            {
                "ipv4": "1.2.3.4",
                "port": 22
            },
            {
                "ipv6": "2.3.4.5",
                "port": 23
            },
            {
                "host": "localhost", 
                "port": 25
            }
        ],
        "permitlisten": "none",
        "maxstartups": {
            "start": 10,
            "rate": 20,
            "full": 30
        },
        "ciphers": {
            "action": "add",
            "values": ["*"]
        },
        "listenaddress": [
            {
                "hostname": "localhost",
                "address": "1.2.3.4"
            },
            {
                "ipv4": "1.2.3.4",
                "port": 22
            }
        ],
        "persourcemaxstartups": "none",
        "ipqos": {
            "allSessions": "assuredForwardingLevel11"
        },
        "passwordAuthentication": "yes",
        "syslogFacility": "AUTH",
        "authorizedKeysFile": [{
            "value": "test"
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
