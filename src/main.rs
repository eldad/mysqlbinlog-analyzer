use std::io::stdin;

use clap::{Parser, Subcommand};

mod binlog;
mod tablestats;

use binlog::*;
use tablestats::*;

fn binlog_records_from_stdin() -> Vec<BinlogRecord> {
    stdin()
        .lines()
        .filter_map(Result::ok)
        .into_iter()
        .map(|line| {
            let line_ref: &str = &line;
            line_ref.try_into()
        })
        .filter_map(Result::ok)
        .collect()
}
#[derive(Subcommand, Debug)]
#[command()]
enum Mode {
    Stats,
    EmptyUpdates,
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

fn stats(binlog_records: &[BinlogRecord]) {
    let table_stats = tablestats_from_binlog_records(&binlog_records);

    println!("schema_name,table_name,inserts,updates,deletes");
    table_stats.iter().for_each(|(&(schema_name, table_name), &stats)| {
        println!(
            "{schema_name},{table_name},{},{},{}",
            stats.inserts, stats.updates, stats.deletes
        )
    });
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let binlog_records = binlog_records_from_stdin();

    eprintln!("{} records parsed", binlog_records.len());

    match args.mode.unwrap_or(Mode::Stats) {
        Mode::Stats => stats(&binlog_records),
        Mode::EmptyUpdates => todo!(),
    }

    Ok(())
}
