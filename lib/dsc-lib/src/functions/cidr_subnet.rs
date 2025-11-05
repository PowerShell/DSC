// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Function;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use rust_i18n::t;
use serde_json::Value;
use std::net::{Ipv4Addr, Ipv6Addr};
use tracing::debug;

#[derive(Debug, Default)]
pub struct CidrSubnet {}

impl Function for CidrSubnet {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "cidrSubnet".to_string(),
            description: t!("functions.cidrSubnet.description").to_string(),
            category: vec![FunctionCategory::CIDR],
            min_args: 3,
            max_args: 3,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
                vec![FunctionArgKind::Number],
                vec![FunctionArgKind::Number],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.cidrSubnet.invoked"));

        let cidr_string = args[0].as_str().unwrap();
        let new_cidr = args[1].as_i64().unwrap() as u8;
        let subnet_index = args[2].as_i64().unwrap();

        if subnet_index < 0 {
            return Err(DscError::FunctionArg(
                "cidrSubnet".to_string(),
                t!("functions.cidrSubnet.negativeSubnetIndex").to_string(),
            ));
        }

        let network = cidr_string.parse::<IpNetwork>().map_err(|_| {
            DscError::FunctionArg(
                "cidrSubnet".to_string(),
                t!("functions.cidrSubnet.invalidCidr", cidr = cidr_string).to_string(),
            )
        })?;

        let result = match network {
            IpNetwork::V4(net) => {
                if new_cidr > 32 {
                    return Err(DscError::FunctionArg(
                        "cidrSubnet".to_string(),
                        t!("functions.cidrSubnet.invalidPrefixV4", prefix = new_cidr).to_string(),
                    ));
                }

                if new_cidr < net.prefix() {
                    return Err(DscError::FunctionArg(
                        "cidrSubnet".to_string(),
                        t!(
                            "functions.cidrSubnet.newCidrTooSmall",
                            newCidr = new_cidr,
                            currentCidr = net.prefix()
                        )
                        .to_string(),
                    ));
                }

                calculate_ipv4_subnet(net, new_cidr, subnet_index as usize)?
            }
            IpNetwork::V6(net) => {
                if new_cidr > 128 {
                    return Err(DscError::FunctionArg(
                        "cidrSubnet".to_string(),
                        t!("functions.cidrSubnet.invalidPrefixV6", prefix = new_cidr).to_string(),
                    ));
                }

                if new_cidr < net.prefix() {
                    return Err(DscError::FunctionArg(
                        "cidrSubnet".to_string(),
                        t!(
                            "functions.cidrSubnet.newCidrTooSmall",
                            newCidr = new_cidr,
                            currentCidr = net.prefix()
                        )
                        .to_string(),
                    ));
                }

                calculate_ipv6_subnet(net, new_cidr, subnet_index as usize)?
            }
        };

        Ok(Value::String(result))
    }
}

fn calculate_ipv4_subnet(
    net: Ipv4Network,
    new_prefix: u8,
    index: usize,
) -> Result<String, DscError> {
    let old_prefix = net.prefix();
    let network_addr = net.network();

    let subnet_bits = new_prefix - old_prefix;
    let num_subnets = 2_usize.pow(subnet_bits as u32);

    if index >= num_subnets {
        return Err(DscError::FunctionArg(
            "cidrSubnet".to_string(),
            t!(
                "functions.cidrSubnet.subnetIndexOutOfRange",
                index = index,
                maxIndex = num_subnets - 1
            )
            .to_string(),
        ));
    }

    let host_bits = 32 - new_prefix;
    let subnet_size = 2_u32.pow(host_bits as u32);

    let base_addr = u32::from(network_addr);
    let subnet_addr = base_addr + (index as u32 * subnet_size);
    let subnet_ip = Ipv4Addr::from(subnet_addr);

    let subnet = Ipv4Network::new(subnet_ip, new_prefix).map_err(|_| {
        DscError::FunctionArg(
            "cidrSubnet".to_string(),
            t!("functions.cidrSubnet.subnetCreationFailed").to_string(),
        )
    })?;

    Ok(format!("{}/{}", subnet.network(), new_prefix))
}

