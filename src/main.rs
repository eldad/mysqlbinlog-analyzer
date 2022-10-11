use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, io::stdin, fmt::Display};

enum BinlogOperation {
    Delete,
    Update,
    Insert,
}

struct BinlogRecord {
    table_name: String,
    op: BinlogOperation,
}

#[derive(Default, Clone, Copy)]
struct TableStats {
    deletes: usize,
    inserts: usize,
    updates: usize,
}

impl Display for TableStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} deletes, {} inserts, {} updates", self.deletes, self.inserts, self.updates)
    }
}

impl TryFrom<&str> for BinlogRecord {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        lazy_static! {
            static ref RE_DELETE: Regex = Regex::new(r"### DELETE FROM `[^`]+`\.`([^`]+)`$")
                .expect("DELETE regex is invalid");
            static ref RE_UPDATE: Regex =
                Regex::new(r"### UPDATE `([^`]+)`\.`([^`]+)`$").expect("UPDATE regex is invalid");
            static ref RE_INSERT: Regex = Regex::new(r"### INSERT INTO `[^`]+`\.`([^`]+)`$")
                .expect("INSERT regex is invalid");
        }

        if let Some(captures) = RE_DELETE.captures(value) {
            if let Some(table_name) = captures.get(1) {
                return Ok(BinlogRecord {
                    table_name: table_name.as_str().to_owned(),
                    op: BinlogOperation::Delete,
                });
            }
        }

        if let Some(captures) = RE_INSERT.captures(value) {
            if let Some(table_name) = captures.get(1) {
                return Ok(BinlogRecord {
                    table_name: table_name.as_str().to_owned(),
                    op: BinlogOperation::Insert,
                });
            }
        }

        if let Some(captures) = RE_UPDATE.captures(value) {
            if let Some(table_name) = captures.get(1) {
                return Ok(BinlogRecord {
                    table_name: table_name.as_str().to_owned(),
                    op: BinlogOperation::Update,
                });
            }
        }

        Err(anyhow::anyhow!("not a IUD operation record"))
    }
}

fn main() -> anyhow::Result<()> {
    let binlog_records: Vec<BinlogRecord> = stdin()
        .lines()
        .filter_map(Result::ok)
        .map(|line| {
            let line_ref: &str = &line;
            line_ref.try_into()
        })
        .filter_map(Result::ok)
        .collect();

    println!("{} records parsed", binlog_records.len());

    let table_stats: HashMap<String, TableStats> =
        binlog_records
            .into_iter()
            .fold(HashMap::new(), |mut acc, record| {
                {
                    let table_name = record.table_name;
                    let mut stats: TableStats =
                        acc.get_mut(&table_name).map(|x| *x).unwrap_or_default();
                    match record.op {
                        BinlogOperation::Delete => stats.deletes += 1,
                        BinlogOperation::Update => stats.updates += 1,
                        BinlogOperation::Insert => stats.inserts += 1,
                    }
                    acc.insert(table_name, stats);
                }
                acc
            });

    table_stats
        .iter()
        .for_each(|(table_name, stats)| println!("{table_name} {stats}"));

    Ok(())
}
