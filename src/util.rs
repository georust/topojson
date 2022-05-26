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

use crate::json::{JsonObject, JsonValue};
use crate::{Arc, ArcIndexes, Bbox, Error, Geometry, NamedGeometry, Position, TransformParams};

pub fn expect_type(value: &mut JsonObject) -> Result<String, Error> {
    let prop = expect_property(value, "type")?;
    expect_string(prop)
}

fn expect_string(value: JsonValue) -> Result<String, Error> {
    match value {
        JsonValue::String(s) => Ok(s),
        _ => Err(Error::ExpectedStringValue),
    }
}

pub fn expect_f64(value: &JsonValue) -> Result<f64, Error> {
    match value.as_f64() {
        Some(v) => Ok(v),
        None => Err(Error::ExpectedF64Value),
    }
}

pub fn expect_i32(value: &JsonValue) -> Result<i32, Error> {
    match value.as_i64() {
        Some(v) => Ok(v as i32),
        None => Err(Error::Expectedi32Value),
    }
}

pub fn expect_array(value: &JsonValue) -> Result<&Vec<JsonValue>, Error> {
    match value.as_array() {
        Some(v) => Ok(v),
        None => Err(Error::ExpectedArrayValue),
    }
}

pub fn expect_object(value: &JsonValue) -> Result<&JsonObject, Error> {
    match value.as_object() {
        Some(v) => Ok(v),
        None => Err(Error::ExpectedObjectValue),
    }
}

fn expect_property(obj: &mut JsonObject, name: &'static str) -> Result<JsonValue, Error> {
    match obj.remove(name) {
        Some(v) => Ok(v),
        None => Err(Error::ExpectedProperty(name.to_string())),
    }
}

pub fn get_foreign_members(object: JsonObject) -> Result<Option<JsonObject>, Error> {
    if object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(object))
    }
}

fn get_coords_value(object: &mut JsonObject) -> Result<JsonValue, Error> {
    expect_property(object, "coordinates")
}

fn get_arcs_value(object: &mut JsonObject) -> Result<JsonValue, Error> {
    expect_property(object, "arcs")
}

/// Retrieve the 'arcs' member of a Topology.
///
/// Used by Topology.
pub fn get_arcs_position(object: &mut JsonObject) -> Result<Vec<Arc>, Error> {
    match object.remove("arcs") {
        Some(a) => json_to_arc_positions(&a),
        None => Err(Error::TopologyExpectedArcs),
    }
}

/// Retrieve a single Position from the value of the "coordinates" key.
///
/// Used by Value::Point
pub fn get_coords_one_pos(object: &mut JsonObject) -> Result<Position, Error> {
    let coords_json = get_coords_value(object)?;
    json_to_position(&coords_json)
}

/// Retrieve a one dimensional Vec of Positions from the value of the "coordinates" key.
///
/// Used by Value::MultiPoint
pub fn get_coords_1d_pos(object: &mut JsonObject) -> Result<Vec<Position>, Error> {
    let coords_json = get_coords_value(object)?;
    json_to_1d_positions(&coords_json)
}

/// Retrieve an ArcIndexes from the value of the "arcs" member of a Geometry.
///
/// Used by Value::LineString
pub fn get_arc_ix(object: &mut JsonObject) -> Result<ArcIndexes, Error> {
    let arc_indexes_json = get_arcs_value(object)?;
    json_to_arc_indexes(&arc_indexes_json)
}

/// Retrieve a one dimensional Vec of ArcIndexes from the value
/// of the 'arcs' member of a Geometry.
///
/// Used by Value::MultiLineString and Value::Polygon
pub fn get_arc_ix_1d(object: &mut JsonObject) -> Result<Vec<ArcIndexes>, Error> {
    let arc_indexes_json = get_arcs_value(object)?;
    json_to_1d_arc_indexes(&arc_indexes_json)
}

/// Retrieve a two dimensional Vec of ArcIndexes from the value
/// of the 'arcs' member of a Geometry.
///
/// Used by Value::MultiPolygon
pub fn get_arc_ix_2d(object: &mut JsonObject) -> Result<Vec<Vec<ArcIndexes>>, Error> {
    let arc_indexes_json = get_arcs_value(object)?;
    json_to_2d_arc_indexes(&arc_indexes_json)
}

/// Retrieve the geometries contained in the 'geometries' member of a GeometryCollection.
///
/// Used by Value::GeometryCollection
pub fn get_geometries(object: &mut JsonObject) -> Result<Vec<Geometry>, Error> {
    let geometries_json = expect_property(object, "geometries")?;
    let geometries_array = expect_owned_array(geometries_json)?;
    let mut geometries = Vec::with_capacity(geometries_array.len());
    for json in geometries_array {
        let obj = expect_owned_object(json)?;
        let geometry = Geometry::from_json_object(obj)?;
        geometries.push(geometry);
    }
    Ok(geometries)
}

/// Used by Geometry
pub fn get_id(object: &mut JsonObject) -> Result<Option<JsonValue>, Error> {
    Ok(object.remove("id"))
}

/// Used by Topology and Geometry
pub fn get_bbox(object: &mut JsonObject) -> Result<Option<Bbox>, Error> {
    let bbox_json = match object.remove("bbox") {
        Some(b) => b,
        None => return Ok(None),
    };
    let bbox_array = match bbox_json {
        JsonValue::Array(a) => a,
        _ => return Err(Error::BboxExpectedArray),
    };
    let bbox = bbox_array
        .into_iter()
        .map(|i| i.as_f64().ok_or(Error::BboxExpectedNumericValues))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(bbox))
}

