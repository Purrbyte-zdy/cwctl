## cwctl - Console Management Plugin for Class Widgets 2

cwctl is a console management plugin for Class Widgets 2. It provides a command‑line interface to manage the CW2 plugin ecosystem directly from your terminal.

> [!CAUTION]
> The plugin has not developed yet.

The plugin is written in Rust and uses reedline for a modern CLI experience. It communicates with aria2‑next via JSON‑RPC to enable multi‑threaded concurrent downloads of other plugins. Both cwctl and the bundled aria2‑next are compiled and packaged into a `.cwplugin` file during the GitHub Release action. Class Widgets 2 is responsible for installing and invoking cwctl, while all download heavy lifting is delegated to aria2‑next.

### Features

- Full‑featured console with syntax highlighting, auto‑completion, and command history (powered by reedline)
- Multi‑threaded plugin downloads using aria2‑next
- Native CW2 plugin format – follows the standard plugin specification
- Automated packaging via GitHub Actions – produces a ready‑to‑use .cwplugin file
- High performance and low resource footprint, written in Rust

### Architecture

```
+----------------------------------------------------------+
|                    Class Widgets 2                          |
|  +----------------------------------------------------+   |
|  |                   cwctl (.cwplugin)                 |   |
|  |  +-------------+    +----------------------------+  |   |
|  |  |  reedline   |    |   JSON‑RPC Client          |  |   |
|  |  | (CLI front) |───▶| (aria2‑next RPC calls)    |  |   |
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

CW2 loads the plugin and invokes its entry point. cwctl then starts the interactive CLI, and any download command is translated into JSON‑RPC requests sent to the aria2‑next subprocess.

### Technology Stack

| Component       | Technology       | Purpose                         |
|-----------------|------------------|---------------------------------|
| CLI framework   | reedline         | Readline‑alike for Rust         |
| Download core   | aria2‑next       | Multi‑protocol, multi‑thread    |
| RPC protocol    | JSON‑RPC 2.0     | Standard aria2 interface        |
| CI / CD         | GitHub Actions   | Build and package automation    |

### Installation

#### Via Class Widgets 2

1. ~~Download the latest .cwplugin file from the Releases page.~~
2. Install it through the CW2 plugin store or by dragging the file into the application.
3. CW2 will automatically recognise and load the plugin.

### Manual (for development)

```bash
git clone https://github.com/your-repo/cwctl
cd cwctl
cargo build --release
# Package manually (see Building section)
```

### Usage

After installation, cwctl can be launched from within Class Widgets 2. Once started, you will enter a dedicated console environment.

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

You can also type the manual `cwctl` to use it.

### Interactive Features

- Tab completion for commands and plugin names
- Command history (navigate with arrow keys)
- Syntax highlighting for better readability

### Building

#### Local Build

todo.

#### GitHub Release Action

The repository includes .github/workflows/release.yml. When a tag is pushed, the action will:

- Compile the Rust code for multiple platforms
- Produce the corresponding `.cwplugin` package
- Attach it to the GitHub Release

### Project Structure

```
cwctl/
├── plugin.json          # Plugin metadata (required by CW2)
├── __init__.py          # Python loader for CW2 integration
├── cwctl                # Compiled Rust binary
└── README.md
```

> [!Note]
> The `plugin.json` file must be present for CW2 to recognise the plugin. Its content follows the standard CW2 plugin specification.

### License

This project is licensed under the Apache License, Version 2.0. See the LICENSE file for details.

### Related Links

- Class Widgets 2 – Main application
- Class Widgets SDK – Plugin development SDK
- Plugin Template v2 – Official template
- Plugin Development Guide – Full documentation
- reedline – CLI library
- aria2‑next – Download engine (bundled separately)

### Contributing

Issues and pull requests are welcome. Please ensure that:

- Code passes `cargo fmt` and `cargo clippy`
- Commit messages follow Conventional Commits
- New features include appropriate tests
