extern crate geojson;
extern crate serde_json;
extern crate topojson;

use geojson::GeoJson;
use std::fs;
use topojson::{to_geojson, TopoJson};

pub fn main() {
    let cases = vec![
        // (Input path, layer name, output path)
        // Topology file containing a Geometry collection of only Polygons :
        (
            "examples/commune_D974_2016_2-2.topojson",
            "commune_dep_974",
            "/tmp/communes.geojson",
        ),
        // Topology file containing a Geometry collection of Polygons and MultiPolygons:
        (
            "examples/nuts2-2013-data.topojson",
            "nuts2-2013-data",
            "/tmp/nuts2.geojson",
        ),
        // Topology file containing a Geometry collection of only MultiLineStrings :
        (
            "examples/nuts2_inner_border.topojson",
            "nuts2_inner_border",
            "/tmp/nuts2_inner_border.geojson",
        ),
    ];
    for case in cases {
        let file_contents = fs::read_to_string(case.0).expect("Unable to read file");
        let topo = file_contents.parse::<TopoJson>().expect("unable to parse");

        match topo {
            TopoJson::Topology(t) => {
                let geojson = to_geojson(&t, &String::from(case.1))
                    .expect("Unable to convert TopoJSON to GeoJSON");
                let geojson_string = GeoJson::FeatureCollection(geojson).to_string();
                fs::write(case.2, geojson_string).expect("Unable to write file");
            }
            _ => unimplemented!(),
        };
    }
}
