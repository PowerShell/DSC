// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Function;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct CidrHost {}

impl Function for CidrHost {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "cidrHost".to_string(),
            description: t!("functions.cidrHost.description").to_string(),
            category: vec![FunctionCategory::Cidr],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
                vec![FunctionArgKind::Number],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.cidrHost.invoked"));

        let cidr_string = args[0].as_str().unwrap();
        let host_index = args[1].as_i64().unwrap();

        if host_index < 0 {
            return Err(DscError::FunctionArg(
                "cidrHost".to_string(),
                t!("functions.cidrHost.negativeHostIndex").to_string(),
            ));
        }

        let network = cidr_string.parse::<IpNetwork>().map_err(|_| {
            DscError::FunctionArg(
                "cidrHost".to_string(),
                t!("functions.cidrHost.invalidCidr", cidr = cidr_string).to_string(),
            )
        })?;

        let result = match network {
            IpNetwork::V4(net) => calculate_ipv4_host(&net, host_index as u32)?,
            IpNetwork::V6(net) => calculate_ipv6_host(&net, host_index as u128)?,
        };

        Ok(Value::String(result))
    }
}

fn calculate_ipv4_host(net: &Ipv4Network, host_index: u32) -> Result<String, DscError> {
    let prefix = net.prefix();

    // Special case: /32 has no usable hosts
    if prefix == 32 {
        return Err(DscError::FunctionArg(
            "cidrHost".to_string(),
            t!("functions.cidrHost.noUsableHosts").to_string(),
        ));
    }

    // Special case: /31 (point-to-point) has 2 usable hosts (both IPs are usable)
    if prefix == 31 {
        let host_ip = net.nth(host_index).ok_or_else(|| {
            DscError::FunctionArg(
                "cidrHost".to_string(),
                t!(
                    "functions.cidrHost.hostIndexOutOfRange",
                    index = host_index,
                    maxIndex = 1
                )
                .to_string(),
            )
        })?;
        return Ok(host_ip.to_string());
    }

    // Regular case: skip network address (0) and broadcast (last)
    // Usable hosts are at positions 1 to (size - 2)
    let size = net.size();
    let max_usable_index = size.saturating_sub(2); // size - 2 (network and broadcast)

    if host_index >= max_usable_index {
        return Err(DscError::FunctionArg(
            "cidrHost".to_string(),
            t!(
                "functions.cidrHost.hostIndexOutOfRange",
                index = host_index,
                maxIndex = max_usable_index.saturating_sub(1)
            )
            .to_string(),
        ));
    }

    // Get the host at position (host_index + 1) to skip network address
    let host_ip = net.nth(host_index + 1).ok_or_else(|| {
        DscError::FunctionArg(
            "cidrHost".to_string(),
            t!("functions.cidrHost.hostCalculationFailed").to_string(),
        )
    })?;

    Ok(host_ip.to_string())
}

