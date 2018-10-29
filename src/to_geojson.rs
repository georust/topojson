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

use geojson::{Feature, FeatureCollection, Geometry as GeoJsonGeometry, Value as GeoJsonGeomValue};
use { Error, Arc, Topology, Position, NamedGeometry, TransformParams, Geometry, Value as TopoJsonGeomValue };

fn decode_arc(arc: &[Position], tr: &Option<TransformParams>) -> Vec<Position> {
    match tr {
        None => {
            let mut ring = Vec::with_capacity(arc.len());
            for pt in arc {
                ring.push(pt.clone());
            }
            ring
        },
        Some(_tr) => {
            let (t0, t1, s0, s1) = (_tr.translate[0], _tr.translate[1], _tr.scale[0], _tr.scale[1]);
            let mut ring = Vec::with_capacity(arc.len());
            let (mut x, mut y) = (0., 0.);
            for pt in arc {
                let mut new_pt = pt.clone();
                x += new_pt[0];
                y += new_pt[1];
                new_pt[0] = x * s0 + t0;
                new_pt[1] = y * s1 + t1;
                ring.push(new_pt);
            }
            ring
        }
    }
}

fn make_pt(pos: &Vec<f64>, tr: &Option<TransformParams>) -> Vec<f64> {
    match tr {
        None => pos.clone(),
        Some(_tr) => {
            let mut new_pos = pos.clone();
            new_pos[0] = new_pos[0] * _tr.scale[0] + _tr.translate[0];
            new_pos[1] = new_pos[1] * _tr.scale[1] + _tr.translate[1];
            new_pos
        }
    }
}

pub fn convert_geom_coords(geom: &Geometry, tr: &Option<TransformParams>) -> Result<Feature, Error> {
    let geoj_geom_value = match &geom.value {
        TopoJsonGeomValue::Point(ref pos) => GeoJsonGeomValue::Point(make_pt(&pos, tr)),
        TopoJsonGeomValue::MultiPoint(positions) => GeoJsonGeomValue::MultiPoint(
            positions.iter()
                .map(|pos| make_pt(&pos, tr))
                .collect()),
        _ => unreachable!(),
    };
    Ok(Feature {
        bbox: geom.bbox.clone(),
        foreign_members: geom.foreign_members.clone(),
        geometry: Some(GeoJsonGeometry {
            bbox: None,
            foreign_members: None,
            value: geoj_geom_value,
        }),
        id: geom.id.clone(),
        properties: geom.properties.clone()
    })
}

pub fn make_ring(arcs: &[Arc], ixs: &[i32], tr: &Option<TransformParams>) -> Vec<Position> {
    let mut result_line = Vec::with_capacity(ixs.len());
    for _ix in ixs {
        let mut ix;
        let revert;
        if *_ix < 0 {
            revert = true;
            ix = (_ix.abs() - 1) as usize;
        } else {
            revert = false;
            ix = *_ix as usize;
        }
        let line_arc = &arcs[ix];
        let mut line = decode_arc(&line_arc, &tr);
        if revert { line.reverse(); }
        result_line.append(&mut line);
    }
    result_line
}

pub fn convert_geom_arcs(geom: &Geometry, arcs: &[Arc], tr: &Option<TransformParams>) -> Result<Feature, Error> {
    let geom_value = match &geom.value {
        TopoJsonGeomValue::LineString(ref arc_indexes) => {
            GeoJsonGeomValue::LineString(make_ring(&arcs, arc_indexes, &tr))
        },
        TopoJsonGeomValue::MultiLineString(arc_indexes) => {
            GeoJsonGeomValue::MultiLineString(arc_indexes.iter()
                .map(|ixs| make_ring(&arcs, ixs, &tr))
                .collect())
        },
        TopoJsonGeomValue::Polygon(arc_indexes) => {
            GeoJsonGeomValue::Polygon(arc_indexes.iter()
                .map(|ixs| make_ring(&arcs, ixs, &tr))
                .collect())
        },
        TopoJsonGeomValue::MultiPolygon(arcs_indexes) => {
            let mut polygons = Vec::with_capacity(arcs_indexes.len());
            for _arc_indexes_poly in arcs_indexes {
                polygons.push(_arc_indexes_poly.iter()
                    .map(|ixs| make_ring(&arcs, ixs, &tr))
                    .collect());
            }
            GeoJsonGeomValue::MultiPolygon(polygons)
        },
        _ => unreachable!(),
    };
    Ok(Feature{
        geometry: Some(GeoJsonGeometry {
            value: geom_value,
            bbox: None,
            foreign_members: None,
        }),
        bbox: geom.bbox.clone(),
        foreign_members: geom.foreign_members.clone(),
        id: geom.id.clone(),
        properties: geom.properties.clone()
    })
}

pub fn convert_geometry_collection(geom: &Geometry, arcs: &[Arc], tr: &Option<TransformParams>) -> Result<Vec<Feature>, Error> {
    let features = match geom.value {
        TopoJsonGeomValue::GeometryCollection(ref geoms) => {
            let mut features = Vec::with_capacity(geoms.len());
            for g in geoms.iter() {
                match &g.value {
                    TopoJsonGeomValue::Point(..) | TopoJsonGeomValue::MultiPoint(..) => {
                        features.push(convert_geom_coords(&g, &tr)?);
                    },
                    TopoJsonGeomValue::LineString(..) | TopoJsonGeomValue::MultiLineString(..)
                    | TopoJsonGeomValue::Polygon(..) | TopoJsonGeomValue::MultiPolygon(..) => {
                        features.push(convert_geom_arcs(&g, &arcs, &tr)?);
                    },
                    // According to https://github.com/topojson/topojson-client#feature
                    // a geometry collection of geometry collections should be mapped to
                    // a feature collection of features, each with a geometry collection.
                    _ => unimplemented!(),
                }
            }
            features
        },
        _ => unreachable!(),
    };
    Ok(features)
}

