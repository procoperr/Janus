# Contributing

Thanks for your interest in making Janus better. I really appreciate it.

## Getting Started

First, fork and clone the repo:

```bash
git clone https://github.com/yourusername/janus.git
cd janus
./scripts/setup-dev.sh
```

The setup script installs everything you need. Commit message validation hooks, Rust toolchain components, and Bun for the development tools.

You'll need Rust 1.75+ and Bun installed. The script will check and let you know if anything's missing.

## Making Changes

Create a branch for your work:

```bash
git checkout -b feat/your-feature
```

Make your changes. A few things I care about:

**Keep it fast.** Performance is core to what Janus is. If your change affects hot paths, include benchmark results showing the impact.

**Keep it simple.** I'd rather have clear code than clever code. The next person reading it (probably me in 6 months) will thank you.

**Add tests.** New features need tests. Bug fixes should include a test showing the bug is fixed. Put unit tests in the same file under `#[cfg(test)]`, integration tests in `tests/`.

**Document public APIs.** Every public function and struct needs a doc comment. Include examples when it helps.

Before committing:

```bash
make format
make lint
make test
```

Or run it all at once:

```bash
make check
```

## Commit Messages

This project uses [Conventional Commits](https://conventionalcommits.org) for automatic versioning and releases. The format is:

```
type: brief description

optional longer explanation

optional footer
```

Common types:
- `feat` - new feature (triggers minor version bump)
- `fix` - bug fix (triggers patch version bump)  
- `docs` - documentation only
- `perf` - performance improvement
- `refactor` - code cleanup
- `test` - adding tests

For breaking changes, add `!` after the type:

```bash
git commit -m "feat!: change CLI flags"
```

Good examples:
```bash
git commit -m "feat: add exclude pattern support"
git commit -m "fix: handle symlinks correctly"
git commit -m "perf: optimize hash computation for small files"
```

The Git hooks will catch bad commit messages before you push.

## Opening a Pull Request

Push your branch and open a PR:

```bash
git push origin feat/your-feature
```

In the PR description, tell me:
- What you changed and why
- How you tested it
- Any concerns or questions

If it fixes an issue, mention it: `Fixes #123`

I'll review as soon as I can. I might ask questions or suggest changes. It's all part of making Janus better together.

## Code Style

I use standard Rust formatting (rustfmt) and clippy with warnings as errors. The pre-commit hooks run these automatically.

A few non-obvious preferences:

**Streaming over loading.** Don't load entire files into memory. Use buffered I/O.

**Explicit errors in libraries.** Use `thiserror` for the library, `anyhow` for the CLI binary.

**Comments explain why.** The code shows what you're doing. Comments should explain why you chose that approach.

## Running Benchmarks

If you're working on performance:

```bash
# Create baseline before your changes
cargo bench -- --save-baseline before

# Make your changes...

# Compare
cargo bench -- --baseline before
```

Criterion will show the performance delta. Small variations under 5% are normal noise.

## Questions?

If something's unclear, just ask. Open an issue or discussion on GitHub. I'd rather answer questions early than merge something that doesn't quite fit.

## Release Process

Don't worry about versioning or releases. When your PR merges to master, the CI automatically:
- Analyzes commit messages
- Determines version bump
- Builds binaries for all platforms
- Publishes to crates.io
- Creates GitHub release

You don't need to do anything manually.

## Philosophy

I want Janus to be beautifully fast, simple & reliable. That's the filter for every change. Does it make Janus faster, simpler, or more reliable? If yes, let's talk. If it adds complexity without clear benefit, probably not.

That said, I'm open to ideas. Sometimes the best features are ones I never thought of.

---

Thanks for contributing. Every improvement, no matter how small, makes Janus better for everyone.
