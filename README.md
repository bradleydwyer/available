# available

AI-powered project name finder — generates names via LLMs and checks domain + package registry availability in one shot.

## About

When starting a new project, finding a name that's available across domains and package registries is tedious manual work. `available` combines [caucus](https://github.com/bradleydwyer/caucus) (multi-LLM generation), [domain-check](https://github.com/bradleydwyer/domain-check), and [pkg-check](https://github.com/bradleydwyer/pkg-check) into a single tool that generates name suggestions and checks their availability in one command.

It also runs as an MCP server, so AI assistants can find available project names directly.

## Installation

**Homebrew (macOS):**
```bash
brew install bradleydwyer/tap/available
```

**From source (requires Rust 1.85+):**
```bash
cargo install --git https://github.com/bradleydwyer/available
```

### API keys

At least one LLM API key is required for name generation:

```bash
export ANTHROPIC_API_KEY=sk-...    # Claude
export OPENAI_API_KEY=sk-...       # GPT
export GOOGLE_API_KEY=...          # Gemini
export XAI_API_KEY=...             # Grok
```

Set multiple keys to get suggestions from several models at once. The `--check` mode works without any API keys.

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

No LLM needed — just check names you already have in mind:

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

Exposes three tools over stdio:

| Tool | Description |
|------|-------------|
| `find_names` | Generate names via AI and check availability |
| `check_names` | Check specific names (up to 50) |
| `list_models` | Show configured LLM providers |

#### Claude Code configuration

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

## How scoring works

Each name gets a score from 0% to 100% based on availability:

| Component | Weight |
|-----------|--------|
| .com domain | 30% |
| .dev domain | 10% |
| .io domain | 10% |
| Package registries | 50% (split evenly across checked registries) |

Available = full credit, unknown = half credit, taken/registered = zero.

## License

available is licensed under the MIT license. See the [`LICENSE`](LICENSE) file for details.
