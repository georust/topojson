use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson, Object, Array};
use geojson::{FeatureCollection, Feature};
use {TopoJsonResult, TopoJsonError};

pub struct Topology(BTreeMap<String, FeatureCollection>);
// TODO TopoJSON can have any top-level geometry in an object, it
// doesn't have to be a FeatureCollection (which is spelled
// GeometryCollection in TopoJSON).

fn parse_transform(json: &Json) -> TopoJsonResult<(f64, f64, f64, f64)> {
    let transform = expect_object!(json);
    let translate = expect_array!(expect_property!(transform, "translate", "Missing 'translate' field"));
    let scale = expect_array!(expect_property!(transform, "scale", "Missing 'scale' field"));
    match (translate.len(), scale.len()) {
        (2, 2) => (),
        _ => return Err(TopoJsonError::new("Both 'translate' and 'scale' must be two-element arrays"))
    };
    let t0 = expect_f64!(translate[0]);
    let t1 = expect_f64!(translate[1]);
    let s0 = expect_f64!(scale[0]);
    let s1 = expect_f64!(scale[1]);
    return Ok((t0, t1, s0, s1));
}

struct Space {
    arcs: Vec<Vec<Json>>,
    transform: Option<(f64, f64, f64, f64)>,
}

impl Space {
    fn from_topology(topo: &Object) -> TopoJsonResult<Space> {
        let transform = match topo.get("transform") {
            Some(transform_json) => Some(try!(parse_transform(transform_json))),
            None => None
        };
        let mut arcs = vec![];
        for arc_json in expect_array!(expect_property!(topo, "arcs", "Missing 'arcs' field")).iter() {
            let arc = match transform {
                Some((t0, t1, s0, s1)) => {
                    let mut arc = vec![];
                    let mut x = 0;
                    let mut y = 0;
                    for point_json in expect_array!(arc_json).iter() {
                        let mut p = expect_array!(point_json).clone();
                        if p.len() < 2 {
                            return Err(TopoJsonError::new("Point must have at least two elements"));
                        }
                        x += expect_i64!(p[0]);
                        y += expect_i64!(p[1]);
                        p[0] = Json::F64((x as f64) * s0 + t0);
                        p[1] = Json::F64((y as f64) * s1 + t1);
                        arc.push(Json::Array(p));
                    }
                    arc
                },
                None => expect_array!(arc_json).clone()
            };
            arcs.push(arc);
        }
        return Ok(Space{arcs: arcs, transform: transform});
    }

    pub fn point(&self, p: &Vec<Json>) -> TopoJsonResult<Vec<Json>> {
        if p.len() < 2 {
            return Err(TopoJsonError::new("Point must have at least two elements"));
        }
        let mut rv = p.clone();
        println!("{:?}", self.transform);
        match self.transform {
            Some((t0, t1, s0, s1)) => {
                rv[0] = Json::F64(expect_f64!(rv[0]) * s0 + t0);
                rv[1] = Json::F64(expect_f64!(rv[1]) * s1 + t1);
            },
            None => ()
        };
        return Ok(rv);
    }

    pub fn arc(&self, arc_index: isize) -> TopoJsonResult<Json> {
        let (inverse, offset) = if arc_index > -1 {
            (false, arc_index as usize)
        } else {
            (true, -1 - arc_index as usize)
        };
        if offset >= self.arcs.len() {
            println!("offset = {}", offset);
            return Err(TopoJsonError::new("Arc index too large"));
        }
        let arc = &self.arcs[offset];
        let arcs_json = if inverse {
            arc.iter().rev().map(|p| p.to_json()).collect()
        } else {
            arc.iter().map(|p| p.to_json()).collect()
        };
        return Ok(Json::Array(arcs_json));
    }
}


fn transform_coordinates(topo_coord: &Array, space: &Space) -> TopoJsonResult<Json> {
    return Ok(Json::Array(try!(space.point(topo_coord))));
}

