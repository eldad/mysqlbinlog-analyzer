use lazy_static::lazy_static;
use regex::{Captures, Regex};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinlogOperation {
    Delete,
    Update,
    Insert,
}

pub struct BinlogRecord {
    pub table_name: String,
    pub schema_name: String,
    pub op: BinlogOperation,
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

        if !value.starts_with("### ") {
            return Err(anyhow::anyhow!("not a IUD operation record"));
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

impl BinlogOperation {
    pub(crate) fn is_opposite(&self, op: BinlogOperation) -> bool {
        matches!((self, op), (BinlogOperation::Insert, BinlogOperation::Delete) | (BinlogOperation::Delete, BinlogOperation::Insert))
    }
}
