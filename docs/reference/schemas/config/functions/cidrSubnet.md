---
description: Reference for the 'cidrSubnet' DSC configuration document function
ms.date:     11/03/2025
ms.topic:    reference
title:       cidrSubnet
---

# cidrSubnet

## Synopsis

Creates a subnet CIDR block from a larger network block.

## Syntax

```Syntax
cidrSubnet(<cidrNotation>, <newPrefixLength>, <subnetNumber>)
```

## Description

The `cidrSubnet()` function calculates a subnet [CIDR][01] block from a larger
network block by subdividing it based on a new prefix length and subnet index.
This function is essential for network segmentation, allowing you to
systematically divide a large address space into smaller, manageable subnets
for different purposes like DMZs, application tiers, or regional deployments.

The subnet number is zero-indexed, meaning the first subnet is `0`, the second
is `1`, and so on. The new prefix length must be greater than or equal to the
original prefix to create a valid subnet.

## Examples

### Example 1 - Create multiple subnets from a network block

This configuration divides a `/16` network into multiple `/24` subnets,
demonstrating how to create separate network segments for different purposes.

```yaml
# cidrSubnet.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Network segmentation
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        baseNetwork: 10.0.0.0/16
        webTierSubnet: "[cidrSubnet('10.0.0.0/16', 24, 0)]"
        appTierSubnet: "[cidrSubnet('10.0.0.0/16', 24, 1)]"
        dataTierSubnet: "[cidrSubnet('10.0.0.0/16', 24, 2)]"
        managementSubnet: "[cidrSubnet('10.0.0.0/16', 24, 3)]"
```

```bash
dsc config get --file cidrSubnet.example.1.dsc.config.yaml
```

```yaml
results:
- name: Network segmentation
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        baseNetwork: 10.0.0.0/16
        webTierSubnet: 10.0.0.0/24
        appTierSubnet: 10.0.1.0/24
        dataTierSubnet: 10.0.2.0/24
        managementSubnet: 10.0.3.0/24
messages: []
hadErrors: false
```

### Example 2 - Create subnets for multiple regions

This configuration demonstrates creating dedicated subnets for different regions
or environments, showing how to systematically allocate non-overlapping network
segments from a larger address space.

```yaml
# cidrSubnet.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  baseNetwork:
    type: string
    defaultValue: 10.144.0.0/20
  newPrefix:
    type: int
    defaultValue: 24
resources:
  - name: Regional subnets
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        eastus: "[cidrSubnet(parameters('baseNetwork'), parameters('newPrefix'), 0)]"
        westus: "[cidrSubnet(parameters('baseNetwork'), parameters('newPrefix'), 1)]"
        northeurope: "[cidrSubnet(parameters('baseNetwork'), parameters('newPrefix'), 2)]"
        westeurope: "[cidrSubnet(parameters('baseNetwork'), parameters('newPrefix'), 3)]"
        southeastasia: "[cidrSubnet(parameters('baseNetwork'), parameters('newPrefix'), 4)]"
```

```bash
dsc config get --file cidrSubnet.example.2.dsc.config.yaml
```

```yaml
results:
- name: Regional subnets
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        eastus: 10.144.0.0/24
        westus: 10.144.1.0/24
        northeurope: 10.144.2.0/24
        westeurope: 10.144.3.0/24
        southeastasia: 10.144.4.0/24
messages: []
hadErrors: false
```

### Example 3 - Nested subnetting with host allocation

This example demonstrates combining `cidrSubnet()` with [`cidrHost()`][03]
and [`parseCidr()`][04] to create a complete network configuration including
subnets and host IP assignments.

```yaml
# cidrSubnet.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  vnetCidr:
    type: string
    defaultValue: 172.16.0.0/12
  subnetIndex:
    type: int
    defaultValue: 42
resources:
  - name: Complete network configuration
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        vnetAddressSpace: "[parameters('vnetCidr')]"
        subnetCidr: "[cidrSubnet(parameters('vnetCidr'), 24, parameters('subnetIndex'))]"
        subnetDetails: "[parseCidr(cidrSubnet(parameters('vnetCidr'), 24, parameters('subnetIndex')))]"
        gatewayIP: "[cidrHost(cidrSubnet(parameters('vnetCidr'), 24, parameters('subnetIndex')), 1)]"
        loadBalancerIP: "[cidrHost(cidrSubnet(parameters('vnetCidr'), 24, parameters('subnetIndex')), 4)]"
```

