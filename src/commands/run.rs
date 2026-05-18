use crate::cli::RunArgs;
use crate::engine::runner::Runner;
use anyhow::Result;

pub fn run(args: RunArgs) -> Result<()> {
    let config = crate::config::load_config()?;
    let runner = Runner {
        config,
        format: args.format,
        ci_mode: args.ci,
    };
    let summary = runner.run()?;
    if args.ci {
        let json = serde_json::to_string_pretty(&summary.to_ci_output())?;
        println!("{}", json);
    }
    Ok(())
}
