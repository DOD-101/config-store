# Config-Store

Config-Store is a simple key-value store designed to be used from shell scripts. 

## Example

Say, for example, you have a keybinding that switches between two applications. For this, you will probably need to save what app you are currently in. 

Now you could create an entry with the `set` command and then toggle between the states with `toggle`

## Usage

`config-store --help` or see doc comments in `./src/commands.rs`

## To-Do

- [ ] Create packages (cargo & nix?)

- [ ] Iron out any bugs

## Technical details 

- The data (aka the key-value pairs) are stored in `/tmp/config-store.db`, which is a sqlite3 database.

- Internally, the commands are mostly wrappers around SQL statements, but as simple shell commands.

- While it is technically possible to have multiple different entries with the same name, because the primary key is not the name.
  This is impossible to do with the commands provided, since `set` will always update a value if it exists.

- Because the data is stored on disk, config-store needs no server process. Not only does this make it simpler, it also means there is no overhead to 
  using it to store your variables.

## License 

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT License](https://opensource.org/license/MIT)

at your option.

