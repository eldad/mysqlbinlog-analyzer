use std::io::stdin;

use crate::{binlog::BinlogRecord, tablestats::tablestats_from_binlog_records};

fn binlog_records_from_stdin() -> Vec<BinlogRecord> {
    stdin()
        .lines()
        .filter_map(Result::ok)
        .map(|line| {
            let line_ref: &str = &line;
            line_ref.try_into()
        })
        .filter_map(Result::ok)
        .collect()
}

pub fn stats() {
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
}
