use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::{collections::HashMap, fmt::Display, io::stdin};

enum BinlogOperation {
    Delete,
    Update,
    Insert,
}

struct BinlogRecord {
    table_name: String,
    schema_name: String,
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
        write!(
            f,
            "{} deletes, {} inserts, {} updates",
            self.deletes, self.inserts, self.updates
        )
    }
}

fn binlog_record_from_capture(captures: Captures, op: BinlogOperation) -> anyhow::Result<BinlogRecord> {
    let schema_name: String = captures
        .get(1)
        .ok_or_else(|| anyhow::anyhow!("missing capture"))?
        .as_str()
        .to_owned();

    let table_name: String = captures
        .get(2)
        .ok_or_else(|| anyhow::anyhow!("missing capture"))?
        .as_str()
        .to_owned();

    Ok(BinlogRecord {
        table_name,
        schema_name,
        op,
    })
}

impl TryFrom<&str> for BinlogRecord {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        lazy_static! {
            static ref RE_DELETE: Regex =
                Regex::new(r"### DELETE FROM `([^`]+)`\.`([^`]+)`$").expect("DELETE regex is invalid");
            static ref RE_UPDATE: Regex =
                Regex::new(r"### UPDATE `([^`]+)`\.`([^`]+)`$").expect("UPDATE regex is invalid");
            static ref RE_INSERT: Regex =
                Regex::new(r"### INSERT INTO `([^`]+)`\.`([^`]+)`$").expect("INSERT regex is invalid");
        }

        if let Some(captures) = RE_DELETE.captures(value) {
            return binlog_record_from_capture(captures, BinlogOperation::Delete);
        }

        if let Some(captures) = RE_INSERT.captures(value) {
            return binlog_record_from_capture(captures, BinlogOperation::Insert);
        }

        if let Some(captures) = RE_UPDATE.captures(value) {
            return binlog_record_from_capture(captures, BinlogOperation::Update);
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

    let table_stats: HashMap<(String, String), TableStats> =
        binlog_records.into_iter().fold(HashMap::new(), |mut acc, record| {
            {
                let table_name = record.table_name;
                let schema_name = record.schema_name;
                let key = (schema_name, table_name);

                let mut stats: TableStats = acc.get_mut(&key).map(|x| *x).unwrap_or_default();
                match record.op {
                    BinlogOperation::Delete => stats.deletes += 1,
                    BinlogOperation::Update => stats.updates += 1,
                    BinlogOperation::Insert => stats.inserts += 1,
                }
                acc.insert(key, stats);
            }
            acc
        });

    table_stats
        .iter()
        .for_each(|((schema_name, table_name), stats)| println!("{schema_name}.{table_name}: {stats}"));

    Ok(())
}
