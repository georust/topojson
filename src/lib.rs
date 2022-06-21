// Copyright 2018 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Examples
//!
//! This crate uses `serde` for serialization and allows conversion to [GeoJson](https://github.com/georust/geojson/) objects.
//!
//! To get started, add `topojson` to your `Cargo.toml`:
//!
//! ```text
//! [dependencies]
//! topojson = "*"
//! ```
//!
//! ## Reading
//!
//! ```rust
//! # extern crate topojson;
//! use topojson::TopoJson;
//!
//! let topojson_str = r#"
//! {
//!     "arcs": [[[0.0, 0.0], [0.0, 9999.0], [2000.0, 0.0], [0.0, -9999.0], [-2000.0, 0.0]]],
//!     "objects": {"example ": {
//!             "type": "GeometryCollection",
//!             "geometries": [
//!                 {"coordinates": [4000.0, 5000.0],
//!                  "properties": {"prop0": "value0"},
//!                  "type": "Point"},
//!                 {"arcs": [[0]],
//!                  "properties": {"prop0": "value0", "prop1": {"this": "that"}},
//!                  "type": "Polygon"}
//!             ]
//!      }},
//!     "transform": {"scale": [0.0005, 0.0001], "translate": [100.0, 0.0]},
//!     "type": "Topology"
//! }
//! "#;
//!
//! let topo = topojson_str.parse::<TopoJson>().unwrap();
//! ```
//!
//! ## Writing
//!
//!
//! `TopoJson` can then be serialized by calling `to_string`:
//!
//! ```rust
//! # extern crate topojson;
//! # extern crate serde_json;
//!
//! use topojson::{TopoJson, Geometry, Value, Topology, NamedGeometry};
//! use serde_json;
//!
//! let topo = Topology {
//!     arcs: vec![
//!         vec![vec![2.2, 2.2], vec![3.3, 3.3]]
//!     ],
//!     objects: vec![NamedGeometry {
//!         name: String::from("example"),
//!         geometry: Geometry {
//!             value: Value::LineString(vec![0]),
//!             bbox: None,
//!             id: None,
//!             properties: None,
//!             foreign_members: None,
//!         },
//!     }],
//!     bbox: None,
//!     transform: None,
//!     foreign_members: None,
//! };
//!
//! let topojson_string = serde_json::to_string(&topo).unwrap();
//! ```
//!
//! ## Converting to GeoJson
//!
//!
//! `TopoJson` can then be converted to `GeoJson using the `to_geojson` function:
//!
//! ```rust
//! # extern crate topojson;
//! extern crate geojson;
//!
//! use topojson::{TopoJson, to_geojson};
//! use geojson::GeoJson;
//!
//! let topojson_str = r#"
//! {
//!     "arcs": [[[0.0, 0.0], [0.0, 9999.0], [2000.0, 0.0], [0.0, -9999.0], [-2000.0, 0.0]]],
//!     "objects": {"example": { "type": "GeometryCollection", "geometries": [
//!         {"coordinates": [4000.0, 5000.0], "properties": {"prop0": "value0"}, "type": "Point"},
//!         {"arcs": [[0]], "properties": {"prop0": "value0", "prop1": {"this": "that"}}, "type": "Polygon"}]}},
//!     "transform": {"scale": [0.0005, 0.0001], "translate": [100.0, 0.0]},
//!     "type": "Topology"
//! }"#;
//! // Parse to Topology:
//! let topo = topojson_str.parse::<TopoJson>().unwrap();
//!
//! // Conversion to GeoJson FeatureCollection for the "example" object:
//! let geojson = match topo {
//!     TopoJson::Topology(t) => {
//!         to_geojson(&t, &String::from("example"))
//!             .expect("Unable to convert TopoJSON to GeoJSON")
//!     },
//!     _ => unimplemented!(),
//! };
//! ```
//!
extern crate serde;
#[macro_use]
extern crate serde_json;

extern crate geojson;
/// Bounding Boxes
///
/// [TopoJSON Format Specification § 3](https://github.com/topojson/topojson-specification#3-bounding-boxes)
pub type Bbox = Vec<f64>;

/// Positions
///
/// [TopoJSON Format Specification § 2.1.1](https://github.com/topojson/topojson-specification#211-positions)
pub type Position = Vec<f64>;
// pub type PointType = Position;

/// Arcs (an array of position which may have been quantized and delta-encoded)
///
/// [TopoJSON Format Specification $ 2.1.3](https://github.com/topojson/topojson-specification#213-arcs)
///
/// Warning: This has a completely different meaning from the
/// [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) term in the standard library.
/// It is used here to describe what could also be commonly called an *edge*.
pub type Arc = Vec<Position>;

/// Arc indexes (an array of indexes)
///
/// [TopoJSON Format Specification $ 2.1.4](https://github.com/topojson/topojson-specification#214-arc-indexes)
pub type ArcIndexes = Vec<i32>;

pub(crate) mod util;

mod topojson;
pub use crate::topojson::TopoJson;

mod geometry;
pub use crate::geometry::{Geometry, NamedGeometry, Value};

mod topology;
pub use crate::topology::{Topology, TransformParams};

mod to_geojson;
pub use crate::to_geojson::to_geojson;

mod error;
pub use crate::error::Error;

mod json {
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub use serde_json::{Map, Value as JsonValue};
    pub type JsonObject = Map<String, JsonValue>;
}