/// Retrieve the Transforms used by the Topology if any.
///
/// Used by Topology
pub fn get_scale_translate(object: &mut JsonObject) -> Result<Option<TransformParams>, Error> {
    match object.remove("transform") {
        None => Ok(None),
        Some(b) => {
            let tr_json = expect_object(&b)?;
            let scale_json = match tr_json.get("scale") {
                Some(b) => b,
                None => return Err(Error::TransformExpectedScale),
            };
            let scale_array = match scale_json {
                JsonValue::Array(a) => a,
                _ => return Err(Error::ScaleExpectedArray),
            };
            let scale = scale_array
                .iter()
                .map(|i| i.as_f64().ok_or(Error::ScaleExpectedNumericValues))
                .collect::<Result<Vec<_>, _>>()?;

            let translate_json = match tr_json.get("translate") {
                Some(b) => b,
                None => return Err(Error::TransformExpectedTranslate),
            };
            let translate_array = match translate_json {
                JsonValue::Array(a) => a,
                _ => return Err(Error::TranslateExpectedArray),
            };
            let translate = translate_array
                .iter()
                .map(|i| i.as_f64().ok_or(Error::TranslateExpectedNumericValues))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Some(TransformParams {
                scale: [scale[0], scale[1]],
                translate: [translate[0], translate[1]],
            }))
        }
    }
}

/// Retrieve the 'properties' member of a Geometry if any.
///
/// Used by Geometry
pub fn get_properties(object: &mut JsonObject) -> Result<Option<JsonObject>, Error> {
    match object.remove("properties") {
        // If their is any 'properties' member, it must be an Object:
        Some(JsonValue::Object(properties)) => Ok(Some(properties)),
        // Null is handled as if their is no 'properties' member:
        Some(JsonValue::Null) | None => Ok(None),
        _ => Err(Error::PropertiesExpectedObjectOrNull),
    }
}

/// Retrieve the 'objects' member of a Topololy.
///
/// Used by Topology
pub fn get_objects(object: &mut JsonObject) -> Result<Vec<NamedGeometry>, Error> {
    match object.remove("objects") {
        // The ' member must exists and must be an Object:
        Some(JsonValue::Object(ref mut objects_json)) => {
            let keys: Vec<String> = objects_json.keys().map(|a| a.to_owned()).collect();
            let mut res = Vec::with_capacity(keys.len());
            for key in keys {
                let g = expect_owned_object(objects_json.remove(&key).unwrap())?;
                res.push(NamedGeometry {
                    name: key,
                    geometry: Geometry::from_json_object(g)?,
                });
            }
            Ok(res)
        }
        Some(_) | None => Err(Error::TopologyExpectedObjects),
    }
}

fn json_to_position(json: &JsonValue) -> Result<Position, Error> {
    let coords_array = expect_array(json)?;
    let mut coords = Vec::with_capacity(coords_array.len());
    for position in coords_array {
        coords.push(expect_f64(position)?);
    }
    Ok(coords)
}

fn json_to_1d_positions(json: &JsonValue) -> Result<Vec<Position>, Error> {
    let coords_array = expect_array(json)?;
    let mut coords = Vec::with_capacity(coords_array.len());
    for item in coords_array {
        coords.push(json_to_position(item)?);
    }
    Ok(coords)
}

fn json_to_arc_indexes(json: &JsonValue) -> Result<ArcIndexes, Error> {
    let arc_array = expect_array(json)?;
    let mut arc_ixs = Vec::with_capacity(arc_array.len());
    for item in arc_array {
        arc_ixs.push(expect_i32(item)?);
    }
    Ok(arc_ixs)
}

fn json_to_1d_arc_indexes(json: &JsonValue) -> Result<Vec<ArcIndexes>, Error> {
    let arc_array = expect_array(json)?;
    let mut arc_ixs = Vec::with_capacity(arc_array.len());
    for item in arc_array {
        arc_ixs.push(json_to_arc_indexes(item)?);
    }
    Ok(arc_ixs)
}

fn json_to_2d_arc_indexes(json: &JsonValue) -> Result<Vec<Vec<ArcIndexes>>, Error> {
    let arc_array = expect_array(json)?;
    let mut arc_ixs = Vec::with_capacity(arc_array.len());
    for item in arc_array {
        arc_ixs.push(json_to_1d_arc_indexes(item)?);
    }
    Ok(arc_ixs)
}

fn json_to_arc_positions(json: &JsonValue) -> Result<Vec<Arc>, Error> {
    let coords_array = expect_array(json)?;
    let mut arcs = Vec::with_capacity(coords_array.len());
    for item in coords_array {
        arcs.push(json_to_1d_positions(item)?);
    }
    Ok(arcs)
}

fn expect_owned_array(value: JsonValue) -> Result<Vec<JsonValue>, Error> {
    match value {
        JsonValue::Array(v) => Ok(v),
        _ => Err(Error::ExpectedArrayValue),
    }
}

fn expect_owned_object(value: JsonValue) -> Result<JsonObject, Error> {
    match value {
        JsonValue::Object(o) => Ok(o),
        _ => Err(Error::ExpectedObjectValue),
    }
}
