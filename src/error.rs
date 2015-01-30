#[derive(Copy)]
pub struct TopoJsonError {
    pub desc: &'static str,
}

impl TopoJsonError {
    pub fn new(desc: &'static str) -> TopoJsonError {
        return TopoJsonError{desc: desc};
    }
}

pub type TopoJsonResult<T> = Result<T, TopoJsonError>;
