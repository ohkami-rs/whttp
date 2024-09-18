/// # HTTP method
/// 
/// ## Note
/// - `whttp` doesn't support `TRACE` method.
/// - Currently, custom method is not implemented.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Method {
    GET,
    PUT,
    POST,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    CONNECT,
}

impl Method {
    #[inline]
    pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"GET"     => Some(Self::GET),
            b"PUT"     => Some(Self::PUT),
            b"POST"    => Some(Self::POST),
            b"PATCH"   => Some(Self::PATCH),
            b"DELETE"  => Some(Self::DELETE),
            b"HEAD"    => Some(Self::HEAD),
            b"OPTIONS" => Some(Self::OPTIONS),
            b"CONNECT" => Some(Self::CONNECT),
            _ => None
        }
    }
}
