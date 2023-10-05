# File-Content Resource

## Overview

The file-content resource is used to manage file content on Linux, macOS, and Windows systems.

## Examples

### Example 1

This snippet will asserts that the file with the specified SHA256 hash is present at the given path.

```bash
$ _EXAMPLE_CONTENT="My example content."
$ cat << END | dsc resource get -r file-content
path: /path/to/file
hash:
  algorithm: SHA256
  checksum: $(echo ${_EXAMPLE_CONTENT} | sha256sum | awk '{print $1;}')
content: ${_EXAMPLE_CONTENT}
END
```

**Output:**
  
```yaml
actualState:
  path: /path/to/file
  hash:
    algorithm: SHA256
    checksum: 7ffcdecfc6d97bd4dd419e1c17caf988a8ebd94249d72f69db7e7040c9924314
  content: My example content.
```

### Example 2

This snippet will create a file with at the given path, with the given content. The source content
in this example is using YAML [**folded** style](https://yaml.org/spec/1.2.2/#81-block-scalar-styles)
multiline syntax. Since the content is under 80 characters, the output will not use block style for
the, but it does use it for the checksum.

```bash
$ cat << END | dsc resource set -r file-content
path: /path/to/file
content: >
  folded style,
  multiline,
  no hard breaks
END
```

**Output:**
  
```yaml
beforeState:
  _exist: false
afterState:
  _exist: true
  path: /path/to/file
  hash:
    algorithm: SHA512
    checksum: >
      9481b193486b1d5ccb9534de916f45f2b242cbd284caf89b6d1e524bc60b23a91d56a7c83b45a94f45dd990b080021513904791326e41be8367f247398c296f3
  content: folded style, multiline, no hard breaks
```

### Example 3

This snippet will set the content of the file, with the given content if the checksum does not match.
However, in this case the existing file already has the correct content, so the resource will not
change the file.

```bash
$ _EXAMPLE_CONTENT="My example content."
$ cat << END | dsc resource get -r file-content
path: /path/to/file
hash:
  algorithm: SHA1
  checksum: $(echo ${_EXAMPLE_CONTENT} | sha1sum | awk '{print $1;}')
content: ${_EXAMPLE_CONTENT}
END
```

**Output:**
  
```yaml
beforeState:
  path: /path/to/file
  hash:
    algorithm: SHA1
    checksum: 9f78aea13fe04534523acf8ba28f866c373f7442
  content: My example content.
```

### Example 4

This snippet will set the content of the given file. The content in this example is using
YAML [**literal** style](https://yaml.org/spec/1.2.2/#81-block-scalar-styles) multiline syntax.

**Warning:** This operation will clobber the existing content.

The output style used for long lines and multilines depends on the content:

- If the file contains hard breaks, then the output will always use,
  YAML [**literal** style](https://yaml.org/spec/1.2.2/#81-block-scalar-styles) (`|`) multiline
  syntax, with the **keep** option (`+`).
- If the file does not contain hard breaks:
  - If the actual content is 80 characters long or less, then the output will not use block style.
  - When the content is greater than 80 characters, the **folded** style (`>`) is used and **clip**
    is enabled. As much as possible lines will wrap around 80 characters.

```bash
$ cat << END | dsc resource set -r file-content
path: /path/to/file
hash:
  algorithm: SHA256
content: |
  Include /etc/ssh/sshd_config.d/*.conf
  PasswordAuthentication no
  ChallengeResponseAuthentication no
  UsePAM yes
  X11Forwarding yes
  PrintMotd no
  AcceptEnv LANG LC_*
  Subsystem       sftp    /usr/lib/openssh/sftp-server
END
```

**Output:**
  
```yaml
beforeState:
  path: /path/to/file
  hash:
    algorithm: SHA256
    checksum: dfae581667dc3e3fda151b088557e96fe7331d0a8d3b927f8cd72bcd26487060
  content: |+
    Include /etc/ssh/sshd_config.d/*.conf
    PasswordAuthentication no
    ChallengeResponseAuthentication no
    UsePAM no
    X11Forwarding yes
    PrintMotd no
    AcceptEnv LANG LC_*
    Subsystem       sftp    /usr/lib/openssh/sftp-server
afterState:
  path: /path/to/file
  hash:
    algorithm: SHA256
    checksum: 1e18031fff4d0440bbc83637de412d9ec20f0ad5f2ae27ca47f9405d37646297
  content: |+
    Include /etc/ssh/sshd_config.d/*.conf
    PasswordAuthentication no
    ChallengeResponseAuthentication no
    UsePAM yes
    X11Forwarding yes
    PrintMotd no
    AcceptEnv LANG LC_*
    Subsystem       sftp    /usr/lib/openssh/sftp-server
```

### Example 5

When using YAML, hard line breaks are always preserved using a single **linefeed** (`/n`) character.
In order to use a specific line break sequence, use the escape codes. For example, for CR+LF, use
the escape sequence for Carriage Return (`\r`) + **linefeed** (`\n`). When doing this for multiline
content, it is best to use the **folded** style multiline syntax with the **clip** indicator.

This example also shows the default hash algorithm is SHA-512, and uses the JSON pretty-print format.

```bash
$ cat << END | dsc resource set -r file-content --format jsonPretty
path: /path/to/file
content: >-
  In this content,/r/nhard-breaks have been
  embedded/r/nwith escape sequences, and
  folding is used to/r/nlimit code line
  length,/r/nnot the embedded content.
END
```

**Output:**
  
```json
{
  "path": "/path/to/file",
  "hash": {
    "algorithm": "SHA512",
    "checksum": "61ac34d5f3f00137c57f41f616b1e96036dc011dbe81050e3d7c6dc2332ffe99330a7a4eadc60787e96401bf60ae8894fcf3a71330ae72f289481c277deb8378"
  },
  "content": "In this content,/r/nhard-breaks have been embedded/r/nwith escape sequences, and folding is used to/r/nlimit code line length,/r/nnot the embedded content."
}
```

### Example 6

This snippet shows how to use the `eol` property to set the line ending sequence. The default is to
end line sequences with the **linefeed** (`\n`) character.

The `eol` property provides an alternative to embedding escape sequences in the content. Any embedded
escape sequences in the content are always preserved. The valid values are `LF`, `CRLF`, and `CR`.

Because YAML always uses linefeed (`\n`) as the line ending sequence, if preserving the line ending
sequence in the output is important, it is recommended to use the JSON output format.

```bash
$ cat << END | dsc resource set -r file-content --format jsonPretty
path: "/path/to/file"
hash:
  algorithm: SHA1
  checksum: cc4434a620547d4a6b9498c6293415b293d8c036
content: |
  In this example, hard line breaks
  will use the specified end-of-line
  sequence.
eol: CRLF
END
```

**Output:**
  
```json
{"path":"/path/to/file","hash":{"algorithm":"SHA1","checksum":"cc4434a620547d4a6b9498c6293415b293d8c036"},"content":"In this example, hard line breaks\r\nwill use the specified end-of-line\r\nsequence."}
```

### Example 7

This snippet shows how the checksum can be used to verify a file without providing any content. In
this example, the file exists, but the checksum does not match the desired state.

```bash
$ cat << END | dsc resource get -r file-content
_exist: true
path: /path/to/file
hash:
  algorithm: SHA256
  checksum: 7ffcdecfc6d97bd4dd419e1c17caf988a8ebd94249d72f69db7e7040c9924314
END
```

**Output:**
  
```yaml
actualState:
_exist: true
  path: /path/to/file
  hash:
        algorithm: SHA256
    checksum: dfae581667dc3e3fda151b088557e96fe7331d0a8d3b927f8cd72bcd26487060
```

[//]: # (cSpell:ignore multilines, nhard, nlimit, nnot, nwith,)
