# available

<p align="center">
  <img src="logos/available-logo-2.png" width="256" alt="available logo" />
</p>

Check project name availability across domains and package registries. Optionally generate candidates with LLMs.

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

Multiple keys means suggestions from multiple models. Checking names works without any keys.

## Usage

### Check names (default)

```
$ available aurora drift nexus
  [######----]  62%  aurora               .com[-] .dev[+] .io[+]  pkg: 7/10 available
  [########--]  78%  drift                .com[+] .dev[+] .io[-]  pkg: 9/10 available
  [####------]  40%  nexus                .com[-] .dev[-] .io[-]  pkg: 8/10 available
```

### Generate names

Use `--generate` to have LLMs brainstorm candidates:

```
$ available --generate "a fast task queue for Rust"
Generating names with: claude-opus-4-6, gpt-5.2
Checking 14 names...
  [########--]  80%  rushq                .com[+] .dev[+] .io[+]  pkg: 9/10 available
  [########--]  75%  taskforge            .com[-] .dev[+] .io[+]  pkg: 10/10 available
  [######----]  60%  quicktask            .com[-] .dev[+] .io[-]  pkg: 8/10 available
  ...
```

### Options

```
-a, --all                      Check all common TLDs (~130) and all registries (~30)
    --generate                 Generate names from a description instead of checking
    --models <MODELS>          Comma-separated model names (default: auto-detect from API keys)
    --tlds <TLDS>              Comma-separated TLDs to check (default: com,dev,io,app)
    --all-tlds                 Check all common TLDs (~130)
    --registries <REGISTRIES>  Comma-separated registry IDs (default: popular 10)
    --all-registries           Check all registries (~30)
    --languages <LANGUAGES>    Filter registries by language (e.g. rust,python,javascript)
    --stores <STORES>          Comma-separated app store IDs (default: app_store, google_play)
    --max-names <N>            Maximum names to generate (default: 20)
    --json                     JSON output
-v, --verbose                  Show per-domain and per-registry detail
    --free                     In verbose mode, only show available entries
    --maybe                    In verbose mode, also show parked/unreachable domains
```

### Verbose output

```
$ available aurora --verbose
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
available aurora --json
```

Returns structured JSON with per-name scores, domain details, and package registry results.

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
| .com domain | 25% |
| Other domains | 25% (split evenly) |
| Package registries | 50% (split evenly) |

When app stores are checked, domains get 40%, registries 40%, stores 20%.

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
