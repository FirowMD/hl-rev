# ByteSto4t MCP integration

ByteSto4t includes a stdio MCP server based on `prism-mcp-rs`. It exposes HashLink bytecode loading, inspection, decompilation, editing, merging, and export tools.

Build or install the `bytesto4t` executable, then add it to the MCP client configuration:

```json
{
  "mcpServers": {
    "bytesto4t": {
      "command": "bytesto4t",
      "args": ["--mcp"]
    }
  }
}
```

Use an absolute executable path when `bytesto4t` is not on `PATH`. See [MCP.md](MCP.md) for the tool contract and indexing rules.
