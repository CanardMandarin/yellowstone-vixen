use std::path::PathBuf;

use clap::Parser;
use tokio::task::JoinSet;
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

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Snapshot account counter failed: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), VixenError> {
    let Args {
        snapshot_path,
        per_file,
    } = Args::parse();

    let snapshot = SolanaSnapshot::unpack_compressed(snapshot_path)?;

    let mut total = 0usize;

    let mut account_file_workers = JoinSet::new();

    for account_file in snapshot.account_files() {
        let account_file = account_file.clone();
        account_file_workers.spawn(async move {
            let blocking_task = tokio::task::spawn_blocking(move || -> Result<_, VixenError> {
                let path = account_file.path().to_path_buf();
                let count = account_file.account_count()?;
                Ok((path, count))
            });

            blocking_task.await.map_err(|err| {
                VixenError::Other(format!("Snapshot worker panicked: {err:?}").into())
            })?
        });

        if account_file_workers.len() >= 10 {
            if let Some(join_res) = account_file_workers.join_next().await {
                process_join_result(join_res, per_file, &mut total)?;
            }
        }
    }

    while let Some(join_res) = account_file_workers.join_next().await {
        process_join_result(join_res, per_file, &mut total)?;
    }

    println!("Total accounts: {total}");
    Ok(())
}

fn process_join_result(
    join_res: Result<Result<(PathBuf, usize), VixenError>, tokio::task::JoinError>,
    per_file: bool,
    total: &mut usize,
) -> Result<(), VixenError> {
    let (path, count) = join_res
        .map_err(|err| VixenError::Other(format!("Snapshot worker panicked: {err:?}").into()))??;

    *total += count;

    if per_file {
        println!("{}: {count}", path.display());
    }

    Ok(())
}
