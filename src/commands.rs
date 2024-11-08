use rusqlite::{Connection, Result};
use std::{fmt::Write, process::exit};

#[derive(Debug)]
struct Entry {
    id: i32,
    name: String,
    value: String,
    alternate: String,
}

fn select(connection: &Connection, name: &str) -> Result<Entry, rusqlite::Error> {
    connection.query_row("SELECT * FROM data WHERE name = ?", [name], |row| {
        Ok(Entry {
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

pub fn toggle_cmd(connection: &Connection, name: String) -> Result<String, rusqlite::Error> {
    let entry = select(connection, &name)?;

    connection.execute(
        "UPDATE data SET value = ?, alternate = ? WHERE name = ?",
        [entry.alternate, entry.value.clone(), entry.name],
    )?;

    Ok(entry.value)
}

pub fn list_cmd(connection: &Connection) -> Result<String, rusqlite::Error> {
    Ok(connection
        .prepare("SELECT * FROM data")?
        .query_map([], |row| {
            Ok(Entry {
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
                    id: 1,
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
