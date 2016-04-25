// Copyright (c) 2016, Mikkel Kroman <mk@uplink.io>
// All rights reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error;
use std::fmt;

use serde_json;

#[derive(Debug)]
pub enum Error {
    JsonError(serde_json::Error),
    InvalidFilter(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::JsonError(ref error) => write!(f, "JSON error: {}", error),
            Error::InvalidFilter(_) => write!(f, "Invalid filter"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidFilter(_) => "invalid filter",
            Error::JsonError(ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::JsonError(ref error) => Some(error),
            _ => None
        }
    }
}
