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

use json::{Deserialize, Deserializer, JsonObject, JsonValue, Serialize, Serializer};

use {util, Bbox, Error, Position, ArcIndexes, topojson::Type};

/// The underlying Geometry value (which may contain Position or Arc indexes)
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Point
    ///
    /// [TopoJSON Format Specification § 2.2.1](https://github.com/topojson/topojson-specification#221-point)
    Point(Position),

    /// MultiPoint
    ///
    /// [TopoJSON Format Specification § 2.2.2](https://github.com/topojson/topojson-specification#222-multipoint)
    MultiPoint(Vec<Position>),

    /// LineString
    ///
    /// [TopoJSON Format Specification § 2.2.3](https://github.com/topojson/topojson-specification#223-linestring)
    LineString(ArcIndexes),

    /// MultiLineString
    ///
    /// [TopoJSON Format Specification § 2.2.4](https://github.com/topojson/topojson-specification#224-multilinestring)
    MultiLineString(Vec<ArcIndexes>),

    /// Polygon
    ///
    /// [TopoJSON Format Specification § 2.2.5](https://github.com/topojson/topojson-specification#225-polygon)
    Polygon(Vec<ArcIndexes>),

    /// MultiPolygon
    ///
    /// [TopoJSON Format Specification § 3.1.7](https://github.com/topojson/topojson-specification#226-multipolygon)
    MultiPolygon(Vec<Vec<ArcIndexes>>),

    /// GeometryCollection
    ///
    /// [TopoJSON Format Specification § 3.1.8](https://github.com/topojson/topojson-specification#227-geometry-collection)
    GeometryCollection(Vec<Geometry>),
}

impl Value {
    pub fn to_json_value(&self) -> JsonValue {
        match *self {
            Value::Point(ref x) => ::serde_json::to_value(x),
            Value::MultiPoint(ref x) => ::serde_json::to_value(x),
            Value::LineString(ref x) => ::serde_json::to_value(x),
            Value::MultiLineString(ref x) => ::serde_json::to_value(x),
            Value::Polygon(ref x) => ::serde_json::to_value(x),
            Value::MultiPolygon(ref x) => ::serde_json::to_value(x),
            Value::GeometryCollection(ref x) => ::serde_json::to_value(x),
        }
        .unwrap()

    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // FIXME: We should try to serialize it more efficiently
        self.to_json_value().serialize(serializer)
    }
}

/// Geometry Objects
///
/// [TopoJSON Format Specification § 2.2](https://github.com/topojson/topojson-specification#22-geometry-objects)
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub bbox: Option<Bbox>,
    pub value: Value,
    pub properties: Option<JsonObject>,
    pub id: Option<JsonValue>,
    /// Foreign Members
    ///
    /// [TopoJSON Format Specification](https://github.com/topojson/topojson-specification#22-geometry-objects)
    pub foreign_members: Option<JsonObject>,
}

impl Geometry {
    /// Returns a new `Geometry` with the specified `value`. `bbox` and `foreign_members` will be
    /// set to `None`.
    pub fn new(value: Value) -> Self {
        Geometry {
            bbox: None,
            id: None,
            value,
            properties: None,
            foreign_members: None,
        }
    }
}

impl<'a> From<&'a Geometry> for JsonObject {
    fn from(geometry: &'a Geometry) -> JsonObject {
        let mut map = JsonObject::new();
        if let Some(ref bbox) = geometry.bbox {
            map.insert(String::from("bbox"), ::serde_json::to_value(bbox).unwrap());
        }

        let ty = String::from(match geometry.value {
            Value::Point(..) => "Point",
            Value::MultiPoint(..) => "MultiPoint",
            Value::LineString(..) => "LineString",
            Value::MultiLineString(..) => "MultiLineString",
            Value::Polygon(..) => "Polygon",
            Value::MultiPolygon(..) => "MultiPolygon",
            Value::GeometryCollection(..) => "GeometryCollection",
        });

        map.insert(String::from("type"), ::serde_json::to_value(&ty).unwrap());

        map.insert(
            String::from(match geometry.value {
                Value::GeometryCollection(..) => "geometries",
                Value::LineString(..)
                | Value::MultiLineString(..)
                | Value::Polygon(..)
                | Value::MultiPolygon(..) => "arcs",
                _ => "coordinates",
            }),
            ::serde_json::to_value(&geometry.value).unwrap(),
        );

        if let Some(ref id) = geometry.id {
            map.insert(String::from("id"), serde_json::to_value(id).unwrap());
        }

        if let Some(ref properties) = geometry.properties {
            let mut prop = JsonObject::new();
            for (key, value) in properties {
                prop.insert(key.to_owned(), value.to_owned());
            }
            map.insert(String::from("properties"), ::serde_json::Value::Object(prop));
        }
        if let Some(ref foreign_members) = geometry.foreign_members {
            for (key, value) in foreign_members {
                map.insert(key.to_owned(), value.to_owned());
            }
        }
        map
    }
}

