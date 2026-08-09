# ByteSto4t MCP integration

ByteSto4t includes a stdio MCP server based on `prism-mcp-rs`. It exposes HashLink bytecode loading, inspection, decompilation, editing, merging, and export tools. The desktop workspace also includes an experimental, unofficial MCP-backed Assistant using ChatGPT OAuth and a Codex compatibility endpoint.

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

Use an absolute executable path when `bytesto4t` is not on `PATH`. See [MCP.md](MCP.md) for the Assistant privacy disclosure, encrypted history, trusted-helper requirement, OAuth/provider authorization caveat, VPN/proxy behavior, tool contracts, and indexing rules.