fn transform_arcs(arcs: &Array, space: &Space, depth: usize) -> TopoJsonResult<Json> {
    let mut rv = vec![];
    for arc in arcs.iter() {
        println!("{:?}", arc);
        if depth > 1 {
            rv.push(try!(transform_arcs(expect_array!(arc), space, depth - 1)));
        }
        else {
            rv.push(try!(space.arc(expect_i64!(arc) as isize)));
        }
    }
    if depth == 1 {
        rv = match rv.pop() {
            Some(Json::Array(v)) => v,
            _ => return Err(TopoJsonError::new("Empty arc list"))
        }
    }
    return Ok(Json::Array(rv));
}

fn parse_geometry(object: &Object, space: &Space) -> TopoJsonResult<Feature> {
    let geometry_type = expect_string!(expect_property!(object, "type", "Missing 'type' field"));
    let properties = expect_property!(object, "properties", "Missing 'properties' field");

    let mut geojson_geometry = BTreeMap::new();
    geojson_geometry.insert("type".to_string(), Json::String(geometry_type.to_string()));
    let coordinates = match geometry_type {
        "Point" => {
            let topo_coord = expect_property!(object, "coordinates", "Missing 'coordinates' property");
            try!(transform_coordinates(expect_array!(topo_coord), space))
        },
        "LineString" | "Polygon" => {
            let arcs = expect_property!(object, "arcs", "Missing 'arcs' property");
            println!("arcs! {:?}", arcs);
            let depth = match geometry_type {
                "LineString" => 1,
                "Polygon" => 2,
                _ => unreachable!()
            };
            try!(transform_arcs(expect_array!(arcs), space, depth))
        },
        _ => return Err(TopoJsonError::new("Unknown geometry type"))
    };
    geojson_geometry.insert("coordinates".to_string(), coordinates);

    let mut geojson_feature = BTreeMap::new();
    geojson_feature.insert("type".to_string(), Json::String("Feature".to_string()));
    geojson_feature.insert("geometry".to_string(), Json::Object(geojson_geometry));
    geojson_feature.insert("properties".to_string(), properties.clone());
    println!("feature: {}", Json::Object(geojson_feature.clone()));
    return Ok(Feature::from_json(&geojson_feature));
}


fn parse_geometry_collection(object: &Object, space: &Space) -> TopoJsonResult<FeatureCollection> {
    let geometries = expect_array!(expect_property!(object, "geometries", "Missing 'geometries' field"));
    let mut features = vec![];
    for geometry_json in geometries.iter() {
        features.push(try!(parse_geometry(expect_object!(geometry_json), space)));
    }
    return Ok(FeatureCollection{features: features});
}


impl Topology {
    pub fn from_json(json: &Json) -> TopoJsonResult<Topology> {
        let topo = expect_object!(json);
        match expect_property!(topo, "type", "Missing 'type' field").as_string() {
            Some("Topology") => (),
            _ => return Err(TopoJsonError::new("'type' should be 'Topology'"))
        };
        let space = try!(Space::from_topology(topo));
        let objects = expect_object!(expect_property!(topo, "objects", "Missing 'objects' field"));
        let mut map = BTreeMap::new();
        for (name, value) in objects.iter() {
            let object = expect_object!(value);
            let object_type = expect_string!(expect_property!(object, "type", "Missing 'type' field"));
            let geojson = match object_type {
                "GeometryCollection" => try!(parse_geometry_collection(object, &space)),
                _ => return Err(TopoJsonError::new("'type' should be 'GeometryCollection'"))
            };
            map.insert(name.to_string(), geojson);
        }
        return Ok(Topology(map));
    }

    pub fn object(&self, name: &str) -> Option<&FeatureCollection> {
        let &Topology(ref hash) = self;
        return hash.get(name);
    }
}


#[cfg(test)]
mod tests {
    use rustc_serialize::json::{Json, ToJson};
    use super::Topology;

