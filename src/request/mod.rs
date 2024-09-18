mod method;
mod path;

pub use method::Method;
pub use path::Path;
use crate::Headers;

const BUF_SIZE: usize = 1024;

pub struct Request {
    __buf__: Box<[u8; BUF_SIZE]>,
    pub method:  Method,
    pub path:    Path,
    pub headers: Headers,
}

impl Request {
    pub fn uninit() -> Self {
        Self {
            __buf__: Box::new([0; BUF_SIZE]),
            method:  Method::GET,
            path:    
            headers: Headers::new()
        }
    }

    pub const fn __buf__(&self) -> &[u8; BUF_SIZE] {
        &self.__buf__
    }
    pub fn __buf_mut__(&mut self) -> &mut Box<[u8; BUF_SIZE]> {
        &mut self.__buf__
    }
}