impl Geometry {
    pub fn from_json_object(mut object: JsonObject) -> Result<Self, Error> {
        let value = match &Type::from_str(&util::expect_type(&mut object)?).ok_or(Error::TopoJsonUnknownType)? {
            Type::Point => Value::Point(util::get_coords_one_pos(&mut object)?),
            Type::MultiPoint => Value::MultiPoint(util::get_coords_1d_pos(&mut object)?),
            Type::LineString => Value::LineString(util::get_arc_ix(&mut object)?),
            Type::MultiLineString => Value::MultiLineString(util::get_arc_ix_1d(&mut object)?),
            Type::Polygon => Value::Polygon(util::get_arc_ix_1d(&mut object)?),
            Type::MultiPolygon => Value::MultiPolygon(util::get_arc_ix_2d(&mut object)?),
            Type::GeometryCollection => Value::GeometryCollection(util::get_geometries(&mut object)?),
            _ => return Err(Error::GeometryUnknownType),
        };
        Ok(Geometry {
            value,
            bbox: util::get_bbox(&mut object)?,
            id: util::get_id(&mut object)?,
            properties: util::get_properties(&mut object)?,
            foreign_members: util::get_foreign_members(object)?,
        })
    }
}

impl Serialize for Geometry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // FIXME: We should try to serialize it more efficiently
        JsonObject::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Geometry {
    fn deserialize<D>(deserializer: D) -> Result<Geometry, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as SerdeError;
        use std::error::Error as StdError;

        let val = JsonObject::deserialize(deserializer)?;

        Geometry::from_json_object(val).map_err(|e| D::Error::custom(e.description()))
    }
}

/// One member of the 'objects' member of a Topology
///
/// [TopoJSON Format Specification § 2.1.5](https://github.com/topojson/topojson-specification#215-objects)
#[derive(Clone, Debug, PartialEq)]
pub struct NamedGeometry {
    pub name: String,
    pub geometry: Geometry,
}


#[cfg(test)]
mod tests {
    use json::JsonObject;
    use serde_json;
    use {TopoJson, Geometry, Value, Error};

    fn encode(geometry: &Geometry) -> String {
        serde_json::to_string(&geometry).unwrap()
    }

    fn decode(json_string: String) -> TopoJson {
        json_string.parse().unwrap()
    }

    #[test]
    fn decode_invalid_linestring() {
        let topo_json_str = "{\"coordinates\":[0],\"type\":\"LineString\"}";

        // Decode should fail due to the absence of the 'arcs' member:
        let result = topo_json_str.to_string().parse::<TopoJson>();
        assert!(result.is_err());
        match result {
            Err(e) => assert_eq!(e, Error::ExpectedProperty(String::from("arcs"))),
            _ => panic!()
        }
    }

    #[test]
    fn encode_decode_geometry_with_position() {
        let geometry_json_str = "{\"coordinates\":[1.1,2.1],\"type\":\"Point\"}";
        let geometry = Geometry {
            value: Value::Point(vec![1.1, 2.1]),
            bbox: None,
            id: None,
            properties: None,
            foreign_members: None,
        };

        // Test encode
        let json_string = encode(&geometry);
        assert_eq!(json_string, geometry_json_str);

        // Test decode
        let decoded_geometry = match decode(json_string) {
            TopoJson::Geometry(g) => g,
            _ => unreachable!(),
        };
        assert_eq!(decoded_geometry, geometry);
    }

    #[test]
    fn encode_decode_geometry_with_arc_indexes_polygon() {
        let geometry_json_str =
            "{\"arcs\":[[1]],\"type\":\"Polygon\"}";
        let geometry = Geometry {
            value: Value::Polygon(vec![vec![1]]),
            bbox: None,
            id: None,
            properties: None,
            foreign_members: None,
        };

        // Test encode
        let json_string = encode(&geometry);
        assert_eq!(json_string, geometry_json_str);

        // Test decode
        let decoded_geometry = match decode(json_string) {
            TopoJson::Geometry(g) => g,
            _ => unreachable!(),
        };
        assert_eq!(decoded_geometry, geometry);
    }

