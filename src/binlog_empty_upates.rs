use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::io::stdin;

use crate::binlog::BinlogOperation;
use crate::binlog::BinlogRecord;

fn next_operation_for_table(line: &str, table_name: &str) -> Option<BinlogOperation> {
    match BinlogRecord::try_from(line) {
        Ok(record) => match (record.table_name == table_name, record.op) {
            (true, BinlogOperation::Delete | BinlogOperation::Insert) => Some(record.op),
            _ => None,
        },
        Err(_) => None,
    }
}

fn extract_field_value(line: &str) -> Option<(usize, String)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"###   @([0-9]+)=(.*)$$").expect("DELETE regex is invalid");
    }

    RE.captures(line)
        .and_then(|captures| match (captures.get(1), (captures.get(2))) {
            (Some(index), Some(value)) => index
                .as_str()
                .parse::<usize>()
                .ok()
                .map(|index| (index, value.as_str().to_owned())),
            _ => None,
        })
}

struct RowChangeRecord {
    operation: BinlogOperation,
    fields: HashMap<usize, String>,
}

fn next_valid_stdlin_line() -> Option<String> {
    let mut stdin_lines = stdin().lines();

    loop {
        match stdin_lines.next() {
            None => return None,
            Some(Ok(v)) => return Some(v),
            _ => (),
        }
    }
}

pub fn empty_updates(table_name: &str, ignore: Vec<usize>, key_columns: Vec<usize>) -> anyhow::Result<()> {
    let mut records = HashMap::<Vec<String>, RowChangeRecord>::new();

    let mut empty_updates = 0;
    let mut non_empty_updates = 0;
    let mut inserts = 0;
    let mut deletes = 0;
    let mut lines = 0;

    loop {
        let line = next_valid_stdlin_line();
        if let Some(line) = line {
            lines += 1;

            if let Some(op) = next_operation_for_table(&line, table_name) {
                match op {
                    BinlogOperation::Delete => deletes += 1,
                    BinlogOperation::Insert => inserts += 1,
                    _ => (),
                }

                // Look for ### SET or WHERE marker
                let has_set_marker =
                    next_valid_stdlin_line().map_or_else(|| false, |line| line == "### SET" || line == "### WHERE"); // TODO: SET for INSERT, WHERE for DELETE
                if !has_set_marker {
                    return Err(anyhow::anyhow!("marker missing after operation found"));
                };

                // Get field values
                let mut field_values = HashMap::<usize, String>::new();
                loop {
                    let field_value = next_valid_stdlin_line().and_then(|line| extract_field_value(&line));

                    match field_value {
                        Some((k, v)) => {
                            if !ignore.contains(&k) {
                                field_values.insert(k, v);
                            }
                        }
                        None => break,
                    }
                }

                let mut key = Vec::<String>::new();
                for key_column in &key_columns {
                    let key_value = field_values
                        .remove(key_column)
                        .ok_or_else(|| anyhow::anyhow!("id column missing: {key_column}"))?;
                    key.push(key_value);
                }

                let current = records.get(&key);
                if let Some(current) = current {
                    if current.operation.is_opposite(op) {
                        // is empty?
                        if current.fields == field_values {
                            empty_updates += 1;
                        } else {
                            non_empty_updates += 1;
                        }

                        records.remove(&key);
                    }
                } else {
                    records.insert(
                        key,
                        RowChangeRecord {
                            fields: field_values,
                            operation: op,
                        },
                    );
                }
            }
        } else {
            break;
        }
    }

    eprintln!("lines {lines}, inserts {inserts}, deletes {deletes}, empty updates: {empty_updates}, other updates: {non_empty_updates} (remaining {})", records.len());

    Ok(())
}