fn calculate_ipv6_host(net: &Ipv6Network, host_index: u128) -> Result<String, DscError> {
    let prefix = net.prefix();

    if prefix == 128 {
        return Err(DscError::FunctionArg(
            "cidrHost".to_string(),
            t!("functions.cidrHost.noUsableHosts").to_string(),
        ));
    }

    // /127 is a special case where both addresses are usable
    if prefix == 127 {
        if host_index > 1 {
            return Err(DscError::FunctionArg(
                "cidrHost".to_string(),
                t!(
                    "functions.cidrHost.hostIndexOutOfRange",
                    index = host_index,
                    maxIndex = 1
                )
                .to_string(),
            ));
        }

        // For IPv6 /127, both addresses are usable
        let host_ip = net.iter().nth(host_index as usize).ok_or_else(|| {
            DscError::FunctionArg(
                "cidrHost".to_string(),
                t!("functions.cidrHost.hostCalculationFailed").to_string(),
            )
        })?;
        return Ok(host_ip.to_string());
    }

    // For IPv6, typically the network address (first) is not used for hosts
    // but the broadcast concept doesn't apply. However, following the pattern:
    // Skip the first address (network identifier)
    // The last address in the subnet is technically usable unlike IPv4

    // Check bounds - we need to be careful with large IPv6 networks
    // For practical purposes, limit the index check
    let host_bits = 128 - prefix;

    // If the network is very large (more than 32 host bits), we can't easily check bounds
    if host_bits > 32 {
        let actual_index = (host_index as usize).saturating_add(1);
        let host_ip = net.iter().nth(actual_index).ok_or_else(|| {
            DscError::FunctionArg(
                "cidrHost".to_string(),
                t!(
                    "functions.cidrHost.hostIndexOutOfRange",
                    index = host_index,
                    maxIndex = "unknown"
                )
                .to_string(),
            )
        })?;
        return Ok(host_ip.to_string());
    }

    let size = 2_u128.pow(host_bits as u32);
    let max_usable_index = size.saturating_sub(1); // Skip network address

    if host_index >= max_usable_index {
        return Err(DscError::FunctionArg(
            "cidrHost".to_string(),
            t!(
                "functions.cidrHost.hostIndexOutOfRange",
                index = host_index,
                maxIndex = max_usable_index.saturating_sub(1)
            )
            .to_string(),
        ));
    }

    let actual_index = (host_index as usize).saturating_add(1);
    let host_ip = net.iter().nth(actual_index).ok_or_else(|| {
        DscError::FunctionArg(
            "cidrHost".to_string(),
            t!("functions.cidrHost.hostCalculationFailed").to_string(),
        )
    })?;

    Ok(host_ip.to_string())
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn cidr_host_ipv4_basic() {
        let mut parser = Statement::new().unwrap();

        let result = parser
            .parse_and_execute("[cidrHost('192.168.1.0/24', 0)]", &Context::new())
            .unwrap();
        assert_eq!(result.as_str().unwrap(), "192.168.1.1");

        let result = parser
            .parse_and_execute("[cidrHost('192.168.1.0/24', 1)]", &Context::new())
            .unwrap();
        assert_eq!(result.as_str().unwrap(), "192.168.1.2");

        let result = parser
            .parse_and_execute("[cidrHost('192.168.1.0/24', 253)]", &Context::new())
            .unwrap();
        assert_eq!(result.as_str().unwrap(), "192.168.1.254");
    }

    #[test]
    fn cidr_host_ipv4_larger_network() {
        let mut parser = Statement::new().unwrap();

        let result = parser
            .parse_and_execute("[cidrHost('10.0.0.0/16', 0)]", &Context::new())
            .unwrap();
        assert_eq!(result.as_str().unwrap(), "10.0.0.1");

        let result = parser
            .parse_and_execute("[cidrHost('10.0.0.0/16', 99)]", &Context::new())
            .unwrap();
        assert_eq!(result.as_str().unwrap(), "10.0.0.100");
    }

    #[test]
    fn cidr_host_ipv4_slash_31() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[cidrHost('192.168.1.0/31', 0)]", &Context::new())
            .unwrap();
        assert_eq!(result.as_str().unwrap(), "192.168.1.0");

        let result = parser
            .parse_and_execute("[cidrHost('192.168.1.0/31', 1)]", &Context::new())
            .unwrap();
        assert_eq!(result.as_str().unwrap(), "192.168.1.1");
    }

    #[test]
    fn cidr_host_ipv4_slash_32_no_hosts() {
        let mut parser = Statement::new().unwrap();

        let result = parser.parse_and_execute("[cidrHost('192.168.1.1/32', 0)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn cidr_host_ipv4_index_out_of_range() {
        let mut parser = Statement::new().unwrap();

        let result = parser.parse_and_execute("[cidrHost('192.168.1.0/24', 254)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn cidr_host_ipv4_negative_index() {
        let mut parser = Statement::new().unwrap();

        let result = parser.parse_and_execute("[cidrHost('192.168.1.0/24', -1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn cidr_host_ipv6_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[cidrHost('2001:db8::/64', 0)]", &Context::new())
            .unwrap();
        assert_eq!(result.as_str().unwrap(), "2001:db8::1");
    }

    #[test]
    fn cidr_host_ipv6_slash_127() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[cidrHost('2001:db8::1:0/127', 0)]", &Context::new())
            .unwrap();
        assert_eq!(result.as_str().unwrap(), "2001:db8::1:0");

        let result = parser
            .parse_and_execute("[cidrHost('2001:db8::1:0/127', 1)]", &Context::new())
            .unwrap();
        assert_eq!(result.as_str().unwrap(), "2001:db8::1:1");
    }

    #[test]
    fn cidr_host_ipv6_slash_128_no_hosts() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[cidrHost('2001:db8::1/128', 0)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn cidr_host_invalid_cidr() {
        let mut parser = Statement::new().unwrap();

        let result = parser.parse_and_execute("[cidrHost('invalid', 0)]", &Context::new());
        assert!(result.is_err());
    }
}
