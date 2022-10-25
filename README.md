# mysqlbinglog-analyzer

Basic MySQL binlog analyzer that parses the binlog verbose format and outputs insert/update/delete statistics for each table.

## Usage

> NOTE: The tool has limited functionality at the moment - I only implemented what I needed so far.

Pipe the output of `mysqlbinlog` (see below) to this tool.
It can generate simple stats or do an "empty updates" check (matching INSERTs and DELETEs).

```
mysqlbinlog-analyzer 0.1.0
Eldad Zack
Analyzes the MySQL binlog from the decoded text output

Usage: mysqlbinlog-analyzer [COMMAND]

Commands:
  stats
  empty-updates
  help           Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

### Empty Updates Check

Empty updates check only examines INSERTs and DELETEs to a certain table.

Use the `--id` parameter (column index number as seen in the binlog) to specify the match criteria for the records. `--id` supports composite indices.

Use the `--ignore` parameter to ignore fields such as automatically updated columns (e.g. "updated" timestamp column). multiple columns can be specified.

```
Usage: mysqlbinlog-analyzer empty-updates [OPTIONS] <TABLE_NAME>

Arguments:
  <TABLE_NAME>

Options:
      --ignore <IGNORE>
      --id <ID>          ID column index
  -h, --help             Print help information
```

## mysqlbinlog

Recommended invocation:

```
# Assuming environment variables are secure (note that they can be easily read with sufficient permissions), use `MYSQL_PWD` for the password.
# One can use a mysql configuration file if environment variables are deemed insecure.
# HINT: add -h <hostname> or -s <socket> as needed.
$ mysqlbinlog -u <username> -v --base64-output=decode-rows  --read-from-remote-server <binlog_filename>
```

## Tabular view

The CSV output can be displayed as a table by piping into `column`:

```shell
# `binlogoutput.txt` is the output of `mysqlbinlog -v`
$ <binlogoutput.txt mysqlbinlog-analyzer | column -s, -t
```
