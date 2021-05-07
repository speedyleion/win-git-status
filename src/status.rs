/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use std::fmt;

/// The status of a file in relation to the rest of the git repo.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Status {
    Current,
    New,
    Modified(Option<String>),
    Deleted,
}
impl Default for Status {
    fn default() -> Self {
        Status::Current
    }
}
impl fmt::Display for Status {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Status::Current => fmt.write_str(""),
            Status::New => fmt.write_str("new file:   "),
            Status::Modified(_) => fmt.write_str("modified:   "),
            Status::Deleted => fmt.write_str("deleted:    "),
        }
    }
}
impl Status {
    pub fn is_modified(&self) -> bool {
        matches!(*self, Status::Modified(_))
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct StatusEntry {
    pub name: String,
    pub state: Status,
}

impl fmt::Display for StatusEntry {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.state.to_string())?;
        fmt.write_str(&self.name)?;
        Ok(())
    }
}
