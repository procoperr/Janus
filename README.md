# Janus `jan`

Beautifully fast, simple & reliable file syncing

[![CI](https://github.com/procoperr/janus/workflows/CI/badge.svg)](https://github.com/procoperr/janus/actions) [![Release](https://img.shields.io/github/v/release/procoperr/janus)](https://github.com/procoperr/janus/releases) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Table of contents

* [Overview](#overview)
* [Install](#install)
* [Quick start](#quick-start)
* [CLI reference](#cli-reference)
* [How it works](#how-it-works)
* [Examples](#examples)
* [Development](#development)
* [Contributing](#contributing)
* [License](#license)

## Overview

`jan` (Janus) is my industry standard file synchronization CLI focused on correctness and throughput. I wanted a tool that just works, every time, without the complexity.

Named after the Roman god of frames and transitions, Janus represents the elegant movement of files. It uses content‑addressed techniques to detect renames and avoid redundant copies while streaming data with constant memory usage.

**Design goals**

> *Content correctness, predictable performance, and low memory footprint.*

* Content‑addressed rename detection (no copy if content unchanged)
* BLAKE3 hashing for speed and parallelism
* Streaming transfer; memory usage independent of file size
* Multi‑threaded hashing and I/O
* Minimal, scriptable CLI for automation and CI

## Install

```bash
cargo install janus
```

Or grab a [pre-built binary](https://github.com/procoperr/janus/releases).

## Quick start

```bash
# one‑time sync
jan /path/to/source/ /path/to/dest/

# dry run
jan /src/ /dst/ -n

# delete files in destination that no longer exist in source
jan /src/ /dst/ -d

# non‑interactive delete
jan /src/ /dst/ -dy

# use 4 worker threads
jan /src/ /dst/ -j 4

# quiet mode
jan /src /dst -qy
```


## CLI reference

```
Usage: jan [OPTIONS] <SOURCE> <DEST>

Arguments:
  <SOURCE>  Source directory
  <DEST>    Destination directory

Options:
  -n, --dry-run        Show changes without applying
  -d, --delete         Delete files in dest not in source
  -y                   Skip confirmation prompt
  -q, --quiet          No progress output
  -v, --verbose        Verbose output
  -j, --threads N      Number of threads (default: CPU count)
  -h, --help           Print help
  -V, --version        Print version
```

## How it works

Janus scans both directories in parallel, computing BLAKE3 content hashes for every file. These hashes create a content-addressed index, think of it as a fingerprint database where we can instantly recognize files even if they've moved or been renamed.

When you rename a 5GB file, Janus sees the content is identical and simply updates the path in the destination. No copying, no waiting. That's the kind of intelligence I wanted in a sync tool.

The streaming architecture means we process files in chunks. Whether it's 1KB or 100GB, memory usage stays constant.

## Examples

### Your first sync

```bash
jan ~/photos /backup/photos
```

That's it. Janus will show you what it's going to do and ask for confirmation.

### Detecting renames

```bash
# You rename a large file
mv ~/videos/vacation.mp4 ~/videos/vacation-2024.mp4

# Janus detects the rename instantly
jan ~/videos /backup/videos
```

The sync completes in milliseconds instead of re-copying gigabytes.

### Automated backups

```bash
# In your cron job
0 2 * * * jan /home/user/data /backup/data -qdy
```

The `-qdy` flags make it quiet, delete extras, and skip prompts. Perfect for automation.

### Network drives

```bash
# Reduce threads to avoid overwhelming the network
jan ~/local /mnt/nas/backup -j 4
```

## Development

I welcome contributions. The code is organized simply:

* `core.rs` - scanning, diffing, syncing
* `hash.rs` - BLAKE3/SHA-256 abstraction
* `io.rs` - streaming file operations
* `progress.rs` - progress reporting

```bash
git clone https://github.com/procoperr/janus.git
cd janus
./scripts/setup-dev.sh
make test
```

Commits follow [Conventional Commits](https://conventionalcommits.org). Releases are automatic, just merge to master.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

The goal is simple: keep it beautifully fast, simple & reliable.

## License

[MIT](LICENSE)

Built with care & Rust.
