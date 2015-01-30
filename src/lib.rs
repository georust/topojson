extern crate "rustc-serialize" as rustc_serialize;
extern crate geojson;

pub use topology::Topology;
pub use error::{TopoJsonError, TopoJsonResult};

macro_rules! expect_string {
    ($value:expr) => (try!(
        match $value.as_string() {
            Some(v) => Ok(v),
            None => Err({use TopoJsonError; TopoJsonError::new("Expected string value")})
        }
    ))
}

macro_rules! expect_i64 {
    ($value:expr) => (try!(
        match $value.as_i64() {
            Some(v) => Ok(v),
            None => Err({use TopoJsonError; TopoJsonError::new("Expected i64 value")})
        }
    ))
}

macro_rules! expect_f64 {
    ($value:expr) => (try!(
        match $value.as_f64() {
            Some(v) => Ok(v),
            None => Err({use TopoJsonError; TopoJsonError::new("Expected f64 value")})
        }
    ))
}

macro_rules! expect_array {
    ($value:expr) => (try!(
        match $value.as_array() {
            Some(v) => Ok(v),
            None => Err({use TopoJsonError; TopoJsonError::new("Expected array value")})
        }
    ))
}

macro_rules! expect_object {
    ($value:expr) => (try!(
        match $value.as_object() {
            Some(v) => Ok(v),
            None => Err({use TopoJsonError; TopoJsonError::new("Expected object value")})
        }
    ))
}

macro_rules! expect_property {
    ($obj:expr, $name:expr, $desc:expr) => (
        match $obj.get($name) {
            Some(v) => v,
            None => return Err({use TopoJsonError; TopoJsonError::new($desc)}),
        };
    )
}

mod topology;
mod error;
