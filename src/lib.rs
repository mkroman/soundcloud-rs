// Copyright 2016 Mikkel Kroman <mk@uplink.io>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate url;
extern crate hyper;
#[macro_use] extern crate log;
extern crate serde;
extern crate serde_json;

/// The host address for the API.
pub const API_HOST: &'static str = "api.soundcloud.com";

pub mod error;
pub mod client;
pub mod track;

pub enum Resource {
    Track(track::Track),
}

pub use client::Client;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
