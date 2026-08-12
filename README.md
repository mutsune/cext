# chrome-ext-manager (`cext`)

A CLI tool that manages your private/unpacked Chrome extensions as git repositories.

Chrome has no way to sync extensions you load unpacked. `cext` keeps each one as a git clone in a
single directory, so you can export the whole set as a plain URL list and restore it on another
machine with one command.

## How it works

Every saved extension is one git repository under the storage directory:

```
$HOME/Library/Application Support/Google/private extensions/
├── my-extension/       # git clone of https://github.com/someone/my-extension.git
└── another-extension/  # git clone of https://github.com/other/another-extension.git
```

There is no metadata file to keep in sync: `cext list` recovers each URL by reading the folder's
`git remote get-url origin`. Pass `--dir <DIR>` to any command to use a different storage
directory.

## Requirements

- `git` on your PATH — `add`, `list`, and `import` shell out to it
- Rust 1.85+ (only to build from source)
- The default storage path assumes macOS; other platforms work via `--dir`

## Install

```bash
cargo install --path .
```

Or build and place the binary yourself:

```bash
cargo build --release
cp target/release/cext /usr/local/bin/cext
```

## Commands

| Command | What it does |
| --- | --- |
| `cext add <url> [--name <name>]` | Clone an extension from a git remote URL |
| `cext list [--output <file>]` | Print saved extensions as a URL list |
| `cext import <file>` | Clone every extension in a URL list file |
| `cext remove <name> [--yes]` | Delete a saved extension |

Every command also accepts `--dir <DIR>`. Run `cext <command> --help` for full details.

### `add` — save an extension

```bash
cext add https://github.com/someone/my-extension.git
cext add git@github.com:someone/my-extension.git --name my-ext
```

Without `--name`, this is the same as running `git clone <url>` inside the storage directory — git
picks the folder name. With `--name`, you choose it. Either way, an extension that is already
saved is skipped rather than re-cloned, so re-running `add` is safe.

### `list` — export the saved set

```bash
cext list
```

Prints one remote URL per line to stdout:

```
https://github.com/someone/my-extension.git
https://github.com/other/another-extension.git
```

A folder without an `origin` remote is skipped with a warning. Use `--output` to write to a file
instead, which produces the exact format `import` reads:

```bash
cext list --output extensions.txt
```

### `import` — restore a saved set

```bash
cext import extensions.txt
```

Reads one URL per line; blank lines and `#` comments are ignored. Extensions that are already
saved are skipped, so only missing ones get cloned:

```
# Work extensions
https://github.com/someone/my-extension.git
https://github.com/other/another-extension.git
```

### `remove` — delete an extension

```bash
cext remove my-extension
cext remove my-extension --yes
```

`name` is the folder name under the storage directory. A confirmation prompt is shown unless you
pass `-y` / `--yes`.

## Syncing across machines

```bash
# on the machine that has the extensions
cext list --output extensions.txt

# on the other machine
cext import extensions.txt
```

Commit `extensions.txt` to a dotfiles repo and the set stays reproducible.

## Loading into Chrome

`cext` only clones and stores the extension files — Chrome still has to load them:

1. Open `chrome://extensions`
2. Turn on **Developer mode**
3. Click **Load unpacked** and select the extension's folder under the storage directory

## Development

```
src/
├── main.rs   # entry point, command dispatch
├── cli.rs    # clap subcommand definitions
└── ops.rs    # add / list / import / remove implementations
```

Built with [clap](https://docs.rs/clap/) (derive API) and [anyhow](https://docs.rs/anyhow/). All
git work is done by invoking the system `git` binary as a subprocess rather than linking a git
library.

```bash
cargo test
```