    #[test]
    fn encode_decode_geometry_with_arc_indexes_linestring() {
        // let geometry_json_str =
        //     "{\"type\": \"Polygon\", \"properties\": {\"prop0": \"value0\",\"prop1\": {\"this\": \"that\"}},\"arcs\": [[1]]}";
        let geometry_json_str =
            "{\"arcs\":[0],\"type\":\"LineString\"}";
        let geometry = Geometry {
            value: Value::LineString(vec![0]),
            bbox: None,
            id: None,
            properties: None,
            foreign_members: None,
        };

        // Test encode
        let json_string = encode(&geometry);
        assert_eq!(json_string, geometry_json_str);

        // Test decode
        let decoded_geometry = match decode(json_string) {
            TopoJson::Geometry(g) => g,
            _ => unreachable!(),
        };
        assert_eq!(decoded_geometry, geometry);
    }


    #[test]
    fn encode_decode_geometry_with_foreign_member() {
        let geometry_json_str =
            "{\"coordinates\":[1.1,2.1],\"other_member\":true,\"type\":\"Point\"}";
        let mut foreign_members = JsonObject::new();
        foreign_members.insert(
            String::from("other_member"),
            serde_json::to_value(true).unwrap(),
        );
        let geometry = Geometry {
            value: Value::Point(vec![1.1, 2.1]),
            bbox: None,
            id: None,
            properties: None,
            foreign_members: Some(foreign_members),
        };

        // Test encode
        let json_string = encode(&geometry);
        assert_eq!(json_string, geometry_json_str);

        // Test decode
        let decoded_geometry = match decode(geometry_json_str.into()) {
            TopoJson::Geometry(g) => g,
            _ => unreachable!(),
        };
        assert_eq!(decoded_geometry, geometry);
    }

    #[test]
    fn encode_decode_geometry_with_properties() {
        let geometry_json_str =
            "{\"coordinates\":[1.1,2.1],\"properties\":{\"prop0\":0},\"type\":\"Point\"}";
        let mut properties = JsonObject::new();
        properties.insert(
            String::from("prop0"),
            serde_json::to_value(0).unwrap(),
        );
        let geometry = Geometry {
            value: Value::Point(vec![1.1, 2.1]),
            bbox: None,
            id: None,
            properties: Some(properties),
            foreign_members: None,
        };

        // Test encode
        let json_string = encode(&geometry);
        assert_eq!(json_string, geometry_json_str);

        // Test decode
        let decoded_geometry = match decode(geometry_json_str.into()) {
            TopoJson::Geometry(g) => g,
            _ => unreachable!(),
        };
        assert_eq!(decoded_geometry, geometry);
    }
    //
    #[test]
    fn encode_decode_geometry_collection() {
        // Properties for the geometry collection:
        let mut properties0 = JsonObject::new();
        properties0.insert(
            String::from("prop0"),
            serde_json::to_value(0).unwrap(),
        );
        // Properties for one the geometry in the geometry collection:
        let mut properties1 = JsonObject::new();
        properties1.insert(
            String::from("prop1"),
            serde_json::to_value(1).unwrap(),
        );
        let geometry_collection = Geometry {
            bbox: None,
            id: None,
            value: Value::GeometryCollection(vec![
                Geometry {
                    bbox: None,
                    id: None,
                    value: Value::Point(vec![100.0, 0.0]),
                    properties: Some(properties1),
                    foreign_members: None,
                },
                Geometry {
                    bbox: None,
                    id: None,
                    value: Value::LineString(vec![0]),
                    properties: None,
                    foreign_members: None,
                },
            ]),
            properties: Some(properties0),
            foreign_members: None,
        };

        let geometry_collection_string =
            "{\"geometries\":[{\"coordinates\":[100.0,0.0],\"properties\":{\"prop1\":1},\"type\":\"Point\"},{\"arcs\":[0],\"type\":\"LineString\"}],\"properties\":{\"prop0\":0},\"type\":\"GeometryCollection\"}";

        // Test encode
        let json_string = encode(&geometry_collection);
        assert_eq!(json_string, geometry_collection_string);

        // Test decode
        let decoded_geometry = match decode(geometry_collection_string.into()) {
            TopoJson::Geometry(g) => g,
            _ => unreachable!(),
        };
        assert_eq!(decoded_geometry, geometry_collection);
    }
}
