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

use json::{Deserialize, Deserializer, JsonObject, Serialize, Serializer};

use serde_json;

use {util, Bbox, Error, Arc, NamedGeometry, TopoJson};

/// Transforms
///
/// [TopoJSON Format Specification § 2.1.2](https://github.com/topojson/topojson-specification#212-transforms)
#[derive(Clone, Debug, PartialEq)]
pub struct TransformParams {
    pub scale: [f64; 2],
    pub translate: [f64; 2],
}

impl<'a> From<&'a TransformParams> for JsonObject {
    fn from(transform: &'a TransformParams) -> JsonObject {
        let mut map = JsonObject::new();
        map.insert(String::from("scale"), ::serde_json::to_value(&transform.scale).unwrap());
        map.insert(String::from("translate"), ::serde_json::to_value(&transform.translate).unwrap());
        map
    }
}

impl TransformParams {
    pub fn from_json_object(mut object: JsonObject) -> Result<Self, Error> {
        let scale_translate = util::get_scale_translate(&mut object)?;
        Ok(scale_translate.unwrap())
    }
}

impl Serialize for TransformParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        JsonObject::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TransformParams {
    fn deserialize<D>(deserializer: D) -> Result<TransformParams, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as SerdeError;
        use std::error::Error as StdError;

        let val = JsonObject::deserialize(deserializer)?;

        TransformParams::from_json_object(val).map_err(|e| D::Error::custom(e.description()))
    }
}

/// Topology object
///
/// [TopoJSON Format Specification § 2.1](https://github.com/topojson/topojson-specification#21-topology-objects)
#[derive(Clone, Debug, PartialEq)]
pub struct Topology {
    pub bbox: Option<Bbox>,
    pub objects: Vec<NamedGeometry>,
    pub transform: Option<TransformParams>,
    pub arcs: Vec<Arc>,
    // /// Foreign Members
    // pub foreign_members: Option<JsonObject>,
}

impl<'a> From<&'a Topology> for JsonObject {
    fn from(topo: &'a Topology) -> JsonObject {
        let mut map = JsonObject::new();
        map.insert(String::from("type"), json!("Topology"));
        map.insert(String::from("arcs"), serde_json::to_value(&topo.arcs).unwrap());

        let mut objects = JsonObject::new();
        for named_geom in topo.objects.iter() {
            objects.insert(named_geom.name.clone(), serde_json::to_value(named_geom.geometry.clone()).unwrap());
        }
        map.insert(String::from("objects"), serde_json::Value::Object(objects));

        if let Some(ref bbox) = topo.bbox {
            map.insert(String::from("bbox"), serde_json::to_value(bbox).unwrap());
        }

        if let Some(ref transform_params) = topo.transform {
            map.insert(String::from("transform"), serde_json::to_value(transform_params).unwrap());
        }
        // if let Some(ref foreign_members) = topo.foreign_members {
        //     for (key, value) in foreign_members {
        //         map.insert(key.to_owned(), value.to_owned());
        //     }
        // }

        map
    }
}

impl Topology {
    pub fn from_json_object(mut object: JsonObject) -> Result<Self, Error> {
        match util::expect_type(&mut object)? {
            ref type_ if type_ == "Topology" => Ok(Topology {
                bbox: util::get_bbox(&mut object)?,
                objects: util::get_objects(&mut object)?,
                transform: util::get_scale_translate(&mut object)?,
                arcs: util::get_arcs_position(&mut object)?
                // foreign_members: util::get_foreign_members(object)?,
            }),
            type_ => Err(Error::ExpectedType {
                expected: "Topology".to_owned(),
                actual: type_,
            }),
        }
    }

    pub fn list_names(&self) -> Vec<String> {
        self.objects
            .iter()
            .cloned()
            .map(|g| g.name)
            .collect::<Vec<String>>()
    }
}

impl Serialize for Topology {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        JsonObject::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Topology {
    fn deserialize<D>(deserializer: D) -> Result<Topology, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as SerdeError;
        use std::error::Error as StdError;

        let val = JsonObject::deserialize(deserializer)?;

        Topology::from_json_object(val).map_err(|e| D::Error::custom(e.description()))
    }
}

