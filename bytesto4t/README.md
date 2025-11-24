# ByteSto4t

GUI application for exploring and editing HashLink bytecode using the `hlbc`

## MCP Integration

ByteSto4t includes a built‑in MCP server scaffold based on `prism-mcp-rs`. This lets AI coding agents discover and call tools inside the app.

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