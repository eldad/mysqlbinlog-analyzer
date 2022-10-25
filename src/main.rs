use clap::{Parser, Subcommand};

mod binlog;
mod binlog_empty_upates;
mod binlog_stats;
mod tablestats;

#[derive(Subcommand, Debug)]
#[command()]
enum Mode {
    Stats,
    EmptyUpdates {
        table_name: String,

        // List of ignored index (e.g. timestamp fields)
        #[arg(long)]
        ignore: Vec<usize>,

        /// ID column index
        #[arg(long)]
        id: Vec<usize>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, help_template = "
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}")]
struct Args {
    #[command(subcommand)]
    mode: Option<Mode>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.mode.unwrap_or(Mode::Stats) {
        Mode::Stats => binlog_stats::stats(),
        Mode::EmptyUpdates { table_name, ignore, id } => {
            if id.is_empty() {
                return Err(anyhow::anyhow!("must specify at least one ID column"));
            }
            binlog_empty_upates::empty_updates(&table_name, ignore, id)?
        }
    }

    Ok(())
}
