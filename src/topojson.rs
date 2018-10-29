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

use std::fmt;
use std::str::FromStr;

use json::{self, Deserialize, Deserializer, JsonObject, Serialize, Serializer};
use { Error, Geometry, Topology };


/// TopoJSON Objects (either Topology or Geometry)
///
/// [TopoJSON Format Specification § 2](https://github.com/topojson/topojson-specification#2-topojson-objects)
#[derive(Clone, Debug, PartialEq)]
pub enum TopoJson {
    /// [TopoJSON Format Specification § 2.2](https://github.com/topojson/topojson-specification#22-geometry-objects)
    Geometry(Geometry),
    /// [TopoJSON Format Specification § 2.1](https://github.com/topojson/topojson-specification#21-topology-objects)
    Topology(Topology),
}

impl<'a> From<&'a TopoJson> for JsonObject {
    fn from(topo: &'a TopoJson) -> JsonObject {
        match *topo {
            TopoJson::Geometry(ref geometry) => geometry.into(),
            TopoJson::Topology(ref topo) => topo.into(),
        }
    }
}

impl From<Geometry> for TopoJson {
    fn from(geometry: Geometry) -> Self {
        TopoJson::Geometry(geometry)
    }
}

impl From<Topology> for TopoJson {
    fn from(topo: Topology) -> Self {
        TopoJson::Topology(topo)
    }
}

impl TopoJson {
    pub fn from_json_object(object: JsonObject) -> Result<Self, Error> {
        let type_ = match object.get("type") {
            Some(json::JsonValue::String(t)) => Type::from_str(t),
            _ => return Err(Error::ExpectedProperty("type".to_owned())),
        };
        let type_ = type_.ok_or(Error::TopoJsonUnknownType)?;
        match type_ {
            Type::Point
            | Type::MultiPoint
            | Type::LineString
            | Type::MultiLineString
            | Type::Polygon
            | Type::MultiPolygon
            | Type::GeometryCollection => Geometry::from_json_object(object).map(TopoJson::Geometry),
            Type::Topology => Topology::from_json_object(object).map(TopoJson::Topology),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Type {
    Point,
    MultiPoint,
    LineString,
    MultiLineString,
    Polygon,
    MultiPolygon,
    GeometryCollection,
    Topology,
}

impl Type {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Point" => Some(Type::Point),
            "MultiPoint" => Some(Type::MultiPoint),
            "LineString" => Some(Type::LineString),
            "MultiLineString" => Some(Type::MultiLineString),
            "Polygon" => Some(Type::Polygon),
            "MultiPolygon" => Some(Type::MultiPolygon),
            "GeometryCollection" => Some(Type::GeometryCollection),
            "Topology" => Some(Type::Topology),
            _ => None,
        }
    }
}

impl Serialize for TopoJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        JsonObject::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TopoJson {
    fn deserialize<D>(deserializer: D) -> Result<TopoJson, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as SerdeError;
        use std::error::Error as StdError;

        let val = JsonObject::deserialize(deserializer)?;

        TopoJson::from_json_object(val).map_err(|e| D::Error::custom(e.description()))
    }
}

impl FromStr for TopoJson {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let object = get_object(s)?;

        TopoJson::from_json_object(object)
    }
}

fn get_object(s: &str) -> Result<json::JsonObject, Error> {
    ::serde_json::from_str(s)
        .ok()
        .and_then(json_value_into_json_object)
        .ok_or(Error::MalformedJson)
}

fn json_value_into_json_object(json_value: json::JsonValue) -> Option<json::JsonObject> {
    if let json::JsonValue::Object(geo) = json_value {
        Some(geo)
    } else {
        None
    }
}

impl fmt::Display for TopoJson {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::serde_json::to_string(self)
            .map_err(|_| fmt::Error)
            .and_then(|s| f.write_str(&s))
    }
}
