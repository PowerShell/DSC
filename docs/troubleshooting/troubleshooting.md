---
description: >-
    Learn how to troubleshoot common issues and known issues with Microsoft Desired State Configuration (DSC).
ms.date: 06/29/2025
ms.topic: troubleshooting
title:  Troubleshooting Microsoft Desired State Configuration
---

# Troubleshooting Microsoft Desired State Configuration

This article provides troubleshooting instructions for common errors and list out known issues.

## Known issues

The following table lists known issues with Microsoft DSC v3:

| Issue                                                                          | Description                                                                     | Status    | Reported on                                          |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------- | --------- | ---------------------------------------------------- |
| [Unable to parse content from '<manifestUrl>'](#unable-to-parse-content-from-) | When authoring a resource manifest in VSCode, you may encounter parsing errors. | Confirmed | [#917](https://github.com/PowerShell/DSC/issues/917) |

For the most up-to-date information on known issues, visit the [DSC GitHub repository issues page](https://github.com/PowerShell/DSC/issues).

### Unable to parse content from '<manifestUrl>'

When authoring a resource manifest in Visual Studio Code (VSCode), you may encouther the following issue:

<!-- markdownlint-disable MD013 -->
:::image type="complex" source="media/known-issues/unable-to-parse-content.png" alt-text="This screenshot shows the parse content error.":::
   This screenshot shows the unable to parse content error. The error occurs because canonical schema bundling is still not fully supported in the 2020-12 JSON schema.
:::image-end:::
<!-- markdownlint-restore -->

This issue applies for Microsoft DSC v3.0 and above.

**Resolution:** This issue can be resolved by using the `manifest.vscode.json` in the schema URI. For more information, check out the [enhanced authoring][00] page.

## See Also

- [Microsoft Desired State Configuration overview](../overview.md)

<!-- link references -->
[00]: https://learn.microsoft.com/en-us/powershell/dsc/concepts/enhanced-authoring?view=dsc-3.0
