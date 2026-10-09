# duu

A small disk usage tool that lists folder sizes as a tree, largest first.

```
$ duu
62.5M ./
├─ 62.5M target/
├─ 32.0K .git/
├─  5.6K src/
├─  4.8K Cargo.lock
├─  122B Cargo.toml
└─    8B .gitignore
```

## Install

Requires a Rust toolchain that supports edition 2024 (Rust 1.85+).

```sh
cargo install --path .
```

Or build without installing:

```sh
cargo build --release
./target/release/duu
```

## Usage

```
duu [OPTIONS] [PATH]
```

`PATH` defaults to the current directory. If `PATH` is a file, `duu` prints just that file's size.

| Option | Description |
| --- | --- |
| `-d`, `--depth <N>` | Depth to display; `1` shows direct children only (default: `1`) |
| `-t`, `--threshold <SIZE>` | Hide entries smaller than `SIZE` (default: `0`, show all) |
| `-h` | Human-readable sizes, e.g. `5.6K`, `62.5M` (default) |
| `-k` | Sizes in KB |
| `-m` | Sizes in MB |
| `-g` | Sizes in GB |
| `--help` | Show help |

The unit flags (`-h`, `-k`, `-m`, `-g`) are mutually exclusive. With `-k`, `-m` or `-g`, sizes are rounded up to the next whole unit.

Note that `-h` selects human-readable sizes, as in `du`; use `--help` for help.

### Examples

Show two levels of the `src` directory:

```
$ duu -d 2 src
5.6K src/
└─ 5.6K main.rs
```

Show only entries of at least 1 MB (smaller ones are hidden but still counted in their parent's total):

```sh
duu -t 1M
```

`SIZE` is a number of bytes with an optional `K`, `M`, `G` or `T` suffix (1024-based, case-insensitive, optional trailing `B`); decimals such as `1.5G` are allowed.

Show sizes in megabytes:

```sh
duu -m ~/Downloads
```

## How sizes are counted

- Sizes are the sum of file lengths (apparent size), not allocated disk blocks, so they may differ from `du`.
- Symlinks are counted by the size of the link itself; their targets are not followed.
- Directories that cannot be read are reported on stderr and counted as 0.
