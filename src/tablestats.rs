use std::{collections::HashMap, fmt::Display};

use crate::binlog::{BinlogOperation, BinlogRecord};

#[derive(Default, Clone, Copy)]
pub struct TableStats {
    pub deletes: usize,
    pub inserts: usize,
    pub updates: usize,
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

pub fn tablestats_from_binlog_records(binlog_records: &[BinlogRecord]) -> HashMap<(&str, &str), TableStats> {
    binlog_records.iter().fold(HashMap::new(), |mut acc, record| {
        {
            let table_name: &str = &record.table_name;
            let schema_name: &str = &record.schema_name;
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
    })
}
