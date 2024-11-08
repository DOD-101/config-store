//! Module containing all the functions called by different (sub-)commands
//!
//! Each function ending in `_cmd` is mapped to one [crate::Action] and is only to be used by that
//! one [crate::Action].
//!
//! Additionally this module contains some helper functions and the [Entry] struct.
use clap::CommandFactory;
use rusqlite::{Connection, Result};
use std::{fmt::Write, io::Cursor, process::exit};

// TODO: Should this be moved?
/// A struct representing an entry in the db
#[derive(Debug)]
struct Entry {
    /// The id in the db
    _id: i32,
    /// The identifier set & accessed by users
    name: String,
    /// The value of the entry
    value: String,
    /// An additional value that can be toggle to
    ///
    /// This is particularly useful for true / false toggles
    alternate: String,
}

/// Helper function too get an [Entry] from the db
///
/// Since it uses [rusqlite::Connection::query_row] it will only ever return the first match.
///
/// Having multiple different entries with the same name is not supported.
fn select(connection: &Connection, name: &str) -> Result<Entry, rusqlite::Error> {
    connection.query_row("SELECT * FROM data WHERE name = ?", [name], |row| {
        Ok(Entry {
            _id: row.get(0)?,
            name: row.get(1)?,
            value: row.get(2)?,
            alternate: row.get(3)?,
        })
    })
}

/// Helper function to check if an [Entry] exists
fn exists(connection: &Connection, name: &str) -> Result<bool, rusqlite::Error> {
    connection
        .prepare("SELECT name FROM data WHERE name = ?")?
        .exists([name])
}

/// Helper function to create a new [Entry]
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

/// Check for the existence of an [Entry] in the db
///
/// This is merely a wrapper around [exists], which is needed to convert from
/// [`Result<bool, rusqlite::Error>`] to [`Result<String, rusqlite::Error>`].
pub fn exists_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    match exists(connection, &name) {
        Ok(b) => Ok(b.to_string()),
        Err(e) => Err(e),
    }
}

/// Delete an [Entry] in the db
///
/// If the entry doesn't exist, this will do nothing.
pub fn delete_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    connection.execute("DELETE FROM data WHERE name = ?", [name])?;

    Ok("Ok".to_string())
}

/// Returns a value (and/or) alternate from the db
pub fn get_cmd(
    connection: &Connection,
    name: String,
    value_only: bool,
    alternate_only: bool,
) -> Result<String, rusqlite::Error> {
    let entry = select(connection, &name)?;

    if value_only {
        Ok(entry.value)
    } else if alternate_only {
        Ok(entry.alternate)
    } else {
        Ok(format!("{} {}", entry.value, entry.alternate))
    }
}

/// Create a new (if not `change_only`) [Entry] in the db or update an existing one
///
/// Will exit if `change_only == true` and [exists] returns false (aka. the value doesn't exist).
pub fn set_cmd(
    connection: &Connection,
    name: String,
    new_value: Option<String>,
    new_alternate: Option<String>,
    change_only: bool,
) -> Result<String, rusqlite::Error> {
    if exists(connection, &name)? {
        let entry = select(connection, &name)?;

        connection.execute(
            "UPDATE data SET value = ?, alternate = ? WHERE name = ?",
            [
                new_value.unwrap_or(entry.value.clone()),
                new_alternate.unwrap_or(entry.alternate.clone()),
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

/// Toggles an [Entry]'s value & alternate returning the new value
pub fn toggle_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    let entry = select(connection, &name)?;

    connection.execute(
        "UPDATE data SET value = ?, alternate = ? WHERE name = ?",
        [entry.alternate.clone(), entry.value, entry.name],
    )?;

    Ok(entry.alternate)
}

// TODO: The representation of the entries could be changed to make them easier to use from bash
// scripts
/// Lists all entries in the db
pub fn list_cmd(connection: &Connection) -> Result<String, rusqlite::Error> {
    Ok(connection
        .prepare("SELECT * FROM data")?
        .query_map([], |row| {
            Ok(Entry {
                _id: row.get(0)?,
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

/// Drops the `data` table deleting all entries.
///
/// This won't actually delete the file on disk.
pub fn drop_cmd(connection: &Connection) -> Result<String, rusqlite::Error> {
    connection.execute("DROP TABLE data", [])?;

    Ok("Ok".to_string())
}

/// Generates shell completion script
pub fn completions_cmd(shell: clap_complete::Shell) -> String {
    let mut cursor_vec: Vec<u8> = vec![];
    let mut cursor = Cursor::new(&mut cursor_vec);

    clap_complete::generate(
        shell,
        &mut crate::Args::command(),
        crate::Args::command().get_name(),
        &mut cursor,
    );

    String::from_utf8(cursor.get_ref().to_vec()).expect("Failed to generate completion String.")
}

#[cfg(test)]
mod test {
    use super::*;
    fn create_db() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
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

        connection
    }

    #[test]
    fn insert_and_drop() {
        let connection = create_db();

        new(
            &connection,
            "test1".to_string(),
            "value1".to_string(),
            "alternate1".to_string(),
        )
        .unwrap();

        assert_eq!(
            list_cmd(&connection).unwrap(),
            format!(
                "{:?}\n",
                Entry {
                    _id: 1,
                    name: "test1".to_string(),
                    value: "value1".to_string(),
                    alternate: "alternate1".to_string(),
                }
            )
        );

        drop_cmd(&connection).unwrap();
    }

    #[test]
    fn insert_and_exists() {
        let connection = create_db();

        assert_eq!(
            exists_cmd(&connection, "test1".to_string()).unwrap(),
            "false"
        );

        new(
            &connection,
            "test1".to_string(),
            "value1".to_string(),
            "alternate1".to_string(),
        )
        .unwrap();

        assert_eq!(
            exists_cmd(&connection, "test1".to_string()).unwrap(),
            "true"
        );
    }

    #[test]
    fn insert_and_get() {
        let connection = create_db();

        new(
            &connection,
            "test1".to_string(),
            "value1".to_string(),
            "alternate1".to_string(),
        )
        .unwrap();

        assert_eq!(
            get_cmd(&connection, "test1".to_string(), false, false).unwrap(),
            format!("{} {}", "value1", "alternate1")
        );
    }
}
