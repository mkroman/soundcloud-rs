// Copyright (c) 2016, Mikkel Kroman <mk@uplink.io>
// All rights reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use url::Url;
use hyper;
use serde_json;

use std::io::Read;
use std::borrow::Borrow;

use track::{Track, TrackRequestBuilder};

pub type Params<'a, K, V> = &'a [(K, V)];

#[derive(Debug)]
pub struct Client {
    client_id: String,
    http_client: hyper::Client,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct App {
    pub id: usize,
    pub uri: String,
    pub permalink_url: String,
    pub external_url: String,
    pub creator: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    pub id: usize,
    pub uri: String,
    pub created_at: String,
    pub body: String,
    pub timestamp: Option<usize>,
    pub user_id: usize,
    pub user: User,
    pub track_id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: usize,
    pub permalink: String,
    pub username: String,
    pub uri: String,
    pub permalink_url: String,
    pub avatar_url: String,
    pub country: Option<String>,
    pub city: Option<String>,
    pub description: Option<String>,
    #[serde(rename="discogs-name")]
    pub discogs_name: Option<String>, // discogs-name
    #[serde(rename="myspace-name")]
    pub myspace_name: Option<String>, // myspace-name
    pub website: Option<String>,
    #[serde(rename="website-title")]
    pub website_title: Option<String>, // website-title
    pub online: Option<bool>,
    pub track_count: Option<usize>,
    pub playlist_count: Option<usize>,
    pub followers_count: Option<usize>,
    pub followings_count: Option<usize>,
    pub public_favorites_count: Option<usize>,
    // pub avatar_data …
}

impl Client {
    pub fn new(client_id: &str) -> Client {
        Client {
            client_id: client_id.to_owned(),
            http_client: hyper::Client::new(),
        }
    }

    pub fn get_client_id(&self) -> &str {
        &self.client_id
    }

    pub fn get<I, K, V>(&mut self, path: &str, params: Option<I>)
        -> Result<hyper::client::Response, hyper::Error>
    where I: IntoIterator, I::Item: Borrow<(K, V)>, K: AsRef<str>, V: AsRef<str> {
        let mut url = Url::parse(&format!("https://{}{}", super::API_HOST, path)).unwrap();

        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("client_id", &self.client_id);

            if let Some(params) = params {
                query_pairs.extend_pairs(params);
            }
        }

        debug!("get {}", url);

        let response = self.http_client.get(url).send();
        response
    }

    /// Request a track with a given ID.
    pub fn get_track(&mut self, track_id: usize) -> Option<Track> {
        let params: Option<Params<&str, &str>> = None;
        let mut request = self.get(&format!("/tracks/{}", track_id), params).unwrap();
        let mut buffer = String::new();

        request.read_to_string(&mut buffer).unwrap();

        match serde_json::from_str(&buffer) {
            Ok(track) => Some(track),
            Err(serde_json::Error::Syntax(serde_json::ErrorCode::MissingField(_), _, _)) => {
                None
            },
            _ => None
        }
    }

    /// Request soundcloud tracks.
    pub fn tracks(&self) -> TrackRequestBuilder {
        TrackRequestBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn client() -> Client {
        Client::new(env!("SOUNDCLOUD_CLIENT_ID"))
    }
}
