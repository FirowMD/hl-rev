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

| Category | Command | Description |
|---|---|---|
| Core / Bytecode | `load_bytecode` | Load bytecode file into app |
| Core / Bytecode | `get_dashboard_info` | Get dashboard summary info |
| Core / Bytecode | `read_binary_file` | Read file bytes |
| Functions | `get_function_list` | Get list of functions |
| Functions | `create_function` | Create a new function |
| Functions | `update_function` | Update an existing function |
| Functions | `delete_function` | Delete a function |
| Functions | `get_function_full_info` | Get full function info as JSON |
| Functions | `get_function_name_by_index` | Get function display name by vector index |
| Functions | `list_functions_with_constructors` | List functions with constructor flag |
| Functions | `find_functions_using_type` | Find functions using a specific type |
| Functions | `import_function_json` | Import function from JSON file |
| Functions | `export_function_json` | Export function to JSON file |
| Types | `get_type_list` | Get list of types |
| Types | `create_type` | Create a new type |
| Types | `update_type` | Update an existing type |
| Types | `delete_type` | Delete a type |
| Types | `get_type_full_info` | Get full type info as JSON |
| Types | `import_type_json` | Import type from JSON file |
| Types | `export_type_json` | Export type to JSON file |
| Types | `generate_imhex_pattern` | Generate ImHex pattern for a type |
| Decompiler / Inspector | `get_decompiled_info` | Get decompiled info for item (`index`, `typ` args) |
| Decompiler / Inspector | `get_disassembler_info` | Get disassembler info for item (`index`, `typ` args) |
| Decompiler / Inspector | `get_references` | Get references for item (`index`, `typ` args) |
| Data | `get_file_list` | Get list of debug files |
| Data | `get_string_list` | Get list of strings |
| Data | `create_string` | Create a new string |
| Data | `update_string` | Update an existing string |
| Data | `delete_string` | Delete a string by index |
| Data | `get_string_full_info` | Get full info for a string |
| Data | `get_global_list` | Get list of globals |
| Data | `create_global` | Create a new global |
| Data | `update_global` | Update an existing global |
| Data | `delete_global` | Delete a global by index |
| Data | `get_global_full_info` | Get global info |
| Data | `get_native_list` | Get list of natives |
| Data | `create_native` | Create a new native |
| Data | `update_native` | Update an existing native |
| Data | `delete_native` | Delete a native by index |
| Data | `get_native_full_info` | Get full native info |
| Data | `get_constant_list` | Get list of constants |
| Data | `create_constant` | Create a new constant |
| Data | `update_constant` | Update an existing constant |
| Data | `delete_constant` | Delete a constant by index |
| Data | `get_constant_full_info` | Get full info for a constant |
| Data | `get_int_list` | Get list of ints |
| Data | `create_int` | Create a new int |
| Data | `update_int` | Update an existing int |
| Data | `delete_int` | Delete an int by index |
| Data | `get_int_full_info` | Get full info for an int |
| Data | `get_float_list` | Get list of floats |
| Data | `create_float` | Create a new float |
| Data | `update_float` | Update an existing float |
| Data | `delete_float` | Delete a float by index |
| Data | `get_float_full_info` | Get full info for a float |
| Data | `get_bytes_list` | Get list of bytes entries |
| Data | `get_bytes_full_info` | Get full bytes for an entry |

