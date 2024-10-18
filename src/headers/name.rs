use std::ptr::NonNull;
use super::hash::normalized_hash;

/// # HTTP Header
/// 
/// ## usage
/// 
/// - For standard header, always use module const (like `header::ContentType`).
/// 
/// - For other custom header, define your own const by `Header::new`
/// and always use it.
/// 
/// ## selection policy of standard headers
/// 
/// - listed on https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers
/// - not marked as `Deprecated`
/// - not marked as `Experimental`
/// - not marked as `Non-Standard`
/// - not have `Warning`
/// 
/// ## example
/// ```
/// use whttp::Header;
/// 
/// const MY_CUSTOM: &Header = &Header::new("My-Custom");
/// 
/// todo!()
/// ```
#[derive(Clone, Copy)]
pub struct Header {
    name: NonNull<str>,
    pub(crate) hash: u64
}

const _/* trait impls */: () = {
    unsafe impl Send for Header {}
    unsafe impl Sync for Header {}
    
    impl std::hash::Hash for Header {
        #[inline]
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            state.write_u64(self.hash);
        }
    }
    
    impl PartialEq for Header {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.hash == other.hash
        }
    }
    impl Eq for Header {}

    impl std::fmt::Debug for Header {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&**self)
        }
    }

    impl std::ops::Deref for Header {
        type Target = str;

        #[inline]
        fn deref(&self) -> &Self::Target {
            // SAFETY: `Header` constructors' SAFETYs
            unsafe {self.name.as_ref()}
        }
    }

    /* not impl std::ops::DerefMut */
};

#[inline(always)]
pub(crate) const fn normalized(byte: u8) -> Result<u8, InvalidHeader> {
    match &byte {
        | b'!' | b'"' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*'
        | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|'  | b'~'
        | b'0'..=b'9'
        | b'a'..=b'z' => Ok(byte),
        b'A'..=b'Z' => Ok(byte + (b'a' - b'A')),
        _ => Err(InvalidHeader)
    }
}

#[derive(PartialEq)]
pub struct InvalidHeader;
impl std::error::Error for InvalidHeader {}
impl std::fmt::Debug for InvalidHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid header name")
    }
}
impl std::fmt::Display for InvalidHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid header name")
    }
}

impl Header {
    pub const fn new(name: &'static str) -> Self {
        let hash = match normalized_hash(name.as_bytes()) {
            Ok(hash) => hash,
            Err(_) => panic!("invalid header name")
        };

        // SAFETY: 'static reference is always valid
        let name = unsafe {NonNull::new_unchecked(name as *const str as *mut str)};

        Header { name, hash }
    }
}

impl Header {
    /// Parse header name to `Header` with better performance when
    /// `name` is usually a name of standard header.
    /// 
    /// SAFETY: `name` is valid reference whenever the return value can be accessed
    #[inline(always)]
    pub unsafe fn parse_mainly_standard(name: &[u8]) -> Result<Self, InvalidHeader> {
        match Standard::from_bytes(name) {
            Some(s) => Ok(*s.as_header()),
            None => Self::parse(name)
        }
    }

    /// Parse header name to `Header`.
    /// 
    /// SAFETY: `name` is valid reference whenever returned `Header` can be accessed
    #[inline(always)]
    pub unsafe fn parse(name: &[u8]) -> Result<Self, InvalidHeader> {
        let hash = normalized_hash(name)?;

        // SAFETY: `normalized_hash` succeed
        let name = unsafe {std::str::from_utf8_unchecked(name)};
        // SAFETY: function SAFETY
        let name = unsafe {NonNull::new_unchecked(name as *const str as *mut str)};

        Ok(Header { name, hash })
    }
}

