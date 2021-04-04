/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

/// The status of a file in relation to the rest of the git repo.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Status {
    Current,
    New,
    Modified,
    Deleted,
}
impl Default for Status {
    fn default() -> Self {
        Status::Current
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct StatusEntry {
    pub name: String,
    pub state: Status,
}
