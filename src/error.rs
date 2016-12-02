use core::error::FromError;
use geojson::GeoJsonError;

#[derive(Debug, PartialEq, Eq, Copy)]
pub struct TopoJsonError {
    pub desc: &'static str,
}

impl TopoJsonError {
    pub fn new(desc: &'static str) -> TopoJsonError {
        return TopoJsonError{desc: desc};
    }
}

impl FromError<GeoJsonError> for TopoJsonError {
    fn from_error(e: GeoJsonError) -> TopoJsonError {
        return TopoJsonError::new(e.desc);
    }
}

pub type TopoJsonResult<T> = Result<T, TopoJsonError>;