/// Convert a TopoJSON Topology object to a GeoJSON Feature collection.
///
/// (in a similar way than [topojson.feature](https://github.com/topojson/topojson-client#feature) function
/// or [topo2geo](https://github.com/topojson/topojson-client#topo2geo) CLI tool)
pub fn to_geojson(topo: &Topology, key: &str) -> Result<FeatureCollection, Error> {
    let objs: Vec<&NamedGeometry> = topo.objects
        .iter()
        .filter(|ng| ng.name == key)
        .collect();
    let features = match objs.len() {
        0 => return Err(Error::TopoToGeoUnknownKey(key.to_owned())),
        1 => {
            match &objs[0].geometry.value {
                TopoJsonGeomValue::Point(..) | TopoJsonGeomValue::MultiPoint(..) => {
                    vec![convert_geom_coords(&objs[0].geometry, &topo.transform)?]
                },
                TopoJsonGeomValue::LineString(..) | TopoJsonGeomValue::MultiLineString(..)
                | TopoJsonGeomValue::Polygon(..) | TopoJsonGeomValue::MultiPolygon(..) => {
                    vec![convert_geom_arcs(&objs[0].geometry, &topo.arcs, &topo.transform)?]
                },
                | TopoJsonGeomValue::GeometryCollection(..) => {
                    convert_geometry_collection(&objs[0].geometry, &topo.arcs, &topo.transform)?
                }
            }
        },
        _ => unreachable!(),
    };

    Ok(FeatureCollection {
        features: features,
        bbox: None,
        foreign_members: None,
    })
}
#[cfg(test)]
mod tests {
    use {TopoJson, Topology, to_geojson, Error};
    use geojson::GeoJson;

    fn decode(json_string: &str) -> TopoJson {
        json_string.parse().unwrap()
    }

    #[test]
    fn convert_fails_unknown_key() {
        let topo = decode("{\"arcs\":[[[2.2,2.2],[3.3,3.3]]],\"objects\":{\"example\":{\"arcs\":[0],\"type\":\"LineString\"}},\"type\":\"Topology\"}");
        let result = to_geojson(&Into::<Option<Topology>>::into(topo).unwrap(), "foo");

        assert!(result.is_err());
        match result {
            Err(e) => assert_eq!(e, Error::TopoToGeoUnknownKey("foo".to_string())),
            _ => panic!()
        }
    }

    #[test]
    fn convert_quantized_topology_example_specifications() {
        // This is the quantized example from https://github.com/topojson/topojson-specification#11-examples
        let topo_json_str = "{\"arcs\":[[[4000,0],[1999,9999],[2000,-9999],[2000,9999]],[[0,0],[0,9999],[2000,0],[0,-9999],[-2000,0]]],\"objects\":{\"example\":{\"geometries\":[{\"coordinates\":[4000,5000],\"properties\":{\"prop0\":\"value0\"},\"type\":\"Point\"},{\"arcs\":[0],\"properties\":{\"prop0\":\"value0\",\"prop1\":0},\"type\":\"LineString\"},{\"arcs\":[[1]],\"properties\":{\"prop0\":\"value0\",\"prop1\":{\"this\":\"that\"}},\"type\":\"Polygon\"}],\"type\":\"GeometryCollection\"}},\"type\":\"Topology\",\"transform\":{\"scale\":[0.0005000500050005,0.00010001000100010001],\"translate\":[100,0]}}";

        let decoded_topo = match decode(topo_json_str) {
            TopoJson::Topology(t) => t,
            _ => unreachable!(),
        };

        let geojson_obj = to_geojson(&decoded_topo, "example").expect("Unable to convert to GeoJson");
        let geojson_string = GeoJson::FeatureCollection(geojson_obj).to_string();

        // The expected result was obtained using [topo2geo CLI tool](https://github.com/topojson/topojson-client#command-line-reference)
        // (then parsed with rust to obtain the same field order / same rounding issues)
        let expected_geojson_string = "{\"features\":[{\"geometry\":{\"coordinates\":[102.000200020002,0.5000500050005],\"type\":\"Point\"},\"properties\":{\"prop0\":\"value0\"},\"type\":\"Feature\"},{\"geometry\":{\"coordinates\":[[102.000200020002,0.0],[102.999799979998,0.9999999999999999],[103.999899989999,0.0],[105.0,0.9999999999999999]],\"type\":\"LineString\"},\"properties\":{\"prop0\":\"value0\",\"prop1\":0},\"type\":\"Feature\"},{\"geometry\":{\"coordinates\":[[[100.0,0.0],[100.0,0.9999999999999999],[101.000100010001,0.9999999999999999],[101.000100010001,0.0],[100.0,0.0]]],\"type\":\"Polygon\"},\"properties\":{\"prop0\":\"value0\",\"prop1\":{\"this\":\"that\"}},\"type\":\"Feature\"}],\"type\":\"FeatureCollection\"}";

        assert_eq!(geojson_string, expected_geojson_string);
    }
}
