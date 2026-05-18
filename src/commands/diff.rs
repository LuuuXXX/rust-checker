use crate::cli::DiffArgs;
use anyhow::Result;

pub fn run(args: DiffArgs) -> Result<()> {
    let history = crate::history::load_history()?;
    if history.len() < 2 {
        println!("Not enough history entries to diff (need at least 2).");
        return Ok(());
    }

    let count = args.last.unwrap_or(2);
    let entries: Vec<_> = history.iter().rev().take(count).collect();

    if entries.len() < 2 {
        println!("Not enough entries.");
        return Ok(());
    }

    let newer = &entries[0];
    let older = &entries[1];

    println!("## Diff: {} vs {}", newer.timestamp, older.timestamp);
    println!();

    for new_result in &newer.tool_results {
        if let Some(old_result) = older
            .tool_results
            .iter()
            .find(|r| r.name == new_result.name)
        {
            let new_status = format!("{:?}", new_result.status);
            let old_status = format!("{:?}", old_result.status);
            if new_status != old_status {
                println!(
                    "| {} | {} → {} |",
                    new_result.name, old_status, new_status
                );
            }
        }
    }

    Ok(())
}
