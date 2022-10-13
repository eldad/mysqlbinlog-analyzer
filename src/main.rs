use std::io::stdin;

mod binlog;
use binlog::*;

mod tablestats;
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

fn main() -> anyhow::Result<()> {
    let binlog_records = binlog_records_from_stdin();

    eprintln!("{} records parsed", binlog_records.len());

    let table_stats = tablestats_from_binlog_records(&binlog_records);

    println!("schema_name,table_name,inserts,updates,deletes");
    table_stats.iter().for_each(|(&(schema_name, table_name), &stats)| {
        println!(
            "{schema_name},{table_name},{},{},{}",
            stats.inserts, stats.updates, stats.deletes
        )
    });

    Ok(())
}
