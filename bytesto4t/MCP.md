# ByteSto4t MCP server

The server runs over stdio when ByteSto4t is launched with `--mcp`. A typical session starts with `load_bytecode`, uses the list and inspection tools to discover indexes, applies optional in-memory edits, and finishes with `save_bytecode`.

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