```bash
dsc config get --file cidrSubnet.example.3.dsc.config.yaml
```

```yaml
results:
- name: Complete network configuration
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        vnetAddressSpace: 172.16.0.0/12
        subnetCidr: 172.16.42.0/24
        subnetDetails:
          network: 172.16.42.0
          netmask: 255.255.255.0
          broadcast: 172.16.42.255
          firstUsable: 172.16.42.1
          lastUsable: 172.16.42.254
          cidr: 24
        gatewayIP: 172.16.42.1
        loadBalancerIP: 172.16.42.4
messages: []
hadErrors: false
```

### Example 4 - IPv6 subnet allocation

This configuration demonstrates creating IPv6 subnets from a larger IPv6 address
block, showing support for both IPv4 and IPv6 address families.

```yaml
# cidrSubnet.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  ipv6BaseNetwork:
    type: string
    defaultValue: 2001:db8::/32
  subnetPrefix:
    type: int
    defaultValue: 48
resources:
  - name: IPv6 subnets
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        baseNetwork: "[parameters('ipv6BaseNetwork')]"
        subnet0: "[cidrSubnet(parameters('ipv6BaseNetwork'), parameters('subnetPrefix'), 0)]"
        subnet1: "[cidrSubnet(parameters('ipv6BaseNetwork'), parameters('subnetPrefix'), 1)]"
        subnet10: "[cidrSubnet(parameters('ipv6BaseNetwork'), parameters('subnetPrefix'), 10)]"
```

```bash
dsc config get --file cidrSubnet.example.4.dsc.config.yaml
```

```yaml
results:
- name: IPv6 subnets
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        baseNetwork: 2001:db8::/32
        subnet0: 2001:db8::/48
        subnet1: 2001:db8:1::/48
        subnet10: 2001:db8:a::/48
messages: []
hadErrors: false
```

## Parameters

### cidrNotation

The first parameter specifies the base network in CIDR notation from which
subnets will be created. This must be a valid CIDR string including both an
IP address and prefix length (e.g., `10.0.0.0/16`).

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

### newPrefixLength

The second parameter specifies the prefix length for the new subnet. This value
must be greater than or equal to the base network's prefix length.

For example:

- To divide a `/16` into `/24` subnets, use `24` (creates 256 subnets)
- To divide a `/20` into `/24` subnets, use `24` (creates 16 subnets)
- To divide a `/8` into `/16` subnets, use `16` (creates 256 subnets)

The function raises an error if the new prefix length is smaller than the
original, as this would create a larger network rather than a subnet.

```yaml
Type:         integer
Required:     true
MinimumCount: 1
MaximumCount: 1
```

### subnetNumber

The third parameter specifies which subnet to calculate, using zero-based
indexing. The valid range depends on how many subnets the prefix length
difference allows.

For example, dividing a `/16` into `/24` subnets allows subnet numbers from
`0` to `255` (2^(24-16) = 256 subnets).

The function raises an error if the subnet number exceeds the maximum number
of subnets available in the base network.

```yaml
Type:         integer
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `cidrSubnet()` function returns a string containing the calculated subnet
in CIDR notation (e.g., `10.0.5.0/24`).

```yaml
Type: string
```

## Exceptions

The `cidrSubnet()` function raises errors for the following conditions:

- **Invalid CIDR notation**: When the base CIDR string is malformed or missing
  the prefix length
- **Invalid prefix length**: When the new prefix is smaller than the base
  network's prefix
- **Subnet number out of range**: When the subnet number exceeds the maximum
  number of subnets possible with the given prefix lengths
- **Invalid subnet number**: When the subnet number is negative

## Related functions

- [`cidrHost()`][02] - Calculates a host IP address within a CIDR block
- [`parseCidr()`][03] - Parses CIDR notation and returns network details
- [`parameters()`][04] - Retrieves parameter values

<!-- Link reference definitions -->
[01]: https://en.wikipedia.org/wiki/Classless_Inter-Domain_Routing
[02]: ./cidrHost.md
[03]: ./parseCidr.md
[04]: ./parameters.md
