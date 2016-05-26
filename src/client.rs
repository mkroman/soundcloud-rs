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
use std::io::{self, Write};

use track::{Track, TrackRequestBuilder, SingleTrackRequestBuilder};
use error::{Error, Result};

pub type Params<'a, K, V> = &'a [(K, V)];

#[derive(Debug)]
pub struct Client {
    client_id: String,
    http_client: hyper::Client,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct App {
    pub id: usize,
    pub uri: String,
    pub permalink_url: String,
    pub external_url: String,
    pub creator: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    /// Constructs a new `Client` with the provided `client_id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    /// ```
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

    /// Creates and sends a HTTP GET request to the API endpoint.
    ///
    /// A `client_id` parameter will automatically be added to the request.
    ///
    /// Returns the HTTP response on success, an error otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Read;
    /// use soundcloud::Client;
    /// let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    /// let response = client.get("/resolve", Some(&[("url",
    /// "https://soundcloud.com/firepowerrecs/afk-shellshock-kamikaze-promo-mix-lock-load-series-vol-20")]));
    ///
    /// let mut buffer = String::new();
    /// response.unwrap().read_to_string(&mut buffer);
    ///
    /// assert!(!buffer.is_empty());
    /// ```
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

        let response = self.http_client.get(url).send();
        response
    }

    pub fn download<W: Write>(&self, track: &Track, mut writer: W) -> Result<usize> {
        use hyper::header::Location;

        if !track.downloadable || !track.download_url.is_some() {
            return Err(Error::TrackNotDownloadable);
        }

        let url = self.parse_url(track.download_url.as_ref().unwrap());
        let mut response = try!(self.http_client.get(url).send());

        // Follow the redirect just this once.
        if let Some(header) = response.headers.get::<Location>().cloned() {
            let url = Url::parse(&header).unwrap();
            response = try!(self.http_client.get(url).send());
        }

        try!(io::copy(&mut response, &mut writer).map(|n| Ok(n as usize)))
    }

    /// Starts streaming the track provided in the tracks `stream_url` to the `writer` if the track
    /// is streamable via the API.
    pub fn stream<W: Write>(&self, track: &Track, mut writer: W) -> Result<usize> {
        use hyper::header::Location;

        if !track.streamable || !track.stream_url.is_some() {
            return Err(Error::TrackNotStreamable);
        }

        let url = self.parse_url(track.stream_url.as_ref().unwrap());
        let mut response = try!(self.http_client.get(url).send());

        // Follow the redirect just this once.
        if let Some(header) = response.headers.get::<Location>().cloned() {
            let url = Url::parse(&header).unwrap();
            response = try!(self.http_client.get(url).send());
        }

        try!(io::copy(&mut response, &mut writer).map(|n| Ok(n as usize)))
    }

    /// Resolves any soundcloud resource and returns it as a `Url`.
    pub fn resolve(&self, url: &str) -> Result<Url> {
        use hyper::header::Location;
        let response = try!(self.get("/resolve", Some(&[("url", url)])));

        if let Some(header) = response.headers.get::<Location>() {
            Ok(Url::parse(header).unwrap())
        } else {
            Err(Error::ApiError("expected location header".to_owned()))
        }
    }

    /// Returns a builder for a single track-by-id request.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    /// let track = client.track(262681089).get();
    ///
    /// assert_eq!(track.unwrap().id, 262681089);
    /// ```
    pub fn track(&self, id: usize) -> SingleTrackRequestBuilder {
        SingleTrackRequestBuilder::new(self, id)
    }

    /// Returns a builder for searching tracks with multiple criteria.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    /// let tracks = client.tracks().genres(Some(["HipHop"])).get();
    ///
    /// assert!(tracks.unwrap().expect("no tracks found").len() > 0);
    /// ```
    pub fn tracks(&self) -> TrackRequestBuilder {
        TrackRequestBuilder::new(self)
    }

    /// Parses a string and returns a url with the client_id query parameter set.
    fn parse_url<S: AsRef<str>>(&self, url: S) -> Url {
        let mut url = Url::parse(url.as_ref()).unwrap();
        url.query_pairs_mut().append_pair("client_id", &self.client_id);
        url
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
        use std::fs;
        use std::path::Path;

        let client = client();
        let path = Path::new("hi.mp3");
        let track = client.tracks().id(263801976).get().unwrap();
        let mut file = fs::File::create(path).unwrap();
        let ret = client.download(&track, &mut file);

        assert!(ret.unwrap() > 0);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_stream_track() {
        use std::io::BufWriter;
        let client = client();
        let track = client.tracks().id(262681089).get().unwrap();
        let mut buffer = BufWriter::new(vec![]);
        let len = client.stream(&track, &mut buffer);

        assert!(len.unwrap() > 0);
        assert!(buffer.get_ref().len() > 0);
    }
}
