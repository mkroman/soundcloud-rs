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

use std::result;
use std::borrow::Borrow;
use std::io::{Read, Write};
use std::path::Path;
use std::fs::File;

use track::{Track, TrackRequestBuilder};
use error::{Error, Result};

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
    /// Constructs a new client with the provided client id.
    pub fn new(client_id: &str) -> Client {
        let mut client = hyper::Client::new();
        client.set_redirect_policy(hyper::client::RedirectPolicy::FollowNone);

        Client {
            client_id: client_id.to_owned(),
            http_client: client,
        }
    }

    /// Returns the client id.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Sends a HTTP GET request to the API endpoint.
    pub fn get<I, K, V>(&self, path: &str, params: Option<I>)
        -> result::Result<hyper::client::Response, hyper::Error>
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

        let mut response = self.http_client.get(url).send();
        response
    }

    pub fn download<P>(&self, track: &Track, path: P) -> Result<usize>
        where P: AsRef<Path> {
        use std::io::ErrorKind;
        use hyper::header::Location;

        if !track.downloadable || !track.download_url.is_some() {
            return Err(Error::TrackNotDownloadable);
        }

        let mut url = Url::parse(track.download_url.as_ref().unwrap()).unwrap();
        url.query_pairs_mut().append_pair("client_id", &self.client_id);

        let mut file = try!(File::create(path.as_ref()));
        let mut response = try!(self.http_client.get(url).send());
        let mut buffer = [0; 16384];
        let mut len = 0;
        let ret;

        // Follow the redirect just this once.
        if let Some(header) = response.headers.get::<Location>().cloned() {
            response = try!(self.http_client.get(Url::parse(&header).unwrap()).send());
        }

        loop {
            match response.read(&mut buffer) {
                Ok(0) => {
                    ret = Ok(len);
                    break;
                },
                Ok(n) => {
                    len += n;
                    try!(file.write_all(&mut buffer[..n]));
                },
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {},
                Err(e) => {
                    ret = Err(e.into());
                    break;
                }
            }
        }

        ret
    }

    pub fn stream<F>(&self, track: &Track, mut callback: F) -> Result<usize>
        where F: FnMut(&[u8]) {
        use std::io::ErrorKind;
        use hyper::header::Location;

        if !track.streamable || !track.stream_url.is_some() {
            return Err(Error::TrackNotStreamable);
        }

        let mut url = Url::parse(track.stream_url.as_ref().unwrap()).unwrap();
        url.query_pairs_mut().append_pair("client_id", &self.client_id);

        let mut response = try!(self.http_client.get(url).send());
        let mut buffer = [0; 16384];
        let mut len = 0;
        let ret;

        // Follow the redirect just this once.
        if let Some(header) = response.headers.get::<Location>().cloned() {
            response = try!(self.http_client.get(Url::parse(&header).unwrap()).send());
        }

        loop {
            match response.read(&mut buffer) {
                Ok(0) => {
                    ret = Ok(len);
                    break;
                },
                Ok(n) => {
                    len += n;
                    callback(&mut buffer[..n]);
                },
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {},
                Err(e) => {
                    ret = Err(e.into());
                    break;
                }
            }
        }

        ret
    }

    /// Resolves any soundcloud resource and returns it as a `Url`.
    pub fn resolve(&self, url: &str) -> Result<Url> {
        use hyper::header::Location;
        let request = self.get("/resolve", Some(&[("url", url)]));

        match request {
            Ok(response) => {
                if let Some(header) = response.headers.get::<Location>() {
                    return Ok(Url::parse(header).unwrap());
                } else {
                    return Err(Error::ApiError("expected location header".to_owned()));
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    /// Request soundcloud tracks.
    pub fn tracks(&self) -> TrackRequestBuilder {
        TrackRequestBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use url::Url;
    use super::*;

    fn client() -> Client {
        Client::new(env!("SOUNDCLOUD_CLIENT_ID"))
    }

    #[test]
    fn test_resolve_track() {
        let result = client().resolve("https://soundcloud.com/isqa/tree-eater-1");

        assert_eq!(result.unwrap(),
            Url::parse(&format!("https://api.soundcloud.com/tracks/262976655?client_id={}", 
                                env!("SOUNDCLOUD_CLIENT_ID"))).unwrap());
    }

    #[test]
    fn test_get_tracks() {
        let result = client().tracks().query(Some("d0df0dt snuffx")).get();

        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_get_track() {
        let track = client().tracks().id(18201932).get().unwrap();

        assert_eq!(track.id, 18201932);
    }

    #[test]
    fn test_download_track() {
        let client = client();
        let track = client.tracks().id(262681089).get().unwrap();
        let ret = client.download(&track, "hi.mp3");

        assert!(ret.unwrap() > 0);
    }

    #[test]
    fn test_stream_track() {
        let client = client();
        let track = client.tracks().id(262681089).get().unwrap();
        let mut size = 0;
        
        let ret = client.stream(&track, |chunk: &[u8]| size += chunk.len());

        let res = ret.unwrap();

        assert!(size == res);
        assert!(res > 0);
    }
}
