# ByteSto4t MCP server

The server runs over stdio when ByteSto4t is launched with `--mcp`. A typical session starts with `load_bytecode`, uses the list and inspection tools to discover indexes, applies optional in-memory edits, and finishes with `save_bytecode`.

## In-app assistant (experimental and unofficial)

The `Assistant` workspace tab uses ChatGPT OAuth and an experimental ChatGPT Codex compatibility endpoint. This is not an official OpenAI integration, and endpoint availability, account entitlements, or protocol behavior can change. It exposes a read-only subset of the same MCP handlers to the model: dashboard, function and type discovery, inspection, decompilation, disassembly, references, and type-usage lookup. The in-app assistant cannot mutate or save bytecode.

Before the first connection or request, bytesto4t requires acceptance of a versioned privacy disclosure:

- Messages are sent to ChatGPT.
- Selected bytecode metadata and requested decompilation, disassembly, and other tool output may be sent to ChatGPT.
- Chat history is stored locally as XChaCha20-Poly1305 authenticated ciphertext in the Tauri application-data directory. A random 256-bit key is stored in the operating-system credential vault. History records are versioned, bounded, authenticated before use, and replaced atomically.
- A configured external HTTP helper receives OAuth headers and request bodies through stdin. `BYTESTO4T_HTTP_HELPER` must point to a fully trusted executable because that process can read OAuth tokens and Assistant payloads.
- Custom proxies can observe connection metadata.
- Disabling TLS verification permits interception of OAuth credentials and analyzed content. The UI requires a separate warning confirmation before this setting can first be enabled.

Existing plaintext `bytesto4t.assistant.chats.v2` and legacy v1 WebView storage is migrated once. Plaintext keys are deleted only after encrypted persistence succeeds. Removing a chat replaces the encrypted store without that chat; removing all chats deletes the encrypted history file.

OAuth access and refresh tokens, account identifiers, and the history key are stored in the operating-system credential vault and are never written to the normal bytesto4t config. Oversized OAuth token payloads are split across versioned, integrity-checked vault entries to stay below the Windows Credential Manager limit. Model, reasoning, proxy, TLS, and privacy-disclosure preferences are stored in the normal config. The OAuth client ID is a public identifier, not a client secret.

Network requests follow the system proxy and VPN route by default. Custom proxy URLs support HTTP, HTTPS, and SOCKS5, but URLs containing user information, query strings, or fragments are rejected; bytesto4t does not store proxy passwords in its config. If a Windows VPN/filter driver rejects the app's socket with `Access is denied`, OAuth and compatibility requests automatically retry through an external `curl` process. Curl configuration, including headers and bodies, continues to be passed through stdin rather than command-line arguments. Helper failures do not include stderr, authorization headers, request bodies, or provider response bodies.

Assistant response payloads set `"store": false`. This is a request preference, not a guarantee that the provider retains nothing; provider policies still apply. ChatGPT and API billing remain separate.

The Assistant implements OAuth 2.0 Authorization Code with PKCE and communicates with a Codex-compatible ChatGPT endpoint. Reusing an OAuth client or backend owned by another application may require explicit provider authorization; publication of client code or a public OAuth client ID does not grant that authorization.

## Index conventions

Most bytecode pools use a zero-based vector `index`. Functions and natives also share a second, dense identifier namespace named `findex`.

Function list entries use this display format:

```text
<name>@<findex>@<index>
```

For example, `toLowerCase@1@444` has `findex` 1 and vector `index` 444. Tools with an `index` argument use the vector index unless their schema explicitly says otherwise. `export_function_json` accepts the vector `index`; its legacy `function_index` argument is retained as a `findex` lookup for compatibility.

`get_function_list` returns the display entries as text and also exposes `{ "functions": [...] }` in MCP `structuredContent`.

## Tools

