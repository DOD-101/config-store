use core::panic;
use std::fmt::Write;
use std::process::exit;

use clap::{Parser, Subcommand};
use rusqlite::{Connection, Result};

#[derive(Debug)]
struct Value {
    id: i32,
    name: String,
    value: String,
    alternate: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("{:#?}", args);

    let path = "test.db";
    let connection =
        Connection::open(path).unwrap_or_else(|_| panic!("Failed to open sqlite3 DB at {}", path));

    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS data (
                id INTEGER PRIMARY KEY,
                name TEXT,
                value TEXT,
                alternate TEXT
            );",
            (),
        )
        .expect("Failed to create values TABLE");

    let result = match args.command {
        Action::Set {
            name,
            value,
            alternate,
            change_only,
        } => set_cmd(&connection, name, value, alternate, change_only),
        Action::Get {
            name,
            value_only,
            alternate_only,
        } => get_cmd(&connection, name),
        Action::Toggle { name } => toggle_cmd(&connection, name),
        Action::Delete { name } => delete_cmd(&connection, name),
        Action::Check { name } => exists_cmd(&connection, name),
        Action::List => list_cmd(&connection),
    }?;

    println!("{}", result);

    Ok(())
}

#[derive(Debug, Parser)]
struct Args {
    /// What you want to do
    #[command(subcommand)]
    command: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// Set / Change a value & it's alternate
    Set {
        /// The name of the value to set
        name: String,
        /// The value
        #[arg(short, long)]
        value: Option<String>,
        /// The alternate
        #[arg(short, long)]
        alternate: Option<String>,
        /// Only change an entry, don't create a new one
        #[arg(short, long)]
        change_only: bool,
    },
    /// Get a value & it's alternate
    Get {
        /// The name of the entry to get
        name: String,
        /// Only get the value
        #[arg(short, long)]
        value_only: bool,
        /// Only get the alternate
        #[arg(short, long)]
        alternate_only: bool,
    },
    /// Toggle a entry between it's value & it's alternate
    Toggle {
        /// The name of the entry to toggle
        name: String,
    },
    /// Delete an entry
    Delete {
        /// The name of the entry to delete
        name: String,
    },
    /// Check if an entry exists
    Check { name: String },
    /// List the contents of the db
    List,
}

fn select(connection: &Connection, name: &str) -> Result<Value, rusqlite::Error> {
    connection.query_row("SELECT * FROM data WHERE name = ?", [name], |row| {
        Ok(Value {
            id: row.get(0)?,
            name: row.get(1)?,
            value: row.get(2)?,
            alternate: row.get(3)?,
        })
    })
}

fn exists(connection: &Connection, name: &str) -> Result<bool, rusqlite::Error> {
    connection
        .prepare("SELECT name FROM data WHERE name = ?")?
        .exists([name])
}

fn new(
    connection: &Connection,
    name: String,
    value: String,
    alternate: String,
) -> Result<String, rusqlite::Error> {
    connection.execute(
        "INSERT INTO data (name, value, alternate) VALUES (?1, ?2, ?3)",
        [name, value, alternate],
    )?;

    Ok(String::from("Ok"))
}

fn exists_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    match exists(connection, &name) {
        Ok(b) => Ok(b.to_string()),
        Err(e) => Err(e),
    }
}

fn delete_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    connection.execute("DELETE FROM data WHERE name = ?", [name])?;

    Ok("Ok".to_string())
}

fn get_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    let value = select(connection, &name)?;

    Ok(value.name)
}

fn set_cmd(
    connection: &Connection,
    name: String,
    new_value: Option<String>,
    new_alternate: Option<String>,
    change_only: bool,
) -> Result<String, rusqlite::Error> {
    if exists(connection, &name)? {
        let value = select(connection, &name)?;

        connection.execute(
            "UPDATE data SET value = ?, alternate = ? WHERE name = ?",
            [
                new_value.unwrap_or(value.value.clone()),
                new_alternate.unwrap_or(value.alternate.clone()),
                name,
            ],
        )?;

        Ok("Ok".to_string())
    } else if !change_only {
        new(
            connection,
            name,
            new_value.unwrap_or_default(),
            new_alternate.unwrap_or_default(),
        )
    } else {
        eprintln!("Trying to change a value that doesn't exist.");
        exit(1)
    }
}

fn toggle_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    let value = select(connection, &name)?;

    connection.execute(
        "UPDATE data SET value = ?, alternate = ? WHERE name = ?",
        [value.alternate, value.value.clone(), value.name],
    )?;

    Ok(value.value)
}

fn list_cmd(connection: &Connection) -> Result<String, rusqlite::Error> {
    Ok(connection
        .prepare("SELECT * FROM data")?
        .query_map([], |row| {
            Ok(Value {
                id: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                alternate: row.get(3)?,
            })
        })?
        .fold(String::new(), |mut acc, e| {
            writeln!(acc, "{:?}", e.unwrap()).unwrap();
            acc
        }))
}
