// Copyright (c) 2016, Mikkel Kroman <mk@uplink.io>
// All rights reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::str;

use url::Url;
use serde_json;

use error::{Error, Result};
use client::{Client, User, App};

#[derive(Debug)]
pub enum Filter {
    All,
    Public,
    Private
}

impl str::FromStr for Filter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Filter> {
        match s {
            "all" => Ok(Filter::All),
            "public" => Ok(Filter::Public),
            "private" => Ok(Filter::Private),
            _ => Err(Error::InvalidFilter(s.to_string())),
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl Filter {
    pub fn to_str(&self) -> &str {
        match *self {
            Filter::All => "all",
            Filter::Public => "public",
            Filter::Private => "private",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    /// Integer ID.
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
    pub release: Option<String>,
    pub release_day: Option<u64>,
    pub release_month: Option<u64>,
    pub release_year: Option<u64>,
    pub streamable: bool,
    pub downloadable: bool,
    pub purchase_title: Option<String>,
    pub state: String,
    pub license: String,
    pub track_type: Option<String>,
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
    pub likes_count: Option<usize>,
}

#[derive(Debug)]
pub struct TrackRequestBuilder<'a> {
    client: &'a Client,
    query: Option<String>,
    tags: Option<String>,
    filter: Option<Filter>,
    license: Option<String>,
    ids: Option<Vec<usize>>,
    duration: Option<(usize, usize)>,
    bpm: Option<(usize, usize)>,
    genres: Option<String>,
    types: Option<String>
}

#[derive(Debug)]
pub struct SingleTrackRequestBuilder<'a> {
    client: &'a Client,
    pub id: usize,
}

impl<'a> SingleTrackRequestBuilder<'a> {
    /// Constructs a new track request.
    pub fn new(client: &'a Client, id: usize) -> SingleTrackRequestBuilder {
        SingleTrackRequestBuilder {
            client: client,
            id: id,
        }
    }

    /// Sends the request and return the tracks.
    pub fn get(&mut self) -> Result<Track> {
        let no_params: Option<&[(&str, &str)]> = None;
        let response = try!(self.client.get(&format!("/tracks/{}", self.id), no_params));
        let track: Track = try!(serde_json::from_reader(response));

        Ok(track)
    }

    pub fn request_url(&self) -> Url {
        let url = Url::parse(&format!("https://{}/tracks/{}", super::API_HOST, self.id)).unwrap();

        url
    }
}


impl<'a> TrackRequestBuilder<'a> {
    /// Creates a new track request builder, with no set parameters.
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

    /// Sets the search query filter, which will only return tracks with a matching query.
    pub fn query<S>(&'a mut self, query: Option<S>) -> &mut TrackRequestBuilder
        where S: AsRef<str> {
        self.query = query.map(|s| s.as_ref().to_owned());
        self
    }

    /// Sets the tags filter, which will only return tracks with a matching tag.
    pub fn tags<I, T>(&'a mut self, tags: Option<I>) -> &mut TrackRequestBuilder
        where I: AsRef<[T]>, T: AsRef<str> {
        self.tags = tags.map(|s| {
            let tags_as_ref: Vec<_> = s.as_ref().iter().map(T::as_ref).collect();
            tags_as_ref.join(",")
        });
        self
    }

    pub fn genres<I, T>(&'a mut self, genres: Option<I>) -> &mut TrackRequestBuilder
        where I: AsRef<[T]>, T: AsRef<str> {
        self.genres = genres.map(|s| {
            let genres_as_ref: Vec<_> = s.as_ref().iter().map(T::as_ref).collect();
            genres_as_ref.join(",")
        });
        self
    }

    /// Sets whether to filter private or public tracks.
    pub fn filter(&'a mut self, filter: Option<Filter>) -> &mut TrackRequestBuilder {
        self.filter = filter;
        self
    }

    /// Sets the license filter.
    pub fn license<S: AsRef<str>>(&'a mut self, license: Option<S>) -> &mut TrackRequestBuilder {
        self.license = license.map(|s| s.as_ref().to_owned());
        self
    }

    /// Sets a list of track ids to look up.
    pub fn ids(&'a mut self, ids: Option<Vec<usize>>) -> &mut TrackRequestBuilder {
        self.ids = ids;
        self
    }

    /// Returns a builder for a single track.
    pub fn id(&'a mut self, id: usize) -> SingleTrackRequestBuilder {
        SingleTrackRequestBuilder {
            client: &self.client,
            id: id,
        }
    }

    /// Performs the request and returns a list of tracks if there are any results, None otherwise,
    /// or an error if one occurred.
    pub fn get(&mut self) -> Result<Option<Vec<Track>>> {
        use serde_json::Value;

        let response = try!(self.client.get("/tracks", Some(self.request_params())));
        let track_list: Value = try!(serde_json::from_reader(response));

        if let Some(track_list) = track_list.as_array() {
            if track_list.is_empty() {
                return Ok(None);
            } else {
               let tracks: Vec<Track> = track_list
                    .iter().map(|t| serde_json::from_value::<Track>(t.clone()).unwrap()).collect();

                return Ok(Some(tracks)); 
            }
        }

        return Err(Error::ApiError("expected response to be an array".to_owned()));
    }

    fn request_params(&self) -> Vec<(&str, String)> {
        let mut result = vec![];

        if let Some(ref query) = self.query {
            result.push(("q", query.clone()));
        }

        if let Some(ref tags) = self.tags {
            result.push(("tags", tags.clone()));
        }

        if let Some(ref filter) = self.filter {
            result.push(("filter", filter.to_str().to_owned()));
        }

        if let Some(ref ids) = self.ids {
            let ids_as_strings: Vec<String> = ids.iter().map(|id| format!("{}", id)).collect();
            result.push(("ids", ids_as_strings.join(",")));
        }

        if let Some(ref _duration) = self.duration {
            unimplemented!();
        }

        if let Some(ref _bpm) = self.bpm {
            unimplemented!();
        }

        if let Some(ref genres) = self.genres {
            result.push(("genres", genres.clone()));
        }

        if let Some(ref types) = self.types {
            result.push(("types", types.clone()));
        }

        result
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Track) -> bool {
        other.id == self.id
    }
}
