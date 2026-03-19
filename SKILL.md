---
name: available
description: >
  Find and check project name availability across domains and package registries using the `available` CLI tool.
  Use this skill whenever the user wants to brainstorm or generate project names, check if a specific name
  is available (domains, npm, crates.io, PyPI, etc.), find a name for a new tool/library/app, or assess
  domain and package registry availability for any name. Also trigger when the user says things like
  "what should I call this", "is this name taken", "find me a name for", "check if X is available",
  or is starting a new project and needs naming help.
user-invocable: true
argument-hint: "[project description or names to check]"
---

# Available — Project Name Finder

Find memorable, available project names by combining AI generation with real-time domain and package registry checks.

## When This Skill Activates

- User wants to **name a new project** ("I need a name for my CLI tool that...")
- User wants to **check specific names** ("is rushq available?", "check ferrotask and ironpipe")
- User is **starting a new project** and hasn't picked a name yet
- User asks about **domain or package availability** for a name

## Two Modes

### Generate Mode
The user describes what their project does. The tool uses LLMs to brainstorm names, then checks each one against domains and package registries.

### Check Mode
The user already has candidate names. The tool skips LLM generation and goes straight to availability checking — no API keys needed for this mode.

## How to Use

Always pass `--json` so you can parse and present results cleanly.

### Check specific names (default)

```bash
available name1 name2 name3 --json
```

### Generate names

```bash
available --generate "description of the project" --json
```

### Important: use defaults unless the user asks otherwise

The tool checks 4 TLDs (.com, .dev, .io, .app) and 10 package registries by default. These defaults give the best results — do NOT pass `--tlds`, `--registries`, or `--models` unless the user specifically asks to customize them. Passing these flags limits the check to only what you specify, which produces worse results.

Optional flags (only if the user requests):
- `-a` / `--all` — check all ~130 common TLDs and all ~30 registries
- `--all-tlds` — check all ~130 common TLDs instead of the default 4
- `--all-registries` — check all ~30 registries instead of the default 10
- `--languages rust,python` — filter registries by language
- `--max-names 30` — generate more candidates (default: 20)
- `--tlds com,dev,io,org` — override which TLDs to check
- `--registries npm,crates.io` — override which registries to check (limits to ONLY these)
- `--models claude-opus-4-6` — pick specific LLM models
- `-v` / `--verbose` — show per-domain and per-registry detail
- `--free` — in verbose mode, only show available entries (hides taken)
- `--maybe` — in verbose mode, also show parked/unreachable domains (potentially acquirable)

## Parsing the JSON Output

The tool outputs structured JSON to stdout (progress/warnings go to stderr):

```json
{
  "results": [
    {
      "name": "rushq",
      "score": 0.8,
      "suggested_by": ["claude-opus-4-6"],
      "domains": {
        "available": 3,
        "registered": 0,
        "unknown": 0,
        "total": 3,
        "details": [
          {"domain": "rushq.com", "available": "available"},
          {"domain": "rushq.dev", "available": "available"},
          {"domain": "rushq.io", "available": "available"}
        ]
      },
      "packages": {
        "available": 9,
        "taken": 1,
        "unknown": 0,
        "total": 10,
        "details": [
          {"registry": "crates.io", "available": "available"},
          {"registry": "npm", "available": "taken"}
        ]
      }
    }
  ],
  "models_used": ["claude-opus-4-6"],
  "errors": []
}
```

Key fields:
- **`score`**: 0.0–1.0 availability score (higher is better). Weights: .com = 30%, other TLDs = 10% each, package registries = 50% split evenly.
- **`domains.details[].available`**: `"available"`, `"registered"`, or `"unknown"`
- **`packages.details[].available`**: `"available"`, `"taken"`, or `"unknown"`
- **`errors`**: Per-model generation failures (only in generate mode)

## Presenting Results

After running the command and parsing JSON, present results to the user in a clear summary. Focus on the top candidates (score >= 0.7) and highlight what's actually available vs taken.

### Formatting guide

For each top result, show:
1. The name and its score as a percentage
2. Domain availability — which TLDs are free vs taken
3. Package registry availability — summary count plus any notable conflicts (e.g., npm taken)
4. Which models suggested it (in generate mode)

Group results into tiers:
- **Fully available** (score >= 0.9): all or nearly all domains and registries clear
- **Mostly available** (score 0.7–0.9): some conflicts but still viable
- **Partially available** (score < 0.7): mention briefly, note key conflicts

If the user asked about specific names, give a direct answer for each: available or not, and where the conflicts are.

### Errors

If `errors` is non-empty, mention which models failed but don't make it the focus — the results from other models are still valid. If all models fail in generate mode, tell the user to check their API keys.

## Tips

- The tool needs at least one LLM API key set (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GOOGLE_API_KEY`, or `XAI_API_KEY`) for generate mode. Check mode works without any keys.
- Checking is the default mode — just pass names directly. Use `--generate` when the user wants AI-generated name suggestions.
- Results are sorted by score (best first), so the top entries are always the strongest candidates.
- If the binary isn't found, suggest: `cargo install --git https://github.com/brad/available` (adjust URL as needed).