fn calculate_ipv6_subnet(
    net: Ipv6Network,
    new_prefix: u8,
    index: usize,
) -> Result<String, DscError> {
    let old_prefix = net.prefix();
    let network_addr = net.network();

    let subnet_bits = new_prefix - old_prefix;

    if subnet_bits > 32 {
        return Err(DscError::FunctionArg(
            "cidrSubnet".to_string(),
            t!("functions.cidrSubnet.tooManySubnets").to_string(),
        ));
    }

    let num_subnets = 2_usize.pow(subnet_bits as u32);

    if index >= num_subnets {
        return Err(DscError::FunctionArg(
            "cidrSubnet".to_string(),
            t!(
                "functions.cidrSubnet.subnetIndexOutOfRange",
                index = index,
                maxIndex = num_subnets - 1
            )
            .to_string(),
        ));
    }

    let host_bits = 128 - new_prefix;
    let base_addr = u128::from(network_addr);

    let subnet_size = if host_bits < 128 {
        2_u128.pow(host_bits as u32)
    } else {
        0
    };

    let subnet_addr = base_addr + (index as u128 * subnet_size);
    let subnet_ip = Ipv6Addr::from(subnet_addr);

    let subnet = Ipv6Network::new(subnet_ip, new_prefix).map_err(|_| {
        DscError::FunctionArg(
            "cidrSubnet".to_string(),
            t!("functions.cidrSubnet.subnetCreationFailed").to_string(),
        )
    })?;

    Ok(format!("{}/{}", subnet.network(), new_prefix))
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn cidr_subnet_ipv4_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[cidrSubnet('10.144.0.0/20', 24, 0)]", &Context::new())
            .unwrap();

        assert_eq!(result.as_str().unwrap(), "10.144.0.0/24");
    }

    #[test]
    fn cidr_subnet_ipv4_multiple_subnets() {
        let mut parser = Statement::new().unwrap();

        // Test first few subnets of 10.144.0.0/20 split into /24s
        let test_cases = vec![
            (0, "10.144.0.0/24"),
            (1, "10.144.1.0/24"),
            (2, "10.144.2.0/24"),
            (3, "10.144.3.0/24"),
            (4, "10.144.4.0/24"),
            (15, "10.144.15.0/24"),
        ];

        for (index, expected) in test_cases {
            let result = parser
                .parse_and_execute(
                    &format!("[cidrSubnet('10.144.0.0/20', 24, {})]", index),
                    &Context::new(),
                )
                .unwrap();

            assert_eq!(result.as_str().unwrap(), expected);
        }
    }

    #[test]
    fn cidr_subnet_ipv4_larger_subnets() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[cidrSubnet('10.0.0.0/16', 18, 0)]", &Context::new())
            .unwrap();

        assert_eq!(result.as_str().unwrap(), "10.0.0.0/18");

        let result = parser
            .parse_and_execute("[cidrSubnet('10.0.0.0/16', 18, 1)]", &Context::new())
            .unwrap();

        assert_eq!(result.as_str().unwrap(), "10.0.64.0/18");
    }

    #[test]
    fn cidr_subnet_ipv6() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[cidrSubnet('2001:db8::/32', 48, 0)]", &Context::new())
            .unwrap();

        assert_eq!(result.as_str().unwrap(), "2001:db8::/48");

        let result = parser
            .parse_and_execute("[cidrSubnet('2001:db8::/32', 48, 1)]", &Context::new())
            .unwrap();

        assert_eq!(result.as_str().unwrap(), "2001:db8:1::/48");
    }

    #[test]
    fn cidr_subnet_invalid_index() {
        let mut parser = Statement::new().unwrap();
        // 10.144.0.0/20 split into /24s gives 16 subnets (0-15)
        let result =
            parser.parse_and_execute("[cidrSubnet('10.144.0.0/20', 24, 16)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn cidr_subnet_negative_index() {
        let mut parser = Statement::new().unwrap();
        let result =
            parser.parse_and_execute("[cidrSubnet('10.144.0.0/20', 24, -1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn cidr_subnet_new_cidr_too_small() {
        let mut parser = Statement::new().unwrap();
        // New CIDR must be >= current CIDR
        let result =
            parser.parse_and_execute("[cidrSubnet('10.144.0.0/20', 16, 0)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn cidr_subnet_invalid_prefix_v4() {
        let mut parser = Statement::new().unwrap();
        let result =
            parser.parse_and_execute("[cidrSubnet('10.144.0.0/20', 33, 0)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn cidr_subnet_invalid_prefix_v6() {
        let mut parser = Statement::new().unwrap();
        let result =
            parser.parse_and_execute("[cidrSubnet('2001:db8::/32', 129, 0)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn cidr_subnet_same_prefix() {
        let mut parser = Statement::new().unwrap();
        // If new CIDR == current CIDR, only 1 subnet exists (index 0)
        let result = parser
            .parse_and_execute("[cidrSubnet('10.144.0.0/20', 20, 0)]", &Context::new())
            .unwrap();

        assert_eq!(result.as_str().unwrap(), "10.144.0.0/20");
    }
}
