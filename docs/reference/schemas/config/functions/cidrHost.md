---
description: Reference for the 'cidrHost' DSC configuration document function
ms.date:     11/03/2025
ms.topic:    reference
title:       cidrHost
---

# cidrHost

## Synopsis

Calculates a host IP address within a CIDR network block.

## Syntax

```Syntax
cidrHost(<cidrNotation>, <hostNumber>)
```

## Description

The `cidrHost()` function calculates a specific host IP address within a given
[CIDR][01] network block by adding a host number offset to the network address.
This function is particularly useful for systematically assigning IP addresses
to hosts, generating gateway addresses, or allocating IP addresses for network
resources in infrastructure-as-code scenarios.

The host number is zero-indexed, where `0` represents the network address
itself. For typical host assignments, start with `1` to get the first usable
IP address in the network.

## Examples

### Example 1 - Calculate gateway address

Network configurations commonly use the first usable IP address as the gateway.
This example calculates that address using host number `1`.

```yaml
# cidrHost.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Gateway address
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[cidrHost('10.0.1.0/24', 1)]"
```

```bash
dsc config get --file cidrHost.example.1.dsc.config.yaml
```

```yaml
results:
- name: Gateway address
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: 10.0.1.1
messages: []
hadErrors: false
```

### Example 2 - Assign multiple host addresses

This configuration demonstrates calculating host addresses for a subnet created
with [`cidrSubnet()`][02], useful for assigning IP addresses to multiple servers
or network devices.

```yaml
# cidrHost.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  baseNetwork:
    type: string
    defaultValue: 172.16.0.0/16
  subnetIndex:
    type: int
    defaultValue: 10
resources:
  - name: Web server IPs
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        subnet: "[cidrSubnet(parameters('baseNetwork'), 24, parameters('subnetIndex'))]"
        webServer1: "[cidrHost(cidrSubnet(parameters('baseNetwork'), 24, parameters('subnetIndex')), 10)]"
        webServer2: "[cidrHost(cidrSubnet(parameters('baseNetwork'), 24, parameters('subnetIndex')), 11)]"
        webServer3: "[cidrHost(cidrSubnet(parameters('baseNetwork'), 24, parameters('subnetIndex')), 12)]"
```

```bash
dsc config get --file cidrHost.example.2.dsc.config.yaml
```

```yaml
results:
- name: Web server IPs
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        subnet: 172.16.10.0/24
        webServer1: 172.16.10.10
        webServer2: 172.16.10.11
        webServer3: 172.16.10.12
messages: []
hadErrors: false
```

### Example 3 - Generate network device IPs with range

The configuration uses the [`range()`][03] function to generate IP addresses
for multiple network devices systematically, demonstrating how to combine
functions for dynamic IP allocation.

```yaml
# cidrHost.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  networkCidr:
    type: string
    defaultValue: 192.168.100.0/24
  startHost:
    type: int
    defaultValue: 20
  deviceCount:
    type: int
    defaultValue: 5
resources:
  - name: Device IP allocation
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        network: "[parameters('networkCidr')]"
        gateway: "[cidrHost(parameters('networkCidr'), 1)]"
        dnsServer: "[cidrHost(parameters('networkCidr'), 2)]"
        deviceIPs: "[map(range(parameters('startHost'), parameters('deviceCount')), 'i', cidrHost(parameters('networkCidr'), add(i, parameters('startHost'))))]"
```

```bash
dsc config get --file cidrHost.example.3.dsc.config.yaml
```

```yaml
results:
- name: Device IP allocation
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        network: 192.168.100.0/24
        gateway: 192.168.100.1
        dnsServer: 192.168.100.2
        deviceIPs:
        - 192.168.100.20
        - 192.168.100.21
        - 192.168.100.22
        - 192.168.100.23
        - 192.168.100.24
messages: []
hadErrors: false
```

## Parameters

### cidrNotation

The `cidrHost()` function expects the first parameter to be a string in valid
CIDR notation format, including both an IP address and prefix length (e.g.,
`10.0.0.0/16`).

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

### hostNumber

The second parameter specifies the host number offset from the network address.
The value must be a non-negative integer within the valid range of the network.

- For a `/24` network (254 usable hosts), valid values are `0` to `255`
- For a `/16` network (65,534 usable hosts), valid values are `0` to `65535`
- Value `0` returns the network address itself
- Value `1` typically returns the first usable host (often used for gateways)

The function raises an error if the host number exceeds the network capacity.

```yaml
Type:         integer
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `cidrHost()` function returns a string containing the calculated IP address
in standard notation (e.g., `10.0.1.15` for IPv4 or `2001:db8::a` for IPv6).

```yaml
Type: string
```

## Exceptions

The `cidrHost()` function raises errors for the following conditions:

- **Invalid CIDR notation**: When the CIDR string is malformed or missing the
  prefix length
- **Host number out of range**: When the host number exceeds the maximum number
  of addresses in the network
- **Invalid host number**: When the host number is negative

## Related functions

- [`cidrSubnet()`][02] - Creates a subnet from a larger CIDR block
- [`parseCidr()`][04] - Parses CIDR notation and returns network details
- [`range()`][03] - Generates a sequence of numbers
- [`map()`][05] - Applies a function to each element in an array
- [`parameters()`][06] - Retrieves parameter values

<!-- Link reference definitions -->
[01]: https://en.wikipedia.org/wiki/Classless_Inter-Domain_Routing
[02]: ./cidrSubnet.md
[03]: ./range.md
[04]: ./parseCidr.md
[05]: ./map.md
[06]: ./parameters.md
