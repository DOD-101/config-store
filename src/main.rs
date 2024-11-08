//! config-store is a simple and lightweight key-value store designed for easy use from shell
//! scripts
//!
//! It uses a sqlite3 db in `/tmp/config-store.db` to save values. This means that all values persist
//! until reboot. Should `config-store.db` be deleted for any reason config-store will simply create
//! a new one on the next invocation. <b> Please note that this only applies to release builds. For
//! debug builds the db is located at `./test.db`. </b>
//!
//! See [commands] for more information on how individual commands work.
//! for a simple high level overview.
//!
use core::panic;

use clap::{Parser, Subcommand};
use rusqlite::{Connection, Result};

mod commands;

fn main() -> Result<()> {
    let args = Args::parse();

    #[cfg(debug_assertions)]
    let path = "test.db";

    #[cfg(not(debug_assertions))]
    let path = "/tmp/config-store.db";

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
        .expect("Failed to create data TABLE");

    let result = match args.command {
        Action::Set {
            name,
            value,
            alternate,
            change_only,
        } => commands::set_cmd(&connection, name, value, alternate, change_only),
        Action::Get {
            name,
            value_only,
            alternate_only,
        } => commands::get_cmd(&connection, name, value_only, alternate_only),
        Action::Toggle { name } => commands::toggle_cmd(&connection, name),
        Action::Delete { name } => commands::delete_cmd(&connection, name),
        Action::Check { name } => commands::exists_cmd(&connection, name),
        Action::List => commands::list_cmd(&connection),
        Action::Drop => commands::drop_cmd(&connection),
    }?;

    println!("{}", result);

    Ok(())
}

/// Struct containing all command line options
/// For more information see [clap documentation](https://docs.rs/clap/latest/clap/index.html)
#[derive(Debug, Parser)]
#[command(
    version,
    about = "config-store is a simple key-value store designed for use from shell scripts",
    author
)]
struct Args {
    /// What you want to do
    #[command(subcommand)]
    command: Action,
}

/// The different (sub-)commands that are available
#[derive(Debug, Subcommand)]
enum Action {
    /// Set / Change a value & it's alternate
    Set {
        /// The name of the Entry
        name: String,
        /// The value
        #[arg(short, long)]
        value: Option<String>,
        /// The alternate
        #[arg(short, long)]
        alternate: Option<String>,
        /// Only change entries, don't create a new ones
        #[arg(short, long)]
        change_only: bool,
    },
    /// Get a value & it's alternate
    Get {
        /// The name of the entry to get
        name: String,
        /// Only get the value
        #[arg(short, long, conflicts_with = "alternate_only")]
        value_only: bool,
        /// Only get the alternate
        #[arg(short, long, conflicts_with = "value_only")]
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
    /// List all entries
    List,
    /// Delete all entries !! BE VERY CAREFUL WITH THIS !!
    Drop,
}
