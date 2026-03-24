---
name: persona-developer
version: 1.0.0
description: "Developer: Integrate transactional email and SMS into applications using the nlm CLI."
metadata:
  openclaw:
    category: "persona"
    requires:
      bins: ["nlm"]
      skills: ["nlm-sn-email", "nlm-sn-sms", "nlm-mcp", "nlm-config"]
---

# Persona — Developer

Role-based skill bundle for developers integrating transactional email and SMS into
applications using the `nlm` CLI for prototyping, testing, and scripting.

## Prerequisite Skills

This persona depends on the following skills for full command reference:

- **nlm-sn-email** — Send transactional emails and query delivery events
- **nlm-sn-sms** — Send transactional SMS messages and query delivery events
- **nlm-mcp** — MCP tool discovery and invocation for AI-powered features
- **nlm-config** — Profile management and environment setup

## Relevant Workflows

- **recipe-transactional-email-setup** — End-to-end transactional email configuration
- **recipe-mcp-tool-exploration** — Discover and invoke MCP tools

## Instructions

1. **Use MCP tools for AI-powered features** — discover available tools and invoke
   them programmatically:

   ```bash
   nlm mcp tools
   nlm mcp call tool-name '{"param": "value"}'
   ```

2. **Send transactional emails with template variables** using Surenotify's
   `{{variable}}` syntax:

   ```bash
   nlm sn email send \
     --to user@example.com \
     --subject 'Order {{order_id}}' \
     --html '<p>Hi {{name}}, your order is confirmed.</p>'
   ```

3. **Pipe output to `jq` for scripting** — JSON auto-compacts when stdout is piped:

   ```bash
   nlm sn email events | jq '.[] | .event_type'
   nlm sn sms events | jq '.[] | select(.status == "delivered")'
   ```

4. **Use exit codes in scripts** for robust error handling:

   ```bash
   nlm sn email send --to user@example.com --subject "Test" --html "<p>Hi</p>"
   case $? in
     0) echo "Sent successfully" ;;
     1) echo "API rejected the request" ;;
     2) echo "Invalid parameters" ;;
     3) echo "Check your API key" ;;
     4) echo "Network issue or rate limited" ;;
     5) echo "File I/O problem" ;;
   esac
   ```

5. **Use `--dry-run` for testing** API calls without side effects — ideal for
   development and CI:

   ```bash
   nlm sn email send --to test@example.com --subject "CI test" --dry-run
   ```

6. **JSON output auto-compacts when stdout is piped**, so you can chain commands
   without worrying about pretty-print formatting breaking parsers.

## Tips

- Use `NL_SN_API_KEY` and `NL_EDM_API_KEY` environment variables in CI pipelines
  for clean secret management.
- The `--dry-run` flag outputs the HTTP request that *would* be sent — useful for
  debugging template variable interpolation.
- MCP tools extend the CLI with AI-powered capabilities; run `nlm mcp tools` to see
  what is available in your configured MCP server.
- Remember: Surenotify uses `{{variable}}` syntax, EDM uses `${VARIABLE}`. The CLI
  warns if you mix them up.
