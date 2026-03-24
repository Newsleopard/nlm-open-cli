---
name: recipe-multi-profile-workflow
version: 1.0.0
description: "Manage staging and production environments with config profiles."
metadata:
  openclaw:
    category: "recipe"
    domain: "config"
    requires:
      bins: ["nlm"]
      skills: ["nlm-config"]
---

# Recipe: Multi-Profile Workflow

Manage staging and production environments using config profiles — create
profiles, set per-environment API keys, and switch between them.

## Prerequisites

- `nlm` installed
- API keys for each environment (staging, production)

---

## Steps

### Step 1 — Initialize default config

Run the interactive setup wizard to configure your default (production)
profile.

```bash
nlm config init
```

Enter your production API keys when prompted.

### Step 2 — Create a staging profile

```bash
nlm config profile create staging
```

### Step 3 — Set staging API keys

```bash
nlm config set edm_api_key "staging-key" --profile staging
nlm config set sn_api_key "staging-sn-key" --profile staging
```

### Step 4 — Test the staging profile

Verify the staging keys work by checking the account balance.

```bash
nlm edm account balance --profile staging
```

### Step 5 — Use the production profile

Switch back to production (the default profile) for live operations.

```bash
nlm edm account balance --profile default
```

Or simply omit the `--profile` flag — `default` is used automatically.

---

## Tips

- **Environment variables override profiles:** If `NL_EDM_API_KEY` is set, it
  takes precedence over any profile. Unset it when switching profiles
  interactively.
- **CI/CD pattern:** In CI pipelines, use environment variables instead of
  profiles: `NL_EDM_API_KEY=... nlm edm account balance`. This avoids
  persisting secrets to disk.
- **List all profiles:** Use `nlm config profile list` to see available
  profiles and which is currently active.
- **Per-profile format:** Set `default_format` per profile — e.g., `table`
  for staging (human review) and `json` for production (scripting).
