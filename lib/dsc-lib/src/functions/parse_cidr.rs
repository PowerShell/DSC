// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Function;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use ipnetwork::IpNetwork;
use rust_i18n::t;
use serde_json::{json, Value};
use tracing::debug;

#[derive(Debug, Default)]
pub struct ParseCidr {}

impl Function for ParseCidr {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "parseCidr".to_string(),
            description: t!("functions.parseCidr.description").to_string(),
            category: vec![FunctionCategory::Object],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Object],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.parseCidr.invoked"));

        let cidr_string = args[0].as_str().unwrap();

        // Validate that the input contains a CIDR prefix (contains '/')
        if !cidr_string.contains('/') {
            return Err(DscError::FunctionArg(
                "parseCidr".to_string(),
                t!("functions.parseCidr.invalidCidr", cidr = cidr_string).to_string(),
            ));
        }

        let network = cidr_string.parse::<IpNetwork>().map_err(|_| {
            DscError::FunctionArg(
                "parseCidr".to_string(),
                t!("functions.parseCidr.invalidCidr", cidr = cidr_string).to_string(),
            )
        })?;

        let result = match network {
            IpNetwork::V4(net) => {
                let network_addr = net.network();
                let broadcast_addr = net.broadcast();
                let first_usable = if net.prefix() == 32 {
                    network_addr
                } else {
                    let first_ip = u32::from(network_addr);
                    std::net::Ipv4Addr::from(first_ip + 1)
                };
                let last_usable = if net.prefix() == 32 {
                    broadcast_addr
                } else {
                    let last_ip = u32::from(broadcast_addr);
                    std::net::Ipv4Addr::from(last_ip - 1)
                };

                json!({
                    "network": network_addr.to_string(),
                    "netmask": net.mask().to_string(),
                    "broadcast": broadcast_addr.to_string(),
                    "firstUsable": first_usable.to_string(),
                    "lastUsable": last_usable.to_string(),
                    "cidr": net.prefix()
                })
            }
            IpNetwork::V6(net) => {
                let network_addr = net.network();
                let broadcast_addr = net.broadcast();

                json!({
                    "network": network_addr.to_string(),
                    "netmask": net.mask().to_string(),
                    "broadcast": broadcast_addr.to_string(),
                    "firstUsable": network_addr.to_string(),
                    "lastUsable": broadcast_addr.to_string(),
                    "cidr": net.prefix()
                })
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn parse_cidr_ipv4_standard() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[parseCidr('192.168.1.0/24')]", &Context::new())
            .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("network").unwrap().as_str().unwrap(), "192.168.1.0");
        assert_eq!(
            obj.get("netmask").unwrap().as_str().unwrap(),
            "255.255.255.0"
        );
        assert_eq!(
            obj.get("broadcast").unwrap().as_str().unwrap(),
            "192.168.1.255"
        );
        assert_eq!(
            obj.get("firstUsable").unwrap().as_str().unwrap(),
            "192.168.1.1"
        );
        assert_eq!(
            obj.get("lastUsable").unwrap().as_str().unwrap(),
            "192.168.1.254"
        );
        assert_eq!(obj.get("cidr").unwrap().as_u64().unwrap(), 24);
    }

    #[test]
    fn parse_cidr_ipv4_slash_32() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[parseCidr('192.168.1.100/32')]", &Context::new())
            .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("network").unwrap().as_str().unwrap(),
            "192.168.1.100"
        );
        assert_eq!(
            obj.get("broadcast").unwrap().as_str().unwrap(),
            "192.168.1.100"
        );
        assert_eq!(
            obj.get("firstUsable").unwrap().as_str().unwrap(),
            "192.168.1.100"
        );
        assert_eq!(
            obj.get("lastUsable").unwrap().as_str().unwrap(),
            "192.168.1.100"
        );
        assert_eq!(obj.get("cidr").unwrap().as_u64().unwrap(), 32);
    }

    #[test]
    fn parse_cidr_ipv4_slash_16() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[parseCidr('10.0.0.0/16')]", &Context::new())
            .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("network").unwrap().as_str().unwrap(), "10.0.0.0");
        assert_eq!(obj.get("netmask").unwrap().as_str().unwrap(), "255.255.0.0");
        assert_eq!(
            obj.get("broadcast").unwrap().as_str().unwrap(),
            "10.0.255.255"
        );
        assert_eq!(
            obj.get("firstUsable").unwrap().as_str().unwrap(),
            "10.0.0.1"
        );
        assert_eq!(
            obj.get("lastUsable").unwrap().as_str().unwrap(),
            "10.0.255.254"
        );
        assert_eq!(obj.get("cidr").unwrap().as_u64().unwrap(), 16);
    }

    #[test]
    fn parse_cidr_ipv4_slash_20() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[parseCidr('10.144.0.0/20')]", &Context::new())
            .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("network").unwrap().as_str().unwrap(), "10.144.0.0");
        assert_eq!(
            obj.get("netmask").unwrap().as_str().unwrap(),
            "255.255.240.0"
        );
        assert_eq!(
            obj.get("broadcast").unwrap().as_str().unwrap(),
            "10.144.15.255"
        );
        assert_eq!(
            obj.get("firstUsable").unwrap().as_str().unwrap(),
            "10.144.0.1"
        );
        assert_eq!(
            obj.get("lastUsable").unwrap().as_str().unwrap(),
            "10.144.15.254"
        );
        assert_eq!(obj.get("cidr").unwrap().as_u64().unwrap(), 20);
    }

    #[test]
    fn parse_cidr_ipv6() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[parseCidr('2001:db8::/32')]", &Context::new())
            .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("network").unwrap().as_str().unwrap(), "2001:db8::");
        assert_eq!(obj.get("cidr").unwrap().as_u64().unwrap(), 32);
        assert!(obj.get("netmask").is_some());
        assert!(obj.get("broadcast").is_some());
        assert!(obj.get("firstUsable").is_some());
        assert!(obj.get("lastUsable").is_some());
    }

    #[test]
    fn parse_cidr_ipv6_full() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[parseCidr('fe80::1/64')]", &Context::new())
            .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("network").unwrap().as_str().unwrap(), "fe80::");
        assert_eq!(obj.get("cidr").unwrap().as_u64().unwrap(), 64);
        assert!(obj.get("netmask").is_some());
        assert!(obj.get("broadcast").is_some());
    }

    #[test]
    fn parse_cidr_invalid_format() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[parseCidr('invalid')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn parse_cidr_invalid_prefix() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[parseCidr('192.168.1.0/33')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn parse_cidr_no_prefix() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[parseCidr('192.168.1.0')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn parse_cidr_with_host_bits() {
        // CIDR with host bits set should still be parsed (normalized to network address)
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[parseCidr('192.168.1.100/24')]", &Context::new())
            .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("network").unwrap().as_str().unwrap(), "192.168.1.0");
        assert_eq!(
            obj.get("broadcast").unwrap().as_str().unwrap(),
            "192.168.1.255"
        );
    }
}
