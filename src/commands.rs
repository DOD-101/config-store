use rusqlite::{Connection, Result};
use std::{fmt::Write, process::exit};

#[derive(Debug)]
struct Value {
    id: i32,
    name: String,
    value: String,
    alternate: String,
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

pub fn exists_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    match exists(connection, &name) {
        Ok(b) => Ok(b.to_string()),
        Err(e) => Err(e),
    }
}

pub fn delete_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    connection.execute("DELETE FROM data WHERE name = ?", [name])?;

    Ok("Ok".to_string())
}

pub fn get_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    let value = select(connection, &name)?;

    Ok(value.name)
}

pub fn set_cmd(
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

pub fn toggle_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    let value = select(connection, &name)?;

    connection.execute(
        "UPDATE data SET value = ?, alternate = ? WHERE name = ?",
        [value.alternate, value.value.clone(), value.name],
    )?;

    Ok(value.value)
}

pub fn list_cmd(connection: &Connection) -> Result<String, rusqlite::Error> {
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

pub fn drop_cmd(connection: &Connection) -> Result<String, rusqlite::Error> {
    connection.execute("DROP TABLE data", [])?;

    Ok("Ok".to_string())
}
