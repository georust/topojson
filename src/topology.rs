use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson, Object, Array};
use geojson::{FeatureCollection, Feature, Pos};
use {TopoJsonResult, TopoJsonError};

pub struct Topology(BTreeMap<String, FeatureCollection>);
// TODO TopoJSON can have any top-level geometry in an object, it
// doesn't have to be a FeatureCollection (which is spelled
// GeometryCollection in TopoJSON).


struct Space {
    arcs: Vec<Vec<Pos>>,
}

impl Space {
    fn from_topology(topo: &Object) -> TopoJsonResult<Space> {
        let mut arcs = vec![];
        for arc_json in expect_array!(expect_property!(topo, "arcs", "Missing 'arcs' field")).iter() {
            let mut arc = vec![];
            for pos_json in expect_array!(arc_json).iter() {
                arc.push(Pos::from_json(expect_array!(pos_json)));
            }
            arcs.push(arc);
        }
        return Ok(Space{arcs: arcs});
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
            arc.iter().rev().map(|pos| pos.to_json()).collect()
        } else {
            arc.iter().map(|pos| pos.to_json()).collect()
        };
        return Ok(Json::Array(arcs_json));
    }
}


fn transform_coordinates(topo_coord: &Array) -> Json {
    return Json::Array(topo_coord.clone());
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
            transform_coordinates(expect_array!(topo_coord))
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
}
