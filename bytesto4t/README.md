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

- `get_function_list` — lists functions from the current bytecode