    #[test]
    fn test_invalid_topo() {
        fn expect_err(json: &str) {
            match Topology::from_json(&Json::from_str(json).unwrap()) {
                Err(_) => (),
                _ => panic!()
            }
        }
        expect_err("{}");
        expect_err(r#"{"type": "foo"}"#);
        expect_err(r#"{"type": "Topology"}"#);
    }

    #[test]
    fn test_parse_simple_topology_from_spec() {
        let topo_str = r#"{"type":"Topology","objects":{"example":{"type":"GeometryCollection","geometries":[{"type":"Point","properties":{"prop0":"value0"},"coordinates":[102,0.5]},{"type":"LineString","properties":{"prop0":"value0","prop1":0},"arcs":[0]},{"type":"Polygon","properties":{"prop0":"value0","prop1":{"this":"that"}},"arcs":[[-2]]}]}},"arcs":[[[102,0],[103,1],[104,0],[105,1]],[[100,0],[101,0],[101,1],[100,1],[100,0]]]}"#;
        let topo_json = Json::from_str(topo_str).unwrap();
        let topology = match Topology::from_json(&topo_json) {
            Ok(t) => t,
            Err(e) => panic!(e.desc)
        };
        let example = topology.object("example").unwrap();
        let geojson = format!("{}", example.to_json());
        assert_eq!(geojson, r#"{"features":[{"geometry":{"coordinates":[102.0,0.5],"type":"Point"},"properties":{"prop0":"value0"},"type":"Feature"},{"geometry":{"coordinates":[[102.0,0.0],[103.0,1.0],[104.0,0.0],[105.0,1.0]],"type":"LineString"},"properties":{"prop0":"value0","prop1":0},"type":"Feature"},{"geometry":{"coordinates":[[[100.0,0.0],[100.0,1.0],[101.0,1.0],[101.0,0.0],[100.0,0.0]]],"type":"Polygon"},"properties":{"prop0":"value0","prop1":{"this":"that"}},"type":"Feature"}],"type":"FeatureCollection"}"#);
    }

    #[test]
    fn test_parse_quantized_topology_from_spec() {
        let topo_str = r#"{"objects":{"example":{"type":"GeometryCollection","geometries":[{"type":"Point","properties":{"prop0":"value0"},"coordinates":[4000,5000]},{"type":"LineString","properties":{"prop0":"value0","prop1":0},"arcs":[0]},{"type":"Polygon","properties":{"prop0":"value0","prop1":{"this":"that"}},"arcs":[[1]]}]}},"type":"Topology","transform":{"translate":[100,0],"scale":[0.0005000500050005,0.00010001000100010001]},"arcs":[[[4000,0],[1999,9999],[2000,-9999],[2000,9999]],[[0,0],[0,9999],[2000,0],[0,-9999],[-2000,0]]]}"#;
        let topo_json = Json::from_str(topo_str).unwrap();
        let topology = match Topology::from_json(&topo_json) {
            Ok(t) => t,
            Err(e) => panic!(e.desc)
        };
        let example = topology.object("example").unwrap();
        let geojson = format!("{}", example.to_json());
        assert_eq!(geojson, r#"{"features":[{"geometry":{"coordinates":[102.0002,0.50005],"type":"Point"},"properties":{"prop0":"value0"},"type":"Feature"},{"geometry":{"coordinates":[[102.0002,0.0],[102.9998,1.0],[103.9999,0.0],[105.0,1.0]],"type":"LineString"},"properties":{"prop0":"value0","prop1":0},"type":"Feature"},{"geometry":{"coordinates":[[[100.0,0.0],[100.0,1.0],[101.0001,1.0],[101.0001,0.0],[100.0,0.0]]],"type":"Polygon"},"properties":{"prop0":"value0","prop1":{"this":"that"}},"type":"Feature"}],"type":"FeatureCollection"}"#);
    }
}
