# available

<p align="center">
  <img src="logos/available-logo-2.png" width="256" alt="available logo" />
</p>

Find project names that are actually available. Generates candidates with LLMs, then checks domains and package registries in one shot.

Built on [caucus](https://github.com/bradleydwyer/caucus), [parked](https://github.com/bradleydwyer/parked), and [staked](https://github.com/bradleydwyer/staked). Also runs as an MCP server.

## Install

```bash
brew install bradleydwyer/tap/available
```

Or from source (Rust 1.85+):

```bash
cargo install --git https://github.com/bradleydwyer/available
```

### API keys

Name generation needs at least one LLM API key. Set them as environment variables or in a `.env` file:

```bash
ANTHROPIC_API_KEY=sk-...    # Claude
OPENAI_API_KEY=sk-...       # GPT
GOOGLE_API_KEY=...          # Gemini
XAI_API_KEY=...             # Grok
```

Multiple keys means suggestions from multiple models. The `--check` mode works without any keys.

## Usage

### Generate names

```
$ available "a fast task queue for Rust"
Generating names with: claude-opus-4-6, gpt-5.2
Checking 14 names...
  [########--]  80%  rushq                .com[+] .dev[+] .io[+]  pkg: 9/10 available
  [########--]  75%  taskforge            .com[-] .dev[+] .io[+]  pkg: 10/10 available
  [######----]  60%  quicktask            .com[-] .dev[+] .io[-]  pkg: 8/10 available
  ...
```

### Check specific names

No LLM needed. Just check names you already have:

```
$ available --check aurora,drift,nexus
  [######----]  62%  aurora               .com[-] .dev[+] .io[+]  pkg: 7/10 available
  [########--]  78%  drift                .com[+] .dev[+] .io[-]  pkg: 9/10 available
  [####------]  40%  nexus                .com[-] .dev[-] .io[-]  pkg: 8/10 available
```

### Options

```
    --check <NAMES>        Check specific names (comma-separated, no LLM needed)
    --models <MODELS>      Comma-separated model names (default: auto-detect from API keys)
    --tlds <TLDS>          Comma-separated TLDs to check (default: com,dev,io)
    --registries <IDS>     Comma-separated registry IDs (default: popular 10)
    --max-names <N>        Maximum names to generate (default: 20)
    --json                 JSON output
-v, --verbose              Show per-domain and per-registry detail
```

### Verbose output

```
$ available --check aurora --verbose
  [######----]  62%  aurora               .com[-] .dev[+] .io[+]  pkg: 7/10 available
         [-] aurora.com                registered
         [+] aurora.dev                available
         [+] aurora.io                 available
         [+] crates.io                 available
         [-] npm                       taken
         [+] PyPI                      available
         ...
```

### JSON output

```bash
available --check aurora --json
```

Returns structured JSON with per-name scores, domain details, and package registry results.

### MCP server

```bash
available mcp
```

Three tools over stdio:

| Tool | Description |
|------|-------------|
| `find_names` | Generate names via AI and check availability |
| `check_names` | Check specific names (up to 50) |
| `list_models` | Show configured LLM providers |

Claude Code config:

```json
{
  "mcpServers": {
    "available": {
      "command": "/path/to/available",
      "args": ["mcp"]
    }
  }
}
```

## Claude Code Skill

available includes a [skill](skill/SKILL.md) for Claude Code. Install it with [equip](https://github.com/bradleydwyer/equip):

```bash
equip install bradleydwyer/available
```

This lets Claude Code find and check project names directly when you ask for naming help.

## Scoring

Each name gets a 0-100% score based on availability:

| Component | Weight |
|-----------|--------|
| .com domain | 30% |
| .dev domain | 10% |
| .io domain | 10% |
| Package registries | 50% (split evenly) |

Available = full credit, unknown = half, taken = zero.

## License

MIT

## More Tools

**Naming & Availability**
- [parked](https://github.com/bradleydwyer/parked) — Domain availability checker (DNS → WHOIS → RDAP)
- [staked](https://github.com/bradleydwyer/staked) — Package registry name checker (npm, PyPI, crates.io + 19 more)
- [published](https://github.com/bradleydwyer/published) — App store name checker (App Store & Google Play)

**AI Tooling**
- [sloppy](https://github.com/bradleydwyer/sloppy) — AI prose/slop detector
- [caucus](https://github.com/bradleydwyer/caucus) — Multi-LLM consensus engine
- [nanaban](https://github.com/bradleydwyer/nanaban) — Gemini image generation CLI
- [equip](https://github.com/bradleydwyer/equip) — Cross-agent skill manager
