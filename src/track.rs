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

use error::Error;
use client::{Client, User, App};

#[derive(Debug)]
pub enum Filter {
    All,
    Public,
    Private
}

impl str::FromStr for Filter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Filter, Error> {
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
    filter: Option<Filter>,
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

    pub fn tags(&'a mut self, tags: Option<Vec<&str>>) -> &'a mut TrackRequestBuilder {
        self.tags = tags.map(|x| x.join(","));
        self
    }

    pub fn filter(&'a mut self, filter: Option<Filter>) -> &mut TrackRequestBuilder {
        self.filter = filter;
        self
    }

    pub fn license(&'a mut self, license: Option<&str>) -> &mut TrackRequestBuilder {
        self.license = license.map(|x| x.to_owned());
        self
    }

    pub fn ids(&'a mut self, ids: Option<Vec<usize>>) -> &mut TrackRequestBuilder {
        self.ids = ids;
        self
    }

    pub fn get(&mut self) -> Result<(), ()> {
        debug!("get {}", self.request_url());
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
                query_pairs.append_pair("filter", filter.to_str());
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
