## cwctl - Console Management Plugin for Class Widgets 2

cwctl is a Rust command-line tool for managing Class Widgets 2 plugins.

> [!CAUTION]
> The project is under active development. Plugin metadata management is
> available; plugin file installation and `.cwplugin` packaging are not yet
> complete.

cwctl stores a local plugin manifest and can submit plugin URLs to an aria2
JSON-RPC endpoint. Class Widgets 2 integration and bundled aria2 packaging are
planned, but are not part of the current repository.

### Features

- Plugin metadata install, list, info, update, and uninstall commands
- Atomic JSON manifest writes for local plugin state
- Optional aria2 JSON-RPC download submission
- Automated Rust formatting, tests, linting, and release builds in CI

### Architecture

```
+----------------------------------------------------------+
|                    Class Widgets 2                          |
|  +----------------------------------------------------+   |
|  |                   cwctl (.cwplugin)                 |   |
|  |  +-------------+    +----------------------------+  |   |
|  |  | Rust CLI    |    |   JSON‑RPC Client          |  |   |
|  |  |             |───▶| (aria2 RPC calls)          |  |   |
|  |  +-------------+    +------------┬---------------+  |   |
|  +----------------------------------┼-------------------+   |
|                                     |                       |
+-------------------------------------┼-----------------------+
                                      |
                                      v
                             +-----------------+
                             |  aria2‑next     |
                             | (downloader)    |
                             +-----------------+
```

The current binary runs as a standalone CLI. Commands that include a download
URL are translated into JSON-RPC requests sent to an externally running aria2
instance.

### Technology Stack

| Component       | Technology       | Purpose                         |
|-----------------|------------------|---------------------------------|
| CLI             | Rust standard library | Argument parsing and output |
| Download core   | aria2               | Multi-protocol downloader      |
| RPC protocol    | JSON‑RPC 2.0     | Standard aria2 interface        |
| CI              | GitHub Actions   | Format, test, lint, build      |

### Installation

#### From source

The project publishes `.cwplugin` and `.zip` packages from version tags.
Build the Rust binary from source with the commands below.

### Manual (for development)

```bash
git clone https://github.com/Purrbyte-zdy/cwctl
cd cwctl
cargo build --release
```

### Usage

Run `cwctl --help` to display the available commands.
Run `cwctl shell` to open the reedline-powered interactive console. It
provides completion for the built-in commands and persists history at
`~/.local/share/cwctl/history`; enter `exit` or `quit` to leave it.

Successful commands return exit code `0`. Invalid arguments, unavailable
plugins, and failed aria2 requests return exit code `2` and write the error to
stderr.

### Core Commands

```bash
# Install a plugin
cwctl plugin install <plugin-name> [--version <version>] [--url <source>]

# List installed plugins
cwctl plugin list

# Update a plugin
cwctl plugin update <plugin-name>

# Remove a plugin
cwctl plugin uninstall <plugin-name>

# Show plugin details
cwctl plugin info <plugin-name>
```

### Building

#### Local Build

Install the stable Rust toolchain, then run:

```bash
cargo build --release
cargo test
```

The resulting executable is `target/release/cwctl`.

The local plugin manifest is stored at `~/.local/share/cwctl/plugins.json` by
default. Set `CWCTL_HOME` to use another data directory during development or
testing.

`plugin install` records validated plugin metadata in this manifest, while
`plugin update` marks the installed version as `latest`. When a URL is
provided, installation queues the URL in aria2; it does not wait for or verify
the downloaded plugin file yet.

When `plugin install` receives `--url`, it submits the URL to aria2 through
JSON-RPC. The endpoint defaults to
`http://127.0.0.1:6800/jsonrpc`; override it with `CWCTL_ARIA2_RPC` and set
`CWCTL_ARIA2_SECRET` when aria2 requires a secret token.

Every push to `main` and pull request runs `.github/workflows/ci.yml`, which
checks formatting, tests, Clippy, and the release build. Pushing a `v*.*.*`
tag runs `.github/workflows/release.yml`; it installs `class-widgets-sdk` with
uv and creates both `.cwplugin` and `.zip` artifacts with `cw-plugin-pack`.

### Project Structure

```
cwctl/
├── .github/workflows/ci.yml  # Continuous integration
├── src/main.rs               # CLI parsing and command execution
├── src/store.rs              # Local plugin manifest
├── src/aria2.rs              # aria2 JSON-RPC client
├── scripts/check_version.py  # Version consistency check
├── cwplugin.json             # Class Widgets 2 plugin manifest
├── main.py                   # Class Widgets 2 plugin entry point
├── pyproject.toml            # SDK packaging metadata
├── Cargo.toml                # Rust package manifest
└── README.md
```

### License

This project is licensed under the Apache License, Version 2.0. See the LICENSE file for details.

### Related Links

- Class Widgets 2 – Main application
- Class Widgets SDK – Plugin development SDK
- Plugin Template v2 – Official template
- Plugin Development Guide – Full documentation
- aria2 – Download engine

### Contributing

Issues and pull requests are welcome. Please ensure that:

- Code passes `cargo fmt` and `cargo clippy`
- Commit messages follow Conventional Commits
- New features include appropriate tests
