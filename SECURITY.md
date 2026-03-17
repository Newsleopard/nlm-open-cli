# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in `nl-cli`, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please email **rd@newsleopard.tw** with:

- A description of the vulnerability
- Steps to reproduce
- Potential impact

We will acknowledge your report within 48 hours and aim to provide a fix within 7 days for critical issues.

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | Yes       |

## Security Considerations

- API keys are stored in `~/.config/nl/config.toml` with restricted file permissions (600).
- API keys are never printed to stdout or included in logs.
- `nl config list` masks key values in output.
- The `--dry-run` flag masks the `x-api-key` header value.
