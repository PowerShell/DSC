# Microsoft.Windows/Updates DSC Resource

## Overview

The `Microsoft.Windows/Updates` resource enables querying information about Windows Updates using the Windows Update Agent COM APIs. This resource allows you to retrieve detailed information about specific updates available on or installed on a Windows system.

## Features

- Query Windows Update information by title
- Retrieve comprehensive update details including:
  - Installation status
  - Update description
  - Unique update identifier
  - KB article IDs
  - Download size
  - Security severity rating
  - Security bulletin IDs
  - Update type (Software or Driver)

## Requirements

- Windows operating system
- Windows Update Agent (built into Windows)
- Administrator privileges may be required for certain update queries

## Usage

### Get Operation

The `get` operation searches for a Windows Update by title (supports partial matching) and returns detailed information about the update.

#### Input Schema

```json
{
  "title": "Security Update"
}
```

#### Example DSC Configuration

```yaml
# windows-update-query.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/configuration.json
resources:
- name: QuerySecurityUpdate
  type: Microsoft.Windows/Updates
  properties:
    title: "Security Update for Windows"
```

#### Output Example

```json
{
  "title": "2024-01 Security Update for Windows 11 Version 22H2 for x64-based Systems (KB5034123)",
  "isInstalled": true,
  "description": "Install this update to resolve issues in Windows...",
  "id": "12345678-1234-1234-1234-123456789abc",
  "isUninstallable": true,
  "KBArticleIDs": ["5034123"],
  "maxDownloadSize": 524288000,
  "msrcSeverity": "Critical",
  "securityBulletinIds": ["MS24-001"],
  "updateType": "Software"
}
```

## Properties

### Input Properties

| Property | Type   | Required | Description                                    |
|----------|--------|----------|------------------------------------------------|
| title    | string | Yes      | The title or partial title of the update to search for |

### Output Properties

| Property              | Type            | Description                                           |
|-----------------------|-----------------|-------------------------------------------------------|
| title                 | string          | The full title of the Windows Update                  |
| isInstalled           | boolean         | Whether the update is currently installed             |
| description           | string          | Detailed description of the update                    |
| id                    | string          | Unique identifier (GUID) for the update               |
| isUninstallable       | boolean         | Whether the update can be uninstalled                 |
| KBArticleIDs          | array[string]   | Knowledge Base article identifiers                    |
| maxDownloadSize       | integer (int64) | Maximum download size in bytes                        |
| msrcSeverity          | enum            | MSRC severity: Critical, Important, Moderate, or Low  |
| securityBulletinIds   | array[string]   | Security bulletin identifiers                         |
| updateType            | enum            | Type of update: Software or Driver                    |

## Implementation Details

- **Language**: Rust
- **Executable**: `wu_dsc`
- **COM APIs Used**: Windows Update Agent (WUA) COM interfaces
  - `IUpdateSession`
  - `IUpdateSearcher`
  - `IUpdateCollection`
  - `IUpdate`

## Limitations

- Only the `get` operation is currently implemented
- The `set` and `test` operations are not supported (updates should be managed through Windows Update settings)
- Requires Windows operating system
- Search is case-insensitive and matches partial titles

## Building

To build the resource:

```powershell
cd resources/WindowsUpdate
cargo build --release
```

The compiled executable will be located at `target/release/wu_dsc.exe`.

## Testing

To test the resource manually:

```powershell
# Create input JSON
$input = @{ title = "Security Update" } | ConvertTo-Json

# Query for an update
$input | .\wu_dsc.exe get
```

## Error Handling

The resource will return an error if:
- No update matching the specified title is found
- COM initialization fails
- The Windows Update service is unavailable
- Invalid input is provided

## License

Copyright (c) Microsoft Corporation.
Licensed under the MIT License.
