# ByteSto4t

GUI application for exploring and editing HashLink bytecode using the `hlbc`

## MCP Integration

ByteSto4t includes a built‑in MCP server scaffold based on `prism-mcp-rs`. This lets AI coding agents discover and call tools inside the app.

- Current tool: `get_function_list` — returns the list of functions from the loaded bytecode.
- Response shape:

```json
{
  "functions": ["<name><findex>@<index>", "..."]
}
```

### How to enable and use

Add the following to your config file (e.g., `.mcp.json`):

```json
{
  "mcpServers": {
    "bytesto4t-server": {
      "command": "bytesto4t",
      "args": [
        "--mcp"
      ]
    }
  }
}
```

### Available tool(s)

| Category | Command | Description |
|---|---|---|
| Core / Bytecode | `load_bytecode` | Load bytecode file into app |
| Core / Bytecode | `get_dashboard_info` | Get dashboard summary info |
| Core / Bytecode | `set_target_file_path` | Set target file path and load bytecode |
| Core / Bytecode | `set_selected_item` | Set selected item |
| Core / Bytecode | `get_selected_item` | Get selected item |
| Core / Bytecode | `clear_references` | Clear saved references |
| Core / Bytecode | `get_saved_references` | Get saved references |
| Core / Bytecode | `read_binary_file` | Read file bytes |
| Core / Bytecode | `merge_bytecode_with_file` | Merge current bytecode with file |
| Functions | `get_function_list` | Get list of functions |
| Functions | `add_function` | Add a new function (create_function) |
| Functions | `update_function` | Update an existing function |
| Functions | `remove_function` | Remove a function (delete_function) |
| Functions | `get_function_full_info` | Get full function info as JSON |
| Functions | `get_function_name_by_index` | Get function display name by vector index |
| Functions | `list_functions_with_constructors` | List functions with constructor flag |
| Functions | `find_functions_using_type` | Find functions using a specific type |
| Functions | `import_function_json` | Import function from JSON file |
| Functions | `export_function_json` | Export function to JSON file |
| Types | `get_type_list` | Get list of types |
| Types | `add_type` | Add a new type (create_type) |
| Types | `update_type` | Update an existing type |
| Types | `remove_type` | Remove a type (delete_type) |
| Types | `get_type_full_info` | Get full type info as JSON |
| Types | `import_type_json` | Import type from JSON file |
| Types | `export_type_json` | Export type to JSON file |
| Types | `generate_imhex_pattern` | Generate ImHex pattern for a type |
| Decompiler / Inspector | `get_decompiled_info` | Get decompiled info for selected item |
| Decompiler / Inspector | `get_disassembler_info` | Get disassembler info for selected item |
| Decompiler / Inspector | `get_inspector_info` | Get inspector info for selected item |
| Data | `get_file_list` | Get list of debug files |
| Data | `get_string_list` | Get list of strings |
| Data | `add_string` | Add a new string |
| Data | `update_string` | Update an existing string |
| Data | `remove_string` | Remove a string by index |
| Data | `get_string_full_info` | Get full info for a string |
| Data | `get_global_list` | Get list of globals |
| Data | `add_global` | Add a new global |
| Data | `update_global` | Update an existing global |
| Data | `remove_global` | Remove a global by index |
| Data | `get_global_full_info` | Get global info |
| Data | `get_native_list` | Get list of natives |
| Data | `add_native` | Add a new native |
| Data | `update_native` | Update an existing native |
| Data | `remove_native` | Remove a native by index |
| Data | `get_native_full_info` | Get full native info |
| Data | `get_constant_list` | Get list of constants |
| Data | `add_constant` | Add a new constant |
| Data | `update_constant` | Update an existing constant |
| Data | `remove_constant` | Remove a constant by index |
| Data | `get_constant_full_info` | Get full info for a constant |
| Data | `get_int_list` | Get list of ints |
| Data | `add_int` | Add a new int |
| Data | `update_int` | Update an existing int |
| Data | `remove_int` | Remove an int by index |
| Data | `get_int_full_info` | Get full info for an int |
| Data | `get_float_list` | Get list of floats |
| Data | `add_float` | Add a new float |
| Data | `update_float` | Update an existing float |
| Data | `remove_float` | Remove a float by index |
| Data | `get_float_full_info` | Get full info for a float |
| Data | `get_bytes_list` | Get list of bytes entries |
| Data | `get_bytes_full_info` | Get full bytes for an entry |
