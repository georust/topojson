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

/// Error when reading to TopoJson
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    BboxExpectedArray,
    BboxExpectedNumericValues,
    TopologyExpectedObjects,
    TopologyExpectedArcs,
    TransformExpectedScale,
    TransformExpectedTranslate,
    ScaleExpectedArray,
    ScaleExpectedNumericValues,
    TranslateExpectedArray,
    TranslateExpectedNumericValues,
    TopoJsonUnknownType,
    GeometryUnknownType,
    MalformedJson,
    PropertiesExpectedObjectOrNull,
    ExpectedType { expected: String, actual: String },
    TopoToGeoUnknownKey(String),

    // FIXME: make these types more specific
    ExpectedStringValue,
    ExpectedProperty(String),
    Expectedi32Value,
    ExpectedF64Value,
    ExpectedArrayValue,
    ExpectedObjectValue,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::BboxExpectedArray => {
                write!(f, "Encountered non-array type for a 'bbox' object.")
            }
            Error::BboxExpectedNumericValues => {
                write!(f, "Encountered non-numeric value within 'bbox' array.")
            }
            Error::TopologyExpectedObjects => {
                write!(f, "Expected member with the name 'objects' in Topology.")
            }
            Error::TopologyExpectedArcs => {
                write!(f, "Expected member with the name 'arcs' in Topology.")
            }
            Error::TransformExpectedScale => {
                write!(f, "Transform must have a member with the name 'scale'.")
            }
            Error::TransformExpectedTranslate => {
                write!(f, "Transform must have a member with the name 'translate'.")
            }
            Error::ScaleExpectedArray => {
                write!(f, "Encountered non-array type for a 'scale' object.")
            }
            Error::ScaleExpectedNumericValues => {
                write!(f, "Encountered non-numeric value within 'scale' array.")
            }
            Error::TranslateExpectedArray => {
                write!(f, "Encountered non-array type for a 'translate' object.")
            }
            Error::TranslateExpectedNumericValues => {
                write!(f, "Encountered non-numeric value within 'translate' array.")
            }
            Error::TopoJsonUnknownType => write!(f, "Encountered unknown TopoJSON object type."),
            Error::GeometryUnknownType => write!(f, "Encountered unknown 'geometry' object type."),
            Error::MalformedJson =>
            // FIXME: can we report specific serialization error?
            {
                write!(f, "Encountered malformed JSON.")
            }
            Error::PropertiesExpectedObjectOrNull =>
            // FIXME: inform what type we actually found
            {
                write!(
                    f,
                    "Encountered neither object type nor null type for \
                     'properties' object."
                )
            }
            Error::ExpectedType {
                ref expected,
                ref actual,
            } => write!(
                f,
                "Expected TopoJSON type '{}', found '{}'",
                expected, actual,
            ),
            Error::TopoToGeoUnknownKey(ref key) => {
                write!(f, "No object with key '{}' in the given Topology.", key)
            }
            Error::ExpectedStringValue => write!(f, "Expected a string value."),
            Error::ExpectedProperty(ref prop_name) => {
                write!(f, "Expected TopoJSON property '{}'.", prop_name)
            }
            Error::ExpectedF64Value => write!(f, "Expected a floating-point value."),
            Error::Expectedi32Value => write!(f, "Expected a positive integer."),
            Error::ExpectedArrayValue => write!(f, "Expected an array."),
            Error::ExpectedObjectValue => write!(f, "Expected an object."),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BboxExpectedArray => "non-array 'bbox' type",
            Error::BboxExpectedNumericValues => "non-numeric 'bbox' array",
            Error::TopologyExpectedObjects => "no 'objects' member in topology",
            Error::TopologyExpectedArcs => "no 'arcs' member in topology",
            Error::TransformExpectedScale => "no 'scale' member in 'transform' member of topology",
            Error::TransformExpectedTranslate => {
                "no 'translate' member in 'transform' member of topology"
            }
            Error::ScaleExpectedArray => "non-array 'scale' type",
            Error::ScaleExpectedNumericValues => "non-numeric 'scale' array",
            Error::TranslateExpectedArray => "non-array 'translate' type",
            Error::TranslateExpectedNumericValues => "non-numeric 'translate' array",
            Error::TopoJsonUnknownType => "unknown TopoJSON object type",
            Error::GeometryUnknownType => "unknown 'geometry' object type",
            Error::MalformedJson => "malformed JSON",
            Error::PropertiesExpectedObjectOrNull => {
                "neither object type nor null type for properties' object."
            }
            Error::ExpectedType { .. } => "mismatched TopoJSON type",
            Error::TopoToGeoUnknownKey(..) => "requested key not found",
            Error::ExpectedStringValue => "expected a string value",
            Error::ExpectedProperty(..) => "expected a TopoJSON property",
            Error::ExpectedF64Value => "expected a floating-point value",
            Error::Expectedi32Value => "expected a positive integer",
            Error::ExpectedArrayValue => "expected an array",
            Error::ExpectedObjectValue => "expected an object",
        }
    }
}
