//! Module containing the [Entry] struct
use std::fmt::Display;

/// Representation an entry in the db
#[derive(Debug)]
pub struct Entry {
    /// The id in the db
    ///
    /// This is used as the primary key in the db. It is never touched by the user.
    pub _id: i32,
    /// The identifier set & accessed by users
    pub name: String,
    /// The value of the entry
    pub value: String,
    /// An additional value that can be toggled to
    ///
    /// This is particularly useful for true / false toggles
    pub alternate: String,
}

impl Entry {
    pub fn json(self) -> String {
        format!(
            r#"{{ "_id": "{}", "name": "{}", "value": "{}", "alternate": "{}" }}"#,
            self._id, self.name, self.value, self.alternate
        )
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
