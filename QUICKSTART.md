# Quick Start

Welcome! Let me show you how to get `jan` running in about 2 minutes.

## Install

```bash
cargo install janus
```

That's it. Now you have `jan` in your PATH.

## Your First Sync

Let's say you want to back up your documents:

```bash
jan ~/documents /backup/documents
```

Janus will scan both directories, show you what's different, and ask if you want to proceed. Press `y` and it syncs.

## Common Patterns

**Dry run first** (see what would happen):
```bash
jan ~/photos /backup/photos -n
```

**Mirror mode** (delete files in destination that aren't in source):
```bash
jan ~/photos /backup/photos -d
```

**Skip the prompt** (for scripts):
```bash
jan ~/data /backup/data -y
```

**Quiet mode** (no progress bars):
```bash
jan ~/data /backup/data -q
```

**Combine flags** (quiet + delete + yes):
```bash
jan ~/data /backup/data -qdy
```

**Limit threads** (helpful for network drives):
```bash
jan ~/local /mnt/nas/backup -j 4
```

## The Magic Moment

Here's where Janus shines. Rename a large file:

```bash
mv ~/videos/trip.mp4 ~/videos/trip-2024.mp4
```

Now sync:

```bash
jan ~/videos /backup/videos
```

It completes instantly. Janus recognized the content and just updated the path. No 5GB copy, no waiting. That's what I built this for.

## For Scripts

Cron job example:

```bash
# Runs daily at 2 AM, quiet mode, deletes extras, no prompt
0 2 * * * jan /home/data /backup/data -qdy
```

The exit code is 0 on success, non-zero on failure, standard Unix behavior.

## Getting Help

```bash
jan --help        # see all options
jan --version     # check your version
```

## What's Next?

You now know enough to use Janus effectively. The [full README](README.md) has more details on how it works internally.

Have questions? Open an issue on [GitHub](https://github.com/procoperr/janus/issues). I read them all.

That's it. Enjoy syncing.