macro_rules! Standard {
    ($( $name:ident = $bytes:literal | $lower:literal )*) => {
        pub mod standard {
            use super::*;
            $(
                #[allow(non_upper_case_globals)]
                pub const $name: &Header = &Header::new(unsafe {std::str::from_utf8_unchecked($bytes)});
            )*
        }

        enum Standard {
            $( $name, )*
        }
        impl Standard {
            #[inline(always)]
            const fn from_bytes(bytes: &[u8]) -> Option<Self> {
                match bytes {
                    $( $bytes | $lower => Some(Standard::$name) ,)*
                    _ => None
                }
            }

            #[inline(always)]
            const fn as_header(&self) -> &'static Header {
                match self {
                    $( Self::$name => standard::$name, )*
                }
            }
        }
    };
}
Standard! {
    Accept = b"Accept" | b"accept"
    AcceptCH = b"Accept-CH" | b"accept-ch"
    AcceptEncoding = b"Accept-Encoding" | b"accept-encoding"
    AcceptLanguage = b"Accept-Language" | b"accept-language"
    AcceptPatch = b"Accept-Patch" | b"accept-patch"
    AcceptPost = b"Accept-Post" | b"accept-post"
    AcceptRanges = b"Accept-Ranges" | b"accept-ranges"
    AccessControlAllowCredentials = b"Access-Control-Allow-Credentials" | b"access-control-allow-credentials"
    AccessControlAllowHeaders = b"Access-Control-Allow-Headers" | b"access-control-allow-headers"
    AccessControlAllowMethods = b"Access-Control-Allow-Methods" | b"access-control-allow-methods"
    AccessControlAllowOrigin = b"Access-Control-Allow-Origin" | b"access-control-allow-origin"
    AccessControlExposeHeaders = b"Access-Control-Expose-Headers" | b"access-control-expose-headers"
    AccessControlMaxAge = b"Access-Control-Max-Age" | b"access-control-max-age"
    AccessControlRequestHeaders = b"Access-Control-Request-Headers" | b"access-control-request-headers"
    AccessControlRequestMethod = b"Access-Control-Request-Method" | b"access-control-request-method"
    Age = b"Age" | b"age"
    Allow = b"Allow" | b"allow"
    AltSvc = b"Alt-Svc" | b"alt-svc"
    AltUsed = b"Alt-Used" | b"alt-used"
    Authorization = b"Authorization" | b"authorization"
    CacheControl = b"Cache-Control" | b"cache-control"
    ClearSiteData = b"Clear-Site-Data" | b"clear-site-data"
    Connection = b"Connection" | b"connection"
    ContentDisposition = b"Content-Disposition" | b"content-disposition"
    ContentEcoding = b"Content-Ecoding" | b"content-ecoding"
    ContentLanguage = b"Content-Language" | b"content-language"
    ContentLength = b"Content-Length" | b"content-length"
    ContentLocation = b"Content-Location" | b"content-location"
    ContentRange = b"Content-Range" | b"content-range"
    ContentSecurityPolicy = b"Content-Security-Policy" | b"content-security-policy"
    ContentSecurityPolicyReportOnly = b"Content-Security-Policy-Report-Only" | b"content-security-policy-report-only"
    ContentType = b"Content-Type" | b"content-type"
    Cookie = b"Cookie" | b"cookie"
    CrossOriginEmbedderPolicy = b"Cross-Origin-Embedder-Policy" | b"cross-origin-embedder-policy"
    CrossOriginOpenerPolicy = b"Cross-Origin-Opener-Policy" | b"cross-origin-opener-policy"
    CrossOriginResourcePolicy = b"Cross-Origin-Resource-Policy" | b"cross-origin-resource-policy"
    Date = b"Date" | b"date"
    DeviceMemory = b"Device-Memory" | b"device-memory"
    ETag = b"ETag" | b"etag"
    Expect = b"Expect" | b"expect"
    Expires = b"Expires" | b"expires"
    Forwarded = b"Forwarded" | b"forwarded"
    From = b"From" | b"from"
    Host = b"Host" | b"host"
    IfMatch = b"If-Match" | b"if-match"
    IfModifiedSince = b"If-Modified-Since" | b"if-modified-since"
    IfNoneMatch = b"If-None-Match" | b"if-none-match"
    IfRange = b"If-Range" | b"if-range"
    IfUnmodifiedSince = b"If-Unmodified-Since" | b"if-unmodified-since"
    KeepAlive = b"Keep-Alive" | b"keep-alive"
    LastModified = b"Last-Modified" | b"last-modified"
    Link = b"Link" | b"link"
    Location = b"Location" | b"location"
    MaxForwards = b"Max-Forwards" | b"max-forwards"
    Origin = b"Origin" | b"origin"
    Priority = b"Priority" | b"priority"
    ProxyAuthenticate = b"Proxy-Authenticate" | b"proxy-authenticate"
    ProxyAuthorization = b"Proxy-Authorization" | b"proxy-authorization"
    Range = b"Range" | b"range"
    Referer = b"Referer" | b"referer"
    ReferrerPolicy = b"Referrer-Policy" | b"referrer-policy"
    Refresh = b"Refresh" | b"refresh"
    RetryAfter = b"Retry-After" | b"retry-after"
    SecFetchDest = b"Sec-Fetch-Dest" | b"sec-fetch-dest"
    SecFetchMode = b"Sec-Fetch-Mode" | b"sec-fetch-mode"
    SecFetchSite = b"Sec-Fetch-Site" | b"sec-fetch-site"
    SecFetchUser = b"Sec-Fetch-User" | b"sec-fetch-user"
    SecWebSocketAccept = b"Sec-WebSocket-Accept" | b"sec-websocket-accept"
    SecWebSocketExtensions = b"Sec-WebSocket-Extensions" | b"sec-websocket-extensions"
    SecWebSocketKey = b"Sec-WebSocket-Key" | b"sec-websocket-key"
    SecWebSocketProtocol = b"Sec-WebSocket-Protocol" | b"sec-websocket-protocol"
    SecWebSocketVersion = b"Sec-WebSocket-Version" | b"sec-websocket-version"
    Server = b"Server" | b"server"
    SetCookie = b"Set-Cookie" | b"set-cookie"
    StrictTransportSecurity = b"Strict-Transport-Security" | b"strict-transport-security"
    TE = b"TE" | b"te"
    TimingAllowOrigin = b"Timing-Allow-Origin" | b"timing-allow-origin"
    Trailer = b"Trailer" | b"trailer"
    TransferEncoding = b"Transfer-Encoding" | b"transfer-encoding"
    Upgrade = b"Upgrade" | b"upgrade"
    UpgradeInsecureRequests = b"Upgrade-Insecure-Requests" | b"upgrade-insecure-requests"
    UserAgent = b"User-Agent" | b"user-agent"
    Vary = b"Vary" | b"vary"
    Via = b"Via" | b"via"
    WWWAuthenticate = b"WWW-Authenticate" | b"www-authenticate"
    XContentTypeOptions = b"X-Content-Type-Options" | b"x-content-type-options"
    XFrameOptions = b"X-Frame-Options" | b"x-frame-options"
}
