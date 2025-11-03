---
description: Reference for the 'parseCidr' DSC configuration document function
ms.date:     11/03/2025
ms.topic:    reference
title:       parseCidr
---

# parseCidr

## Synopsis

Parses a CIDR notation string and returns network information.

## Syntax

```Syntax
parseCidr(<cidrNotation>)
```

## Description

The `parseCidr()` function takes a [CIDR][01] (Classless Inter-Domain Routing)
notation string and returns an object containing detailed network information
including the network address, netmask, broadcast address, and usable IP range.
This function is useful for calculating network details when configuring
networking resources, firewall rules, or IP address management systems.

The function supports both IPv4 and IPv6 CIDR notation and always requires
explicit prefix length (e.g., `/24` for IPv4 or `/64` for IPv6).

## Examples

### Example 1 - Parse standard IPv4 CIDR

This configuration parses a typical `/24` network and displays the network
details including usable IP range.

```yaml
# parseCidr.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Parse IPv4 network
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[parseCidr('192.168.1.0/24')]"
```

```bash
dsc config get --file parseCidr.example.1.dsc.config.yaml
```

```yaml
results:
- name: Parse IPv4 network
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        network: 192.168.1.0
        netmask: 255.255.255.0
        broadcast: 192.168.1.255
        firstUsable: 192.168.1.1
        lastUsable: 192.168.1.254
        cidr: 24
messages: []
hadErrors: false
```

### Example 2 - Calculate subnet details with parameters

This example demonstrates using `parseCidr()` with the [`cidrSubnet()`][02]
function to create a subnet from a larger network block and extract its details.

```yaml
# parseCidr.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  baseNetwork:
    type: string
    defaultValue: 10.0.0.0/16
  subnetPrefix:
    type: int
    defaultValue: 24
  subnetIndex:
    type: int
    defaultValue: 5
resources:
  - name: Calculate subnet details
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[parseCidr(cidrSubnet(parameters('baseNetwork'), parameters('subnetPrefix'), parameters('subnetIndex')))]"
```

```bash
dsc config get --file parseCidr.example.2.dsc.config.yaml
```

```yaml
results:
- name: Calculate subnet details
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        network: 10.0.5.0
        netmask: 255.255.255.0
        broadcast: 10.0.5.255
        firstUsable: 10.0.5.1
        lastUsable: 10.0.5.254
        cidr: 24
messages: []
hadErrors: false
```

### Example 3 - Extract specific network properties

The configuration extracts specific properties from the parsed CIDR result to
configure network settings, demonstrating how to access individual fields from
the returned object.

```yaml
# parseCidr.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  networkCidr:
    type: string
    defaultValue: 10.144.0.0/20
resources:
  - name: Network configuration
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        networkAddress: "[parseCidr(parameters('networkCidr')).network]"
        subnetMask: "[parseCidr(parameters('networkCidr')).netmask]"
        gatewayIP: "[parseCidr(parameters('networkCidr')).firstUsable]"
        broadcastIP: "[parseCidr(parameters('networkCidr')).broadcast]"
        prefixLength: "[parseCidr(parameters('networkCidr')).cidr]"
```

```bash
dsc config get --file parseCidr.example.3.dsc.config.yaml
```

```yaml
results:
- name: Network configuration
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        networkAddress: 10.144.0.0
        subnetMask: 255.255.240.0
        gatewayIP: 10.144.0.1
        broadcastIP: 10.144.15.255
        prefixLength: 20
messages: []
hadErrors: false
```

## Parameters

### cidrNotation

The `parseCidr()` function expects a single string in valid CIDR notation
format. The string must include both an IP address and a prefix length
separated by a forward slash (e.g., `192.168.1.0/24` or `2001:db8::/32`).

The function raises an error if:

- The input doesn't contain a forward slash (`/`)
- The IP address format is invalid
- The prefix length is out of valid range (0-32 for IPv4, 0-128 for IPv6)

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `parseCidr()` function returns an object with the following properties:

For **IPv4** addresses:

- `network`: The network address (string)
- `netmask`: The subnet mask in dotted decimal notation (string)
- `broadcast`: The broadcast address (string)
- `firstUsable`: The first usable host IP address (string)
- `lastUsable`: The last usable host IP address (string)
- `cidr`: The prefix length (integer)

For **IPv6** addresses:

- `network`: The network address (string)
- `netmask`: The network mask (string)
- `broadcast`: The broadcast address (string)
- `firstUsable`: The first usable address (same as network for IPv6) (string)
- `lastUsable`: The last usable address (same as broadcast for IPv6) (string)
- `cidr`: The prefix length (integer)

**Note**: For `/32` IPv4 networks (single host), `firstUsable` and `lastUsable`
are both set to the network address since there are no additional host addresses.

```yaml
Type: object
```

## Exceptions

The `parseCidr()` function raises errors for the following conditions:

- **Missing prefix**: When the CIDR string doesn't include a prefix length
  (e.g., `192.168.1.0` without `/24`)
- **Invalid IP address**: When the IP address portion is malformed
- **Invalid prefix length**: When the prefix is out of valid range or not a number

## Related functions

- [`cidrSubnet()`][02] - Creates a subnet from a larger CIDR block
- [`cidrHost()`][03] - Calculates a host IP address within a CIDR block
- [`parameters()`][04] - Retrieves parameter values

<!-- Link reference definitions -->
[01]: https://en.wikipedia.org/wiki/Classless_Inter-Domain_Routing
[02]: ./cidrSubnet.md
[03]: ./cidrHost.md
[04]: ./parameters.md
