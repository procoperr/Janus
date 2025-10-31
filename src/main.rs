use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::process;

use janus::{diff_scans, scan_directory, sync_changes, SyncOptions};

#[derive(Parser)]
#[command(
    name = "jan",
    version,
    about = "Beautifully fast, simple & reliable file syncing"
)]
struct Cli {
    /// Source directory
    source: PathBuf,

    /// Destination directory
    dest: PathBuf,

    /// Dry run (show changes without applying)
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Delete files in dest not in source
    #[arg(short, long)]
    delete: bool,

    /// Skip confirmation prompt
    #[arg(short = 'y')]
    yes: bool,

    /// Quiet mode (no progress)
    #[arg(short, long)]
    quiet: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Number of threads (default: CPU count)
    #[arg(short = 'j', long)]
    threads: Option<usize>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(t) = cli.threads {
        rayon::ThreadPoolBuilder::new().num_threads(t).build_global()?;
    }

    if cli.verbose && !cli.quiet {
        println!("Scanning: {}", cli.source.display());
    }

    let src = scan_directory(&cli.source, None)?;
    let dst = scan_directory(&cli.dest, None)?;
    let diff = diff_scans(&src, &dst)?;

    let changes = diff.added.len() + diff.modified.len() + diff.renamed.len();
    if changes == 0 && (!cli.delete || diff.removed.is_empty()) {
        if !cli.quiet {
            println!("In sync");
        }
        return Ok(());
    }

    if !cli.quiet {
        println!(
            "Changes: {} copy, {} rename{}",
            diff.added.len() + diff.modified.len(),
            diff.renamed.len(),
            if cli.delete {
                format!(", {} delete", diff.removed.len())
            } else {
                String::new()
            }
        );
    }

    if cli.dry_run {
        return Ok(());
    }

    if !cli.yes && !cli.quiet {
        print!("Proceed? [y/N] ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok(());
        }
    }

    sync_changes(
        &cli.source,
        &cli.dest,
        &diff,
        &SyncOptions {
            delete_removed: cli.delete,
            preserve_timestamps: true,
            verify_after_copy: false,
        },
        None,
    )?;

    if !cli.quiet {
        println!("Done");
    }

    Ok(())
}
