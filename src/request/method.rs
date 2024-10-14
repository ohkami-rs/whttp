macro_rules! Method {
    ($($name:ident = $bytes:literal)*) => {
        /// # HTTP method
        /// 
        /// ## Note
        /// - `whttp` doesn't support `TRACE` method.
        /// - Currently, custom method is not implemented.
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub enum Method { $($name),* }

        impl Method {
            #[inline]
            pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
                match bytes {
                    $( $bytes => Some(Self::$name), )*
                    _ => None
                }
            }
        }

        impl super::Request {$(
            #[inline]
            #[allow(non_snake_case)]
            pub fn $name(path: impl super::IntoStr) -> Self {
                Self::of(Method::$name, path)
            }
        )*}
    };
}
Method! {
    GET     = b"GET"
    PUT     = b"PUT"
    POST    = b"POST"
    PATCH   = b"PATCH"
    DELETE  = b"DELETE"
    HEAD    = b"HEAD"
    OPTIONS = b"OPTIONS"
    CONNECT = b"CONNECT"
}
