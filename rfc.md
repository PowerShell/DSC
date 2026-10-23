---
RFC:          RFCNNNN
Author:       @Gijsreyn
Sponsor:      @Gijsreyn
Status:       Draft
SupercededBy: null
Version:      1.0
Area:         CLI
CommentsDue:  2026-11-20
---

# Static linting for configuration documents with `dsc lint`

This RFC proposes a `dsc lint` command that checks DSC configuration documents written in YAML or
JSON for problems without applying them. The command parses the document, validates it against
the configuration document schema, and runs a catalog of rules that find mistakes DSC only reports
at runtime today, or doesn't report at all. It reports each problem as a diagnostic with a level,
a rule identifier, a message, and a JSON pointer to the location in the document, so authors,
editors, and CI pipelines can act on the result.

The linting engine lives in `dsc-lib` so other hosts can reuse it. The first phase covers
configuration documents. A second phase extends the same command to resource and extension
manifests, as suggested in the discussion of [#591][01].

## Motivation

> As a configuration author,
> I want DSC to tell me about mistakes in my document before I apply it,
> so that I don't discover a typo in a function name or a missing dependency halfway through a
> `dsc config set` run.

> As a platform engineer who keeps configuration documents in source control,
> I want a command that fails a pull request on errors and emits machine-readable results,
> so that broken documents never reach the machines they target.

> As a resource author,
> I want the same command to check my resource manifest,
> so that I find an inconsistent manifest before publishing it.

Today DSC only validates a document when it runs it. `dsc config validate` exists but is hidden,
is only used by the Group, Assertion, and Include resources, reports a single `valid` boolean
without a reason, exits with code `0` even when the document is invalid, and always discovers
resources before it validates anything. Problems such as a `dependsOn` entry that names a
resource that doesn't exist, a `reference()` to a resource that hasn't run yet, a parameter that
is declared but never used, a secure parameter with a hard-coded default, or a condition written
as a plain string are either reported late, with no location, or not at all.

> [!NOTE]
> I think earlier discussions revealed that eventually `dsc config validate` will be deprecated (question mark).

[#591][01] proposed a `dsc lint` command with one rule: when the operating system root is
overridden (`--system-root`, formerly `--target-path`), the document should use the `systemRoot()`
function. The discussion asked whether linting belongs in an extension or in the core `dsc`
executable, noted that a linter for both configurations and manifests would be valuable, and
observed that a YAML and JSON authoring community needs good linting regardless of whether Bicep
gains DSC-specific rules. [#842][02] asked `dsc config validate` to reject duplicate resource names
and suggested growing that command into static analysis.

This proposal answers those points as follows:

- Linting is part of the core `dsc` executable and of `dsc-lib`. The rules need the document
  schema, the expression parser, the function registry, resource discovery, and the dependency
  resolution logic, all of which live in `dsc-lib`. Putting the engine in the library lets the
  VS Code extension, the MCP server, and other hosts reuse it without spawning a process.
  Extensions can't contribute rules in the first version; a `lint` extension capability is a
  possible later addition.
- One command covers every document kind. `dsc lint` detects the kind of document from its
  `$schema` property. The first phase implements configuration documents and rejects manifests
  with a clear diagnostic; the second phase adds manifests.
- Bicep users benefit too. Bicep compiles to the same configuration document JSON, so `dsc lint`
  can check the compiled output, and the rule identifiers follow the Bicep linter's naming where
  the concept is shared.

## Proposed experience

Linting the document from [#842][02] reports the duplicate resource and exits with code `5`:

```yaml
# demo.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: EnsureTest1Value
  type: Microsoft.Windows/Registry
  properties:
    keyPath: HKEY_LOCAL_MACHINE\SOFTWARE\TEST
    valueName: Test1
    valueData: { DWord: 1 }
- name: EnsureTest2Value
  type: Microsoft.Windows/Registry
  properties:
    keyPath: HKEY_LOCAL_MACHINE\SOFTWARE\TEST
    valueName: Test2
    valueData: { String: Hello World! }
- name: EnsureTest1Value
  type: Microsoft.Windows/Registry
  properties:
    keyPath: HKEY_LOCAL_MACHINE\SOFTWARE\TEST
    valueName: Test
    _exist: false
```

```sh
dsc lint --file ./demo.dsc.yaml
```

```yaml
diagnostics:
- level: error
  rule: duplicate-resource
  category: document
  message: Resource 'EnsureTest1Value' of type 'Microsoft.Windows/Registry' is declared more than once
  path: /resources/2
  file: ./demo.dsc.yaml
summary:
  errors: 1
  warnings: 0
  info: 0
valid: false
discovery: true
```

A pipeline that lints on a build agent without the resources installed skips discovery and fails
on warnings as well as errors:

```sh
dsc lint --file ./demo.dsc.yaml --no-discovery --fail-on warning
```

The rule from [#591][01] fires when `--system-root` is given and no expression calls
`systemRoot()`:

```sh
dsc lint --file ./demo.dsc.yaml --system-root D:\
```

```yaml
diagnostics:
- level: warning
  rule: system-root-unused
  category: environment
  message: A system root of 'D:\' was specified, but no expression calls systemRoot(), so the document doesn't target it
  path: ''
  file: ./demo.dsc.yaml
```

Rule levels can be changed per run or in the settings file, and the rules can be listed:

```sh
dsc lint --file ./demo.dsc.yaml --rule no-unused-parameters=off --rule missing-require-version=warning
dsc lint --list-rules
```

```json
{
  "lint": {
    "rules": {
      "no-unused-parameters": { "level": "off" }
    }
  }
}
```

## Specification

### Command

```text
dsc lint [OPTIONS] (--file <FILE>... | --input <INPUT>)
dsc lint --list-rules [--output-format <FORMAT>]
```

| Option                        | Description                                                                                                                                                      |
| :---------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-f, --file <FILE>`           | A document to lint. `-` reads stdin. The option can be repeated; the result then carries the file name on each diagnostic.                                       |
| `-i, --input <INPUT>`         | The document as a string. Mutually exclusive with `--file`.                                                                                                      |
| `-p, --parameters <INPUT>`    | Parameter values as JSON or YAML, in either format `dsc config` accepts. Enables the rules that check supplied values.                                           |
| `--parameters-file <FILE>`    | Parameter values from a file; `--parameters` takes precedence over it.                                                                                           |
| `-r, --system-root <PATH>`    | The system root the document would be applied with. Enables `system-root-unused`. The path isn't required to exist.                                              |
| `--no-discovery`              | Skip resource discovery and the rules that need it.                                                                                                              |
| `--fail-on <LEVEL>`           | The lowest level that makes the command exit with code `5`: `error` (default), `warning`, or `info`.                                                             |
| `--rule <RULE>=<LEVEL>`       | Override the level of a rule for this run (`off`, `info`, `warning`, `error`). Repeatable. An unknown rule is an argument error.                                 |
| `--list-rules`                | List the rules with the level each reports at in this run instead of linting.                                                                                    |
| `-o, --output-format <FORMAT>`| `json`, `pretty-json`, or `yaml`, with the same defaults as the other commands.                                                                                  |

The `-p` short option is bound to `--parameters` as it is for `dsc config`; the global
`--progress-format` option keeps its `-p` short form because it is only accepted before the
subcommand.

### Document kind

The command reads the `$schema` property and matches it against the recognized URIs of the
configuration document, resource manifest, and extension manifest schemas. In the first phase a
manifest is reported with the fixed `unsupported-document-type` rule. A document without a
recognized `$schema` is treated as a configuration document, so the schema rule reports the
missing or unrecognized URI.

### Pipeline

1. Parse the text as JSON, then as YAML. A failure is the fixed `parse-error` diagnostic at the
   document root, with the parser's message and, when available, the line and column. Input that
   starts with `{` is reported with the JSON parser's message so the message matches what the
   author wrote.
1. Validate the parsed value against the bundled configuration document schema and report every
   violation with the fixed `schema-invalid` rule at the violation's instance path. This is where
   unknown properties, a missing `$schema`, and malformed `dependsOn` entries are reported.
1. Deserialize the value into the typed configuration. When that fails and no schema violation
   explains why, report the deserialization message under `schema-invalid`. Stop when the typed
   document isn't available.
1. Index every expression once. Strings that start with `[` in parameter defaults, variables,
   outputs, user-defined function bodies, resource names, conditions, `dependsOn` entries, copy
   counts, and properties are parsed with the expression grammar without being evaluated. Each
   indexed expression records its JSON pointer and its scope (resource, parameter, variable,
   output, or user-defined function).
1. Run the static rules.
1. Unless discovery is disabled, or the document sets `directives.resourceDiscovery` to
   `duringDeployment`, discover the resource types the document uses with the same filters
   `dsc config` builds (type, `requireVersion`, and `directives.requireAdapter`) and run the
   resource rules. A discovery failure is the fixed `discovery-failed` warning and the resource
   rules are skipped.
1. Sort the diagnostics by file, path, and rule, count them by level, and set `valid` to whether
   there are no errors.

The linter never calls a resource's `get`, `set`, `test`, `export`, `validate`, or `schema`
operation. Property validation only uses the schema embedded in the manifest or loaded by
discovery.

### Levels and precedence

Every rule has a default level: `error`, `warning`, `info`, or `off`. The effective level for a
run is resolved in this order, lowest precedence first:

1. The rule's default.
1. The `lint.rules.<id>.level` setting in `dsc_default.settings.json` and `dsc.settings.json`.
1. The `--rule` options.
1. The `lint.rules.<id>.level` setting in the policy settings file, which wins so that an
   administrator can enforce a rule.

Unknown rules in the settings files are logged and ignored so a stale setting can't break every
run. The fixed rules `parse-error`, `schema-invalid`, `unsupported-document-type`, and
`discovery-failed` ignore overrides with a logged warning.

### Exit codes

| Code | Meaning                                                                                 |
| :--: | :-------------------------------------------------------------------------------------- |
| `0`  | No diagnostic at or above the `--fail-on` level.                                        |
| `1`  | Invalid arguments, such as an unknown rule, a malformed `--rule` value, or no input.    |
| `2`  | Another failure, such as an unreadable `lint` settings section.                         |
| `4`  | A document file or the parameters couldn't be read, or the parameters are malformed.    |
| `5`  | At least one diagnostic at or above the `--fail-on` level.                              |

A document that doesn't parse or doesn't validate against the schema still produces the full
result object on stdout and exits with code `5`, so a pipeline always has structured output to
show.

### Output

The result object is published as the `lint-result` schema (`outputs/lint/result.json`):

```json
{
  "diagnostics": [
    {
      "level": "error",
      "rule": "duplicate-resource",
      "category": "document",
      "message": "Resource 'EnsureTest1Value' of type 'Microsoft.Windows/Registry' is declared more than once",
      "path": "/resources/2",
      "file": "./demo.dsc.yaml",
      "location": { "line": 14, "column": 3 }
    }
  ],
  "summary": { "errors": 1, "warnings": 0, "info": 0 },
  "valid": false,
  "discovery": true
}
```

- `path` is an [RFC 6901][03] JSON pointer into the parsed document. The empty string is the
  document root.
- `file` is present when the document was read from a file.
- `location` is present when the position in the source text is known. In the first phase only
  `parse-error` carries it, because the YAML and JSON parsers DSC uses don't keep positions for
  values. The field is part of the schema from the start so that adding positions later isn't a
  breaking change.
- `valid` is independent of `--fail-on`.
- `discovery` records whether the resource rules ran.

`dsc lint --list-rules` emits an array of objects published as the `lint-rules` schema
(`outputs/lint/rules.json`) with the properties `id`, `category`, `tier` (`static` or
`resource`), `defaultLevel`, `level`, `fixed`, and `description`.

### Rules for configuration documents

Identifiers are kebab-case and follow the Bicep linter's vocabulary where the concept is shared,
so that `no-unused-parameters` and `secure-parameter-default` mean the same thing to a Bicep user.

#### Document

| Rule                             | Default | Checks                                                                                                                                            |
| :------------------------------- | :------ | :------------------------------------------------------------------------------------------------------------------------------------------------ |
| `parse-error`                    | error   | Fixed. The text is valid JSON or YAML.                                                                                                            |
| `schema-invalid`                 | error   | Fixed. Every violation of the configuration document schema, with its path.                                                                       |
| `unsupported-document-type`      | error   | Fixed. The `$schema` isn't a manifest schema (first phase).                                                                                       |
| `duplicate-resource`             | error   | No two sibling resources share a name and a type (case-insensitive). DSC refuses such documents at runtime ([#842][02]).                          |
| `duplicate-resource-name`        | warning | No two sibling resources of different types share a name; dependency resolution looks resources up by name first.                                 |
| `empty-resources`                | warning | The document declares at least one resource.                                                                                                      |
| `unknown-microsoft-dsc-metadata` | warning | Keys under `Microsoft.DSC` metadata are ones DSC recognizes; unknown keys are silently dropped today.                                             |
| `api-version-alias`              | info    | `requireVersion` is used rather than its alias `apiVersion`.                                                                                      |
| `date-version`                   | warning | `requireVersion` isn't a deprecated date-based version.                                                                                           |
| `missing-require-version`        | off     | Every resource pins a version with `requireVersion`.                                                                                              |

#### Expression

| Rule                       | Default | Checks                                                                                                                                       |
| :------------------------- | :------ | :------------------------------------------------------------------------------------------------------------------------------------------- |
| `invalid-expression`       | error   | Every string that starts with `[` (but not `[[`) parses with the expression grammar.                                                         |
| `unknown-function`         | error   | Every call names a built-in function (case-sensitive, with a casing suggestion) or a user-defined function declared in `functions`.           |
| `invalid-argument-count`   | error   | Every call passes a number of arguments within the function's accepted range, using the function registry's metadata and runtime messages.   |
| `invalid-argument-type`    | error   | Every literal argument is of a kind the function accepts at that position; nested expressions aren't checked.                                |
| `undefined-parameter`      | error   | `parameters('x')` names a declared parameter, except inside user-defined function bodies.                                                    |
| `undefined-variable`       | error   | `variables('x')` names a declared variable.                                                                                                  |
| `condition-not-expression` | warning | A `condition` is an expression; a plain string never evaluates to `true`, so the item is always skipped.                                     |

#### Parameter

| Rule                            | Default | Checks                                                                                                                                                        |
| :------------------------------ | :------ | :------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `no-unused-parameters`          | warning | Every declared parameter is referenced.                                                                                                                       |
| `no-unused-variables`           | warning | Every declared variable is referenced.                                                                                                                        |
| `parameter-default-type`        | error   | A literal default satisfies the type, `allowedValues`, value limits, and length limits, reusing the runtime constraint checks.                                 |
| `invalid-parameter-constraints` | error   | Constraints are consistent: `minValue <= maxValue`, `minLength <= maxLength`, value limits only on `int`, length limits only on string and array types, non-empty `allowedValues` of the right type. |
| `secure-parameter-default`      | warning | Secure parameters have no hard-coded default (an empty string or an expression is allowed).                                                                   |
| `prefer-secure-type`            | warning | A `string` or `object` parameter whose name suggests a secret uses a secure type.                                                                             |
| `secure-value-in-output`        | warning | Outputs don't expose a secure parameter or `secret()`; DSC skips such outputs at runtime.                                                                     |
| `use-parameter-description`     | off     | Every parameter has a description.                                                                                                                            |
| `undeclared-parameter-value`    | error   | With supplied parameters: every supplied value matches a declared parameter.                                                                                  |
| `missing-parameter-value`       | error   | With supplied parameters: every parameter without a default has a value.                                                                                      |
| `parameter-value-type`          | error   | With supplied parameters: every value satisfies the type and constraints.                                                                                     |

#### Dependency

| Rule                           | Default | Checks                                                                                                                                                                   |
| :----------------------------- | :------ | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `unresolved-dependency`        | error   | Every literal `dependsOn` entry and `reference(resourceId(...))` call points at a sibling resource; a name with the wrong type gets its own message. Copy loops are skipped. |
| `circular-dependency`          | error   | The `dependsOn` graph of each sibling group has no cycle, including self-dependencies; the message shows the cycle.                                                      |
| `reference-without-depends-on` | warning | A resource that calls `reference()` lists the referenced resource in `dependsOn`, because `reference()` only sees resources that already ran.                             |
| `duplicate-depends-on`         | info    | No dependency is listed twice.                                                                                                                                           |

#### Deprecation

| Rule                           | Default | Checks                                                                                              |
| :----------------------------- | :------ | :-------------------------------------------------------------------------------------------------- |
| `copy-loop-deprecated`         | warning | No resource or output uses a copy loop ([#1429][04]).                                               |
| `copy-options-unsupported`     | error   | No copy loop sets `mode` or `batchSize`, which DSC rejects.                                         |
| `security-context-in-metadata` | warning | The security context is set with `directives.securityContext`, not `Microsoft.DSC` metadata.        |
| `security-context-conflict`    | error   | The metadata and directive security contexts don't contradict each other.                           |
| `dsc-version-unsatisfied`      | warning | `directives.version` is satisfied by the version of `dsc` running the linter.                       |

#### Environment

| Rule                    | Default | Checks                                                                                            |
| :---------------------- | :------ | :------------------------------------------------------------------------------------------------ |
| `system-root-unused`    | warning | With `--system-root`: some expression calls `systemRoot()` ([#591][01]).                          |
| `hardcoded-system-path` | off     | Property values aren't absolute operating system paths that `systemRoot()` should build instead.  |

#### Resource (needs discovery)

| Rule                           | Default | Checks                                                                                                                                      |
| :----------------------------- | :------ | :------------------------------------------------------------------------------------------------------------------------------------------ |
| `discovery-failed`             | warning | Fixed. Discovery succeeded; otherwise the resource rules are skipped.                                                                       |
| `unknown-resource-type`        | warning | Every resource type is installed. A warning, because the document may be valid on the machine it targets.                                   |
| `unsatisfied-require-version`  | warning | An installed version satisfies `requireVersion`.                                                                                            |
| `deprecated-resource`          | warning | No resource type carries a `deprecationMessage`.                                                                                            |
| `property-schema-mismatch`     | error   | Literal properties validate against the embedded resource schema; violations under properties that hold expressions are dropped.            |
| `nested-resources-unsupported` | warning | Only group and adapter resources declare nested `resources`.                                                                                |
| `exist-false-unsupported`      | error   | A resource with `_exist: false` has the `delete` or `setHandlesExist` capability; otherwise `dsc config set` fails.                          |
| `canonical-type-name`          | info    | Resource types are written with the casing of their manifest.                                                                               |

### Rules for manifests (second phase)

The same command, engine, output, and settings apply. The document kind is detected from
`$schema`, and `dsc lint --type resource-manifest` can force it. Proposed rules:

| Rule                      | Default | Checks                                                                                                   |
| :------------------------ | :------ | :------------------------------------------------------------------------------------------------------- |
| `manifest-schema-invalid` | error   | Fixed. Every violation of the manifest schema.                                                           |
| `kind-method-mismatch`    | error   | `adapter` kinds define `adapter`, `importer` kinds define `resolve`, `exporter` kinds define `export`.   |
| `missing-schema`          | warning | A non-adapter resource defines an embedded schema or a schema command.                                   |
| `embedded-schema-invalid` | error   | The embedded schema compiles as a JSON schema.                                                           |
| `deprecated-whatif-method`| warning | The deprecated top-level `whatIf` method isn't used.                                                     |
| `exit-code-zero-success`  | error   | Exit code `0` isn't described as a failure.                                                              |
| `missing-description`     | info    | The manifest has a description.                                                                          |
| `executable-not-found`    | warning | Discovery tier: every executable the manifest names resolves on the path.                                |

### Library design

The engine is the `dsc_lib::lint` module:

- `Linter::new(LintOptions)` resolves the effective rule levels and prepares the expression parser
  and the compiled document schema; `Linter::lint(&str)` and `Linter::lint_value(&Value)` run the
  pipeline and return a `LintResult`. Problems in the document never surface as `Err`; `Err` is
  reserved for engine faults such as an unknown rule or malformed parameters.
- Each rule is a `Rule` value with an identifier, category, tier, default level, localized
  description, and a check function that receives a `LintContext` (the raw value, the typed
  configuration, the flattened resource sites, the expression index, the options, the function
  registry, the supplied parameters, and the discovery results) and reports through a
  `DiagnosticSink` that stamps the rule identity and level on every diagnostic.
- The expression parser gains a parse-only entry point (`Statement::parse`) and read access to
  the parsed tree, and the function dispatcher gains `validate_arg_count` and
  `validate_literal_args`, so the rules use the same metadata and messages as the runtime without
  evaluating anything. Schema validation gains `validate_json_all`, which returns every violation
  with its instance path instead of only the first.
- The CLI reads the raw document text instead of normalizing it to JSON first, so syntax errors
  become diagnostics rather than an early exit.

### Relationship to `dsc config validate`

The hidden `validate` subcommand keeps its current contract for the Group, Assertion, and Include
resources. Once `dsc lint` is in place, `validate` can delegate to the engine's fixed rules and
the resource tier, which gives it the duplicate-resource check from [#842][02] and real reasons,
without changing its output shape. That change is out of scope for this RFC.

## Alternate Proposals and Considerations

- **`dsc config lint` instead of `dsc lint`.** A verb under `config` matches the noun-verb grammar
  of the other commands and inherits the group's `--parameters` and `--system-root` options.
  Manifests would then need `dsc resource lint` and `dsc extension lint`. A single top-level
  command was chosen because it mirrors `bicep lint`, matches the original proposal in
  [#591][01], gives one entry point for every document kind, and accepts several files in one run.
- **Growing `dsc config validate`.** [#842][02] suggested a `--schema-only` flag for the internal
  callers and static analysis in `validate`. That mixes a pass-or-fail contract used by three
  resources with a diagnostics contract meant for people and pipelines. Keeping `validate` as is
  and building `lint` next to it avoids a breaking change for the resources.
- **Linting as an extension.** Extensions today provide discovery, import, and secrets, and run as
  separate processes. The rules need the parser, function registry, schema, and discovery, which
  are internal to `dsc-lib`, and editors want to lint in process. Extension-contributed rules
  remain possible later through a new capability.
- **Relying on Bicep's linter.** Bicep can add DSC-specific rules for Bicep authors, but it
  doesn't help the YAML and JSON authoring community, and the compiled JSON can still be checked
  with `dsc lint`.
- **Discovery off by default.** A purely static default is faster and environment-independent,
  but `bicep lint` restores modules by default and the hidden `validate` discovers today. Discovery
  is on by default with `--no-discovery` as the escape hatch, and the discovery-dependent rules
  report warnings rather than errors so a build agent without resources still passes.
- **Source positions.** The YAML and JSON parsers DSC uses don't keep positions for values, so
  diagnostics carry JSON pointers and only parse errors carry a line and column. A later pass with
  a marker-emitting YAML parser can fill `location` for every diagnostic without a schema change.
- **Future output formats.** SARIF output for code-scanning integrations and inline suppression in
  the document are natural follow-ups once the rule catalog settles.

## Related work items

- [#591][01] - Proposes `dsc lint` and the `systemRoot()` rule; discusses extension versus core
  and linting manifests.
- [#842][02] - Asks `dsc config validate` to reject duplicate resource names.
- [#1429][04] - Deprecates copy loops, which the `copy-loop-deprecated` rule reports.

[01]: https://github.com/PowerShell/DSC/issues/591
[02]: https://github.com/PowerShell/DSC/issues/842
[03]: https://www.rfc-editor.org/rfc/rfc6901
[04]: https://github.com/PowerShell/DSC/issues/1429
