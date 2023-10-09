// Shortwave - mod.rs
// Copyright (C) 2021-2023  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

static STATION_SEARCH: &str = "json/stations/search";
static STATION_BY_UUID: &str = "json/stations/byuuid/";
static STATS: &str = "json/stats";

mod client;
mod error;
mod favicon_downloader;
mod object;
mod station;
mod station_metadata;
mod station_request;
mod station_url;
mod stats;

pub use client::SwClient;
pub use error::Error;
pub use favicon_downloader::FaviconDownloader;
pub use object::Object;
pub use station::SwStation;
pub use station_metadata::StationMetadata;
pub use station_request::StationRequest;
pub use station_url::StationUrl;
pub use stats::Stats;