| Category | Tools |
|---|---|
| Session | `load_bytecode`, `get_dashboard_info`, `get_target_file_info`, `read_binary_file`, `merge_bytecode`, `save_bytecode` |
| Inspection | `get_inspector_info`, `get_decompiled_info`, `get_disassembler_info`, `get_references` |
| Functions | `get_function_list`, `get_function_full_info`, `get_function_name_by_index`, `list_functions_with_constructors`, `find_functions_using_type`, `create_function`, `update_function`, `delete_function`, `import_function_json`, `export_function_json` |
| Types | `get_type_list`, `get_type_full_info`, `create_type`, `update_type`, `delete_type`, `import_type_json`, `export_type_json`, `generate_imhex_pattern` |
| Strings | `get_string_list`, `get_string_full_info`, `create_string`, `update_string`, `delete_string` |
| Globals | `get_global_list`, `get_global_full_info`, `create_global`, `update_global`, `delete_global` |
| Natives | `get_native_list`, `get_native_full_info`, `create_native`, `update_native`, `delete_native` |
| Constants | `get_constant_list`, `get_constant_full_info`, `create_constant`, `update_constant`, `delete_constant` |
| Integers | `get_int_list`, `get_int_full_info`, `create_int`, `update_int`, `delete_int` |
| Floats | `get_float_list`, `get_float_full_info`, `create_float`, `update_float`, `delete_float` |
| Bytes and files | `get_bytes_list`, `get_bytes_full_info`, `get_file_list` |
| Addresses | `load_function_addresses`, `get_function_addresses` |
| Text exports | `save_function_list`, `save_type_list`, `save_file_list`, `save_disassembled_code` |

`get_inspector_info` and `get_references` support these `typ` values: `function`, `class`, `type`, `file`, `global`, `constant`, `string`, `int`, `float`, `native`, and `bytes`.

`get_decompiled_info` supports `function` and object `class` items. `get_disassembler_info` supports functions plus object and enum `class` items.

## Editing behavior

Edits affect the loaded in-memory bytecode. They are not written back to the source file automatically. Use `save_bytecode` to serialize a stripped bytecode file to a new path.

Function and native `findex` values must remain unique and dense across both pools. ByteSto4t allocates the next shared value when `findex` is omitted, rejects collisions, repairs references when indexes are compacted, and rebuilds HLBC runtime lookup indexes after affected edits.

An existing function or native cannot change its `findex` through an update. Create a replacement item when a different shared index is required.

Pool deletion is intentionally conservative. Indexed elements can only be deleted from the end of their vector and only when no surviving bytecode item references them. This prevents silent index shifting and corrupt output.

Function, native, object, enum, field, and constructor names are stored as string-pool references. Arguments named `name` or `lib` in those create/update tools therefore contain decimal string-pool indexes. For `create_function` and `update_function`, `is_constructor: true` resolves the `new` string automatically and ignores `name`.

HLBC does not serialize a standalone function name directly. A function name survives save and reload only when the function is connected to an object prototype or binding; otherwise it is displayed as `<none>` after reload.

### Opcode editing

Opcode JSON uses the externally tagged `hlbc::Opcode` representation, for example
`{"JAlways":{"offset":2}}`. `create_function` and `update_function` validate the complete
function, but `update_function` replaces the opcode array as supplied; it does not repair control
flow after insertions, removals, or moves.

- Jump targets are relative to the next opcode: `target = opcode_index + 1 + offset`. Backward
  jumps emitted for loops target a `Label` opcode.
- `Switch.offsets` and `Switch.end` are non-negative offsets relative to the next opcode. A selector
  outside `offsets` falls through to the next opcode. `end` is the structural join after all switch
  arms, not the default target.
- `MakeEnum.construct`, `EnumAlloc.construct`, and `EnumField.construct` are indexes local to the
  enum type, not indexes in the global type pool.
- `EndTrap.normal` is a boolean marker. `true` is the normal end of a try body; `false` is cleanup
  emitted before an early return, break, or continue.
- `Prefetch.field` uses the wire encoding: `0` means the value itself and `n + 1` means field `n`.
  `Asm.reg` similarly uses `0` for no register and `n + 1` for register `n`.
- When debug data is present, `debug_info` must contain exactly one entry per opcode. Recalculate
  relative offsets and debug positions whenever the opcode array changes.
