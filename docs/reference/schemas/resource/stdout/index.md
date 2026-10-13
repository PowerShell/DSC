---
description: Overview for the JSON Schemas defining expected stdout for DSC Resource operations.
ms.date:     08/21/2025
ms.topic:    reference
title:       Overview of DSC resource operation stdout schemas
---

# Overview of DSC resource operation stdout schemas

DSC is designed around strong contracts and data models to define how resources integrate into DSC
and the wider ecosystem. As part of this contract, DSC defines JSON Schemas for the expected text
resources emit to stdout for each resource operation.

The following schemas describe the expected output for each operation and how DSC validates the
data a resource emits:

- [DSC resource delete operation stdout schema reference][01]
- [DSC resource export operation stdout schema reference][02]
- [DSC resource get operation stdout schema reference][03]
- [DSC resource list operation stdout schema reference][04]
- [DSC resource resolve operation stdout schema reference][05]
- [DSC resource schema operation stdout schema reference][06]
- [DSC resource test operation stdout schema reference][07]
- [DSC resource validate operation stdout schema reference][08]
- [DSC resource what-if operation stdout schema reference][09]

<!-- Link reference definitions -->
[01]: ./delete.md
[02]: ./export.md
[03]: ./get.md
[04]: ./list.md
[05]: ./resolve.md
[06]: ./schema.md
[07]: ./test.md
[08]: ./validate.md
[09]: ./whatIf.md