impl Into<Option<Topology>> for TopoJson {
    fn into(self) -> Option<Topology> {
        match self {
            TopoJson::Topology(i) => Some(i),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json;
    use {TopoJson, Geometry, NamedGeometry, Value, Topology, TransformParams, Error};
    use json::JsonObject;

    fn encode(topo: &Topology) -> String {
        serde_json::to_string(&topo).unwrap()
    }

    fn decode(json_string: String) -> TopoJson {
        json_string.parse().unwrap()
    }

    #[test]
    fn encode_decode_topology_arcs() {
        let topo_json_str = "{\"arcs\":[[[2.2,2.2],[3.3,3.3]]],\"objects\":{},\"type\":\"Topology\"}";
        let topo = Topology {
            arcs: vec![
                vec![vec![2.2, 2.2], vec![3.3, 3.3]]
            ],
            objects: vec![],
            bbox: None,
            transform: None,
        };

        // Test encode
        let json_string = encode(&topo);
        assert_eq!(json_string, topo_json_str);

        // Test decode
        let decoded_topo = match decode(json_string) {
            TopoJson::Topology(t) => t,
            _ => unreachable!(),
        };
        assert_eq!(decoded_topo, topo);
    }

    #[test]
    fn decode_invalid_topology_no_objects() {
        let topo_json_str = "{\"arcs\":[[[2.2,2.2],[3.3,3.3]]],\"type\":\"Topology\"}";

        // Decode should fail due to the absence of the 'objects' member:
        let result = topo_json_str.to_string().parse::<TopoJson>();
        assert!(result.is_err());
        match result {
            Err(e) => assert_eq!(e, Error::TopologyExpectedObjects),
            _ => panic!()
        }
    }

    #[test]
    fn decode_invalid_topology_no_arcs() {
        let topo_json_str = "{\"objects\":{},\"type\":\"Topology\"}";

        // Decode should fail due to the absence of the 'objects' member:
        let result = topo_json_str.to_string().parse::<TopoJson>();
        assert!(result.is_err());
        match result {
            Err(e) => assert_eq!(e, Error::TopologyExpectedArcs),
            _ => panic!()
        }
    }

    #[test]
    fn decode_invalid_topology_bad_type() {
        let topo_json_str = "{\"arcs\":[[[2.2,2.2],[3.3,3.3]]],\"type\":\"foo\",\"objects\":{\"example\":{\"arcs\":[0],\"type\":\"LineString\"}}}";

        // Decode should fail due to the absence of the 'objects' member:
        let result = topo_json_str.to_string().parse::<TopoJson>();
        assert!(result.is_err());
        match result {
            Err(e) => assert_eq!(e, Error::TopoJsonUnknownType),
            _ => panic!()
        }
    }

    #[test]
    fn list_names_objects() {
        let topo = Topology {
            arcs: vec![
                vec![vec![2.2, 2.2], vec![3.3, 3.3]]
            ],
            objects: vec![NamedGeometry {
                name: String::from("example"),
                geometry: Geometry {
                    value: Value::LineString(vec![0]),
                    bbox: None,
                    id: None,
                    properties: None,
                    foreign_members: None,
                },
            }],
            bbox: None,
            transform: None,
        };
        let names = topo.list_names();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "example");
    }

    #[test]
    fn encode_decode_topology_arcs_object() {
        let topo_json_str = "{\"arcs\":[[[2.2,2.2],[3.3,3.3]]],\"objects\":{\"example\":{\"arcs\":[0],\"type\":\"LineString\"}},\"type\":\"Topology\"}";
        let topo = Topology {
            arcs: vec![
                vec![vec![2.2, 2.2], vec![3.3, 3.3]]
            ],
            objects: vec![NamedGeometry {
                name: String::from("example"),
                geometry: Geometry {
                    value: Value::LineString(vec![0]),
                    bbox: None,
                    id: None,
                    properties: None,
                    foreign_members: None,
                },
            }],
            bbox: None,
            transform: None,
        };

        // Test encode
        let json_string = encode(&topo);
        assert_eq!(json_string, topo_json_str);

        // Test decode
        let decoded_topo = match decode(json_string) {
            TopoJson::Topology(t) => t,
            _ => unreachable!(),
        };
        assert_eq!(decoded_topo, topo);
    }
    #[test]
    fn encode_decode_topology_arcs_transform() {
        let topo_json_str = "{\"arcs\":[[[2.2,2.2],[3.3,3.3]]],\"objects\":{},\"transform\":{\"scale\":[0.12,0.12],\"translate\":[1.1,1.1]},\"type\":\"Topology\"}";
        let topo = Topology {
            arcs: vec![
                vec![vec![2.2, 2.2], vec![3.3, 3.3]]
            ],
            objects: vec![],
            bbox: None,
            transform: Some(TransformParams {
                scale: [0.12, 0.12],
                translate: [1.1, 1.1],
            }),
        };

        // Test encode
        let json_string = encode(&topo);
        assert_eq!(json_string, topo_json_str);

        // Test decode
        let decoded_topo = match decode(json_string) {
            TopoJson::Topology(t) => t,
            _ => unreachable!(),
        };
        assert_eq!(decoded_topo, topo);
    }


    #[test]
    fn encode_decode_topology_arcs_transform_geom_collection() {
        let topo_json_str = "{\"arcs\":[[[2.2,2.2],[3.3,3.3]]],\"objects\":{\"example\":{\"geometries\":[{\"coordinates\":[100.0,0.0],\"properties\":{\"prop1\":1},\"type\":\"Point\"},{\"arcs\":[0],\"type\":\"LineString\"}],\"properties\":{\"prop0\":0},\"type\":\"GeometryCollection\"}},\"transform\":{\"scale\":[0.12,0.12],\"translate\":[1.1,1.1]},\"type\":\"Topology\"}";
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

        let topo = Topology {
            arcs: vec![
                vec![vec![2.2, 2.2], vec![3.3, 3.3]]
            ],
            objects: vec![
                NamedGeometry {
                    name: String::from("example"),
                    geometry: Geometry {
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
                    },
                },
            ],
            bbox: None,
            transform: Some(TransformParams {
                scale: [0.12, 0.12],
                translate: [1.1, 1.1],
            }),
        };

        // Test encode
        let json_string = encode(&topo);
        assert_eq!(json_string, topo_json_str);

        // Test decode
        let decoded_topo = match decode(json_string) {
            TopoJson::Topology(t) => t,
            _ => unreachable!(),
        };
        assert_eq!(decoded_topo, topo);
    }

    #[test]
    fn encode_decode_topology_example_quantized_specifications() {
        let topo_json_str = "{\"arcs\":[[[4000.0,0.0],[1999.0,9999.0],[2000.0,-9999.0],[2000.0,9999.0]],[[0.0,0.0],[0.0,9999.0],[2000.0,0.0],[0.0,-9999.0],[-2000.0,0.0]]],\"objects\":{\"example\":{\"geometries\":[{\"coordinates\":[4000.0,5000.0],\"properties\":{\"prop0\":\"value0\"},\"type\":\"Point\"},{\"arcs\":[0],\"properties\":{\"prop0\":\"value0\",\"prop1\":0},\"type\":\"LineString\"},{\"arcs\":[[1]],\"properties\":{\"prop0\":\"value0\",\"prop1\":{\"this\":\"that\"}},\"type\":\"Polygon\"}],\"type\":\"GeometryCollection\"}},\"transform\":{\"scale\":[0.0005000500050005,0.0001000100010001],\"translate\":[100.0,0.0]},\"type\":\"Topology\"}";

        // Properties for the 1st geometry of the geometry collection:
        let mut properties0 = JsonObject::new();
        properties0.insert(
            String::from("prop0"),
            serde_json::to_value("value0").unwrap(),
        );
        // Properties for the 2nd geometry of the geometry collection:
        let mut properties1 = properties0.clone();
        properties1.insert(
            String::from("prop1"),
            serde_json::to_value(0).unwrap(),
        );
        // Properties for the 3rd geometry of the geometry collection:
        let mut properties2 = properties0.clone();
        let mut inner_prop = JsonObject::new();
        inner_prop.insert(
            String::from("this"),
            serde_json::to_value(String::from("that")).unwrap(),
        );
        properties2.insert(
            String::from("prop1"),
            serde_json::to_value(inner_prop).unwrap(),
        );

        let topo = Topology {
            bbox: None,
            objects: vec![
                NamedGeometry {
                    name: String::from("example"),
                    geometry: Geometry {
                        bbox: None,
                        id: None,
                        value: Value::GeometryCollection(vec![
                            Geometry {
                                bbox: None,
                                id: None,
                                value: Value::Point(vec![4000.0, 5000.0]),
                                properties: Some(properties0),
                                foreign_members: None
                            },
                            Geometry {
                                bbox: None,
                                id: None,
                                value: Value::LineString(vec![0]),
                                properties: Some(properties1),
                                foreign_members: None
                            },
                            Geometry {
                                bbox: None,
                                id: None,
                                value: Value::Polygon(vec![vec![1]]),
                                properties: Some(properties2),
                                foreign_members: None
                            }
                        ]),
                        properties: None,
                        foreign_members: None
                    }
                }],
            transform: Some(TransformParams {
                scale: [0.0005000500050005, 0.0001000100010001],
                translate: [100.0, 0.0]
            }),
            arcs: vec![
                vec![
                    vec![4000.0, 0.0], vec![1999.0, 9999.0],
                    vec![2000.0, -9999.0], vec![2000.0, 9999.0]],
                vec![
                    vec![0.0, 0.0], vec![0.0, 9999.0],
                    vec![2000.0, 0.0], vec![0.0, -9999.0], vec![-2000.0, 0.0]]
            ],
        };

        // Test encode
        let json_string = encode(&topo);
        assert_eq!(json_string, topo_json_str);

        // Test decode
        let decoded_topo = match decode(json_string) {
            TopoJson::Topology(t) => t,
            _ => unreachable!(),
        };
        assert_eq!(decoded_topo, topo);
    }
}
