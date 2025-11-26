use std::path::PathBuf;

use clap::Parser;
use yellowstone_vixen::Error as VixenError;
use yellowstone_vixen_solana_snapshot_source::SolanaSnapshot;

/// Simple utility to count how many accounts are present inside a compressed Solana snapshot.
#[derive(Debug, Parser)]
#[command(name = "snapshot-account-counter")]
#[command(about = "Counts accounts by reading the unpacked snapshot account files.")]
struct Args {
    /// Path to the compressed snapshot tar.zst file.
    #[arg(value_name = "SNAPSHOT_PATH")]
    snapshot_path: PathBuf,

    /// Print the count per account file (in addition to the total).
    #[arg(long)]
    per_file: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Snapshot account counter failed: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), VixenError> {
    let args = Args::parse();
    let snapshot = SolanaSnapshot::unpack_compressed(args.snapshot_path.clone())?;

    let mut total = 0usize;
    for account_file in snapshot.account_files() {
        let count = account_file.account_count()?;

        if args.per_file {
            println!("{} -> {}", account_file.path().display(), count);
        }

        total += count;
    }

    println!("Total accounts: {total}");
    Ok(())
}
