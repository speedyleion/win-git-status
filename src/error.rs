/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use std::io;

#[derive(Debug)]
pub struct StatusError {
    pub message: String,
}

impl From<io::Error> for StatusError {
    fn from(err: io::Error) -> StatusError {
        StatusError {
            message: err.to_string(),
        }
    }
}

