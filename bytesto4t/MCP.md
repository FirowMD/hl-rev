### Function Naming Convention

Functions in the bytecode have a unique naming format that includes both a **function index (findex)** and a **vector index (index)**:

- Format: `<name>@<findex>@<index>`
- Example: `toLowerCase@1@444` where:
  - `@1` is the **findex** (function index)
  - `@444` is the **index** (vector index)

**CRITICAL**: When searching for functions by the number after `@`, you are looking for the **findex**, not the index!

- ✅ `getRankingAPI@22664` means: find function with **findex** 22664
- ❌ `getRankingAPI@22664` does NOT mean: find function at vector index 22664

**Important**: Unlike other bytecode elements (types, globals, constants, etc.) that only have an `index`, functions have both `findex` and `index`. This dual indexing is specific to functions and is important for correctly identifying and working with function references in the bytecode.

- Current tool: `get_function_list` — returns the list of functions from the loaded bytecode.
- Response shape:

```json
{
  "functions": ["<name>@<findex>@<index>", "..."]
}
```

### Available tool(s)

| Category | Command | Description | Requires Selected Item |
|---|---|---|---|
| Core / Bytecode | `load_bytecode` | Load bytecode file into app | No |
| Core / Bytecode | `get_dashboard_info` | Get dashboard summary info | No |
| Core / Bytecode | `set_target_file_path` | Set target file path and load bytecode | No |
| Core / Bytecode | `clear_references` | Clear saved references | No |
| Core / Bytecode | `get_saved_references` | Get saved references | No |
| Core / Bytecode | `read_binary_file` | Read file bytes | No |
| Core / Bytecode | `merge_bytecode_with_file` | Merge current bytecode with file | No |
| Functions | `get_function_list` | Get list of functions | No |
| Functions | `add_function` | Add a new function (create_function) | No |
| Functions | `update_function` | Update an existing function | No |
| Functions | `remove_function` | Remove a function (delete_function) | No |
| Functions | `get_function_full_info` | Get full function info as JSON | No |
| Functions | `get_function_name_by_index` | Get function display name by vector index | No |
| Functions | `list_functions_with_constructors` | List functions with constructor flag | No |
| Functions | `find_functions_using_type` | Find functions using a specific type | No |
| Functions | `import_function_json` | Import function from JSON file | No |
| Functions | `export_function_json` | Export function to JSON file | No |
| Types | `get_type_list` | Get list of types | No |
| Types | `add_type` | Add a new type (create_type) | No |
| Types | `update_type` | Update an existing type | No |
| Types | `remove_type` | Remove a type (delete_type) | No |
| Types | `get_type_full_info` | Get full type info as JSON | No |
| Types | `import_type_json` | Import type from JSON file | No |
| Types | `export_type_json` | Export type to JSON file | No |
| Types | `generate_imhex_pattern` | Generate ImHex pattern for a type | No |
| Decompiler / Inspector | `get_decompiled_info` | Get decompiled info for item (`index`, `typ` args) | No |
| Decompiler / Inspector | `get_disassembler_info` | Get disassembler info for item (`index`, `typ` args) | No |
| Decompiler / Inspector | `get_inspector_info` | Get inspector info for item (`index`, `typ` args). Includes references for function/string/global | No |
| Data | `get_file_list` | Get list of debug files | No |
| Data | `get_string_list` | Get list of strings | No |
| Data | `add_string` | Add a new string | No |
| Data | `update_string` | Update an existing string | No |
| Data | `remove_string` | Remove a string by index | No |
| Data | `get_string_full_info` | Get full info for a string | No |
| Data | `get_global_list` | Get list of globals | No |
| Data | `add_global` | Add a new global | No |
| Data | `update_global` | Update an existing global | No |
| Data | `remove_global` | Remove a global by index | No |
| Data | `get_global_full_info` | Get global info | No |
| Data | `get_native_list` | Get list of natives | No |
| Data | `add_native` | Add a new native | No |
| Data | `update_native` | Update an existing native | No |
| Data | `remove_native` | Remove a native by index | No |
| Data | `get_native_full_info` | Get full native info | No |
| Data | `get_constant_list` | Get list of constants | No |
| Data | `add_constant` | Add a new constant | No |
| Data | `update_constant` | Update an existing constant | No |
| Data | `remove_constant` | Remove a constant by index | No |
| Data | `get_constant_full_info` | Get full info for a constant | No |
| Data | `get_int_list` | Get list of ints | No |
| Data | `add_int` | Add a new int | No |
| Data | `update_int` | Update an existing int | No |
| Data | `remove_int` | Remove an int by index | No |
| Data | `get_int_full_info` | Get full info for an int | No |
| Data | `get_float_list` | Get list of floats | No |
| Data | `add_float` | Add a new float | No |
| Data | `update_float` | Update an existing float | No |
| Data | `remove_float` | Remove a float by index | No |
| Data | `get_float_full_info` | Get full info for a float | No |
| Data | `get_bytes_list` | Get list of bytes entries | No |
| Data | `get_bytes_full_info` | Get full bytes for an entry | No |

**Note:** Commands no longer use `set_selected_item`/`get_selected_item`. Operations that previously required a selected item now take `index` (string) and `typ` (string) arguments directly and set selection internally for convenience.
