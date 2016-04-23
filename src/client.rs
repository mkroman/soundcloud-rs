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
use std::default::Default;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    //pub kind: String,
    pub id: u64,
    pub created_at: String,
    pub user_id: u64,
    pub user: User,
    pub title: String,
    pub permalink: String,
    pub permalink_url: String,
    pub uri: String,
    pub sharing: String,
    pub embeddable_by: String,
    pub purchase_url: Option<String>,
    pub artwork_url: Option<String>,
    pub description: Option<String>,
    pub label: Option<serde_json::Value>,
    pub duration: u64,
    pub genre: Option<String>,
    pub tags: Option<String>,
    pub label_id: Option<u64>,
    pub label_name: Option<String>,
    pub release: Option<u64>,
    pub release_day: Option<u64>,
    pub release_month: Option<u64>,
    pub release_year: Option<u64>,
    pub streamable: bool,
    pub downloadable: bool,
    pub purchase_title: Option<String>,
    pub state: String,
    pub license: String,
    pub track_type: String,
    pub waveform_url: String,
    pub download_url: Option<String>,
    pub stream_url: Option<String>,
    pub video_url: Option<String>,
    pub bpm: Option<u64>,
    pub commentable: bool,
    pub isrc: Option<String>,
    pub key_signature: Option<String>,
    pub comment_count: u64,
    pub download_count: u64,
    pub playback_count: u64,
    pub favoritings_count: u64,
    pub original_format: String,
    pub original_content_size: u64,
    pub created_with: Option<App>,
    pub asset_data: Option<Vec<u8>>,
    pub artwork_data: Option<Vec<u8>>,
    pub user_favorite: Option<bool>,
}

#[derive(Debug)]
pub struct TrackRequestBuilder<'a> {
    client: &'a Client,
    query: Option<String>,
    tags: Option<String>,
    filter: Option<String>,
    license: Option<String>,
    ids: Option<Vec<usize>>,
    duration: Option<(usize, usize)>,
    bpm: Option<(usize, usize)>,
    genres: Option<String>,
    types: Option<String>
}

impl<'a> TrackRequestBuilder<'a> {
    pub fn new(client: &'a Client) -> TrackRequestBuilder {
        TrackRequestBuilder {
            client: client,
            query: None,
            tags: None,
            filter: None,
            license: None,
            ids: None,
            duration: None,
            bpm: None,
            genres: None,
            types: None,
        }
    }

    pub fn query(&mut self, query: Option<&str>) -> &'a mut TrackRequestBuilder {
        self.query = query.map(|x| x.to_owned());
        self
    }

    pub fn get(&mut self) -> Result<(), ()> {
        Ok(())
    }

    fn request_url(&self) -> Url {
        let mut url = Url::parse(&format!("https://{}/tracks", super::API_HOST)).unwrap();

        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("client_id", &self.client.get_client_id());

            if let Some(ref query) = self.query {
                query_pairs.append_pair("q", &query);
            }

            if let Some(ref tags) = self.tags {
                query_pairs.append_pair("tags", &tags);
            }

            if let Some(ref filter) = self.filter {
                query_pairs.append_pair("filter", &filter);
            }

            if let Some(ref ids) = self.ids {
                let ids_as_strings: Vec<String> = ids.iter().map(|id| format!("{}", id)).collect();

                query_pairs.append_pair("ids", &ids_as_strings.join(","));
            }

            if let Some(ref duration) = self.duration {
                unimplemented!();
            }

            if let Some(ref bpm) = self.bpm {
                unimplemented!();
            }

            if let Some(ref genres) = self.genres {
                query_pairs.append_pair("genres", &genres);
            }

            if let Some(ref types) = self.types {
                query_pairs.append_pair("types", &types);
            }
        }

        url
    }
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

        request.read_to_string(&mut buffer);

        match serde_json::from_str(&buffer) {
            Ok(track) => Some(track),
            Err(serde_json::Error::Syntax(serde_json::ErrorCode::MissingField(_), _, _)) => {
                None
            },
            _ => None
        }
    }

    /// Request soundcloud tracks.
    pub fn tracks<'a>(&'a mut self) -> TrackRequestBuilder<'a> {
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

    #[test]
    fn test_get() {
        let params: Option<Params<&str, &str>> = None;
        let mut response = client().get("/tracks/13158665", params).unwrap();
        let mut buffer = String::new();
        response.read_to_string(&mut buffer);
        println!("{}", buffer);
    }

    #[test]
    fn test_get_track() {
        let mut track = client().get_track(13158665).unwrap();

        println!("{:?}", track);
    }

    #[test]
    fn test_request_builder() {
        let mut client = client();
        {
            let mut b = client.tracks()
                .query(Some("hello world"))
                .get();
        }

        println!("Client: {:?}", client);
    }
}
