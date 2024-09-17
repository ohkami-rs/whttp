use std::ptr::NonNull;
use super::hash::{normalized_hash, const_normalized_hash};

/// # HTTP Header
/// 
/// ## usage
/// 
/// For standard headers, always use module consts (like `header::ContentType`).
/// 
/// For other custom headers, define them as 
/// 
/// Use module consts for standard headers, and you can define
/// a custom header by `Header::new`.
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
/// const MY_CUSTOM: Header = Header::custom("My-Custom");
/// 
/// todo!()
/// ```
#[derive(Clone, Copy)]
pub struct Header {
    name: NonNull<str>,
    hash: usize,
}

impl std::hash::Hash for Header {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash);
    }
}

// #[inline]
// pub(crate) const fn available(byte: u8) -> bool {
//     match byte {
//         | b'!' | b'"' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*'
//         | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|'  | b'~'
//         | b'0'..=b'9'
//         | b'A'..=b'Z' | b'a'..=b'z'
//         => true,
//         _ => false
//     }
// }

#[inline]
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
        let hash = match const_normalized_hash(name.as_bytes()) {
            Ok(hash) => hash,
            Err(_) => panic!("invalid header name")
        };

        // SAFETY: 'static reference is always valid
        let name = unsafe {NonNull::new_unchecked(name as *const str as *mut str)};

        Header { name, hash }
    }
}

impl Header {
    /// Parse `name` to `Header` with better performance when
    /// `name` is usually a name of standard header.
    /// 
    /// SAFETY: `name` is valid reference whenever the return value can be accessed
    #[inline(always)]
    pub(crate) unsafe fn parse_mainly_standard(name: &[u8]) -> Result<Self, InvalidHeader> {
        match Standard::from_bytes(name) {
            Some(s) => Ok(Header {
                // SAFETY: function SAFETY
                name: NonNull::new_unchecked(s.as_str() as *const str as *mut str),
                hash: s.hash()
            }),
            None => Self::parse(name)
        }
    }

    /// Parse `name` to `Header`.
    /// 
    /// SAFETY: `name` is valid reference whenever the return value can be accessed
    #[inline(always)]
    pub(crate) unsafe fn parse(name: &[u8]) -> Result<Self, InvalidHeader> {
        let hash = normalized_hash(name)?;

        // SAFETY: `normalized_hash` succeed
        let name = unsafe {std::str::from_utf8_unchecked(name)};
        // SAFETY: function SAFETY
        let name = unsafe {NonNull::new_unchecked(name as *const str as *mut str)};

        Ok(Header { name, hash })
    }
}

macro_rules! Standard {
    ($( $name:ident = ($bytes:literal, $lower:literal) => $hash:literal )*) => {
        pub mod standard {
            use super::*;
            $(
                #[allow(non_upper_case_globals)]
                pub const $name: Header = Header::new(unsafe {std::str::from_utf8_unchecked($bytes)});
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
            const fn as_str(&self) -> &'static str {
                match self {
                    $( Self::$name => unsafe {std::str::from_utf8_unchecked($bytes)}, )*
                }
            }

            #[inline(always)]
            const fn hash(&self) -> usize {
                match self {
                    $( Self::$name => $hash, )*
                }
            }
        }

        #[cfg(test)]
        mod test {
            #[test]
            fn test_standard_hash() {
                $(
                    assert_eq!($hash, crate::header::hash::const_normalized_hash($bytes).unwrap());
                    assert_eq!($hash, crate::header::hash::const_normalized_hash($lower).unwrap());
                )*
            }

            #[test]
            fn test_name_cases() {
                $(
                    assert_eq!($bytes[..].to_ascii_lowercase(), $lower);
                )*
            }
        }
    };
}
Standard! {
    Accept = (b"Accept", b"accept") => 8956897560123365965//16433268118574137039
    AcceptCH = (b"Accept-CH", b"accept-ch") => 6097036830808918762
    AcceptEncoding = (b"Accept-Encoding", b"accept-encoding") => 9008826061000250594//2625511035195335676
    AcceptLanguage = (b"Accept-Language", b"accept-language") => 263798198000172577//4857106753043711123
    AcceptPatch = (b"Accept-Patch", b"accept-patch") => 13563119043277128865
    AcceptPost = (b"Accept-Post", b"accept-post") => 18313133509215035340
    AcceptRanges = (b"Accept-Ranges", b"accept-ranges") => 18342038782328862764//12598308797832930634
    AccessControlAllowCredentials = (b"Access-Control-Allow-Credentials", b"access-control-allow-credentials") => 9569548883698504606//9116155820374356126
    AccessControlAllowHeaders = (b"Access-Control-Allow-Headers", b"access-control-allow-headers") => 14084078790202211956//8814696385715034476
    AccessControlAllowMethods = (b"Access-Control-Allow-Methods", b"access-control-allow-methods") => 5070491789143864837//5462557967219305584
    AccessControlAllowOrigin = (b"Access-Control-Allow-Origin", b"access-control-allow-origin") => 10178641993106032301//5378217592900298305
    AccessControlExposeHeaders = (b"Access-Control-Expose-Headers", b"access-control-expose-headers") => 16649258875025620622//13325522807785516598
    AccessControlMaxAge = (b"Access-Control-Max-Age", b"access-control-max-age") => 6947267048838179798//4432901313932580618
    AccessControlRequestHeaders = (b"Access-Control-Request-Headers", b"access-control-request-headers") => 14077911138246316357//16301979022674213810
    AccessControlRequestMethod = (b"Access-Control-Request-Method", b"access-control-request-method") => 1511226599830663409//11634788784195468787
    Age = (b"Age", b"age") => 12164395943281393619//10870321372244433485
    Allow = (b"Allow", b"allow") => 101864317638997375//3848169699148495437
    AltSvc = (b"Alt-Svc", b"alt-svc") => 163385565780702487//5918467845764560387
    AltUsed = (b"Alt-Used", b"alt-used") => 645746796108242549
    Authorization = (b"Authorization", b"authorization") => 17342828658748765377//12196702939659785452
    CacheControl = (b"Cache-Control", b"cache-control") => 12700798416643114791//11800019523689531337
    ClearSiteData = (b"Clear-Site-Data", b"clear-site-data") => 6682031554267653612
    Connection = (b"Connection", b"connection") => 12632990663184834470//16783757005565428516
    ContentDisposition = (b"Content-Disposition", b"content-disposition") => 1390085196203246353//15172909992608599841
    ContentEcoding = (b"Content-Ecoding", b"content-ecoding") => 1761535790260946701//16593443043870130009
    ContentLanguage = (b"Content-Language", b"content-language") => 1401788212079168976//16735614920345560642
    ContentLength = (b"Content-Length", b"content-length") => 14843332951706164276//14334207866575450264
    ContentLocation = (b"Content-Location", b"content-location") => 6809348355172982736//3944620592910008386
    ContentRange = (b"Content-Range", b"content-range") => 6591774876766068439//11588459248563791643
    ContentSecurityPolicy = (b"Content-Security-Policy", b"content-security-policy") => 7848988030993024328//5108162438765258431
    ContentSecurityPolicyReportOnly = (b"Content-Security-Policy-Report-Only", b"content-security-policy-report-only") => 8225512036531862485//1939240664108222842
    ContentType = (b"Content-Type", b"content-type") => 1539870117023715624//3996025485011955786
    Cookie = (b"Cookie", b"cookie") => 12510759127542743569//17962636191368536035
    CrossOriginEmbedderPolicy = (b"Cross-Origin-Embedder-Policy", b"cross-origin-embedder-policy") => 9830598595040102211
    CrossOriginOpenerPolicy = (b"Cross-Origin-Opener-Policy", b"cross-origin-opener-policy") => 8907391105727566330
    CrossOriginResourcePolicy = (b"Cross-Origin-Resource-Policy", b"cross-origin-resource-policy") => 46993471585439987
    Date = (b"Date", b"date") => 2562613028085471028//17579805628842460308
    DeviceMemory = (b"Device-Memory", b"device-memory") => 9442280248183064591
    ETag = (b"ETag", b"etag") => 14205462794407424201//18254449783657381417
    Expect = (b"Expect", b"expect") => 3319114356378929571//9494374193384502225
    Expires = (b"Expires", b"expires") => 14717995381802874822//4291902732285004317
    Forwarded = (b"Forwarded", b"forwarded") => 12510709791974329387//7787083747984806917
    From = (b"From", b"from") => 3435607823061342//15020628208580050622
    Host = (b"Host", b"host") => 3868342997265016712//438791524312454376
    IfMatch = (b"If-Match", b"if-match") => 758385572210193693//17728942688211657341
    IfModifiedSince = (b"If-Modified-Since", b"if-modified-since") => 15420386658409231737//6352457413450827350
    IfNoneMatch = (b"If-None-Match", b"if-none-match") => 8766751325359657529//3333932262875561685
    IfRange = (b"If-Range", b"if-range") => 4422112474835105053//2945925517127017085
    IfUnmodifiedSince = (b"If-Unmodified-Since", b"if-unmodified-since") => 14842325997600933810//7522477305903254470
    KeepAlive = (b"Keep-Alive", b"keep-alive") => 2783802276154524880
    LastModified = (b"Last-Modified", b"last-modified") => 15457224435839056573
    Link = (b"Link", b"link") => 6207054705583559644//2777503232630997308
    Location = (b"Location", b"location") => 1632295297794314716//16649487898551303996
    MaxForwards = (b"Max-Forwards", b"max-forwards") => 7426081339672782312//10752408927369271123
    Origin = (b"Origin", b"origin") => 5691687282345579944//14882833577272632186
    Priority = (b"Priority", b"priority") => 12606404892256893744
    ProxyAuthenticate = (b"Proxy-Authenticate", b"proxy-authenticate") => 14130340937869200619//1820963910701534218
    ProxyAuthorization = (b"Proxy-Authorization", b"proxy-authorization") => 4007097940433767016//12714354196972183062
    Range = (b"Range", b"range") => 13591622306488845170//10582771998975603868
    Referer = (b"Referer", b"referer") => 61951474100055896//5839330224843872351
    ReferrerPolicy = (b"Referrer-Policy", b"referrer-policy") => 1327666139445013389//18395389122136826733
    Refresh = (b"Refresh", b"refresh") => 7953246256297787639//15850643017965868815
    RetryAfter = (b"Retry-After", b"retry-after") => 11304873063226856260//13276509559803940695
    SecFetchDest = (b"Sec-Fetch-Dest", b"sec-fetch-dest") => 8071548918507657405
    SecFetchMode = (b"Sec-Fetch-Mode", b"sec-fetch-mode") => 13625918741039707069
    SecFetchSite = (b"Sec-Fetch-Site", b"sec-fetch-site") => 9883030191812069217
    SecFetchUser = (b"Sec-Fetch-User", b"sec-fetch-user") => 15232089924227654398
    SecWebSocketAccept = (b"Sec-WebSocket-Accept", b"sec-websocket-accept") => 5952345478380611784//10946272471545366737
    SecWebSocketExtensions = (b"Sec-WebSocket-Extensions", b"sec-websocket-extensions") => 12765399274657545454//17103059385744334201
    SecWebSocketKey = (b"Sec-WebSocket-Key", b"sec-websocket-key") => 11097846330773677699//13420602090516222027
    SecWebSocketProtocol = (b"Sec-WebSocket-Protocol", b"sec-websocket-protocol") => 16408706031545691252//11040576895242091634
    SecWebSocketVersion = (b"Sec-WebSocket-Version", b"sec-websocket-version") => 11714057070643420239//5330225619909672710
    Server = (b"Server", b"server") => 2419935139755271097//11765940313756672059
    SetCookie = (b"Set-Cookie", b"set-cookie") => 5506158778252165240//3623682265152868430
    StrictTransportSecurity = (b"Strict-Transport-Security", b"strict-transport-security") => 828070379554355266//13089560602798786294
    TE = (b"TE", b"te") => 2663045123408499844//6712032112658457060
    TimingAllowOrigin = (b"Timing-Allow-Origin", b"timing-allow-origin") => 6419927582714887957
    Trailer = (b"Trailer", b"trailer") => 7062438620934618372//15190164523930466561
    TransferEncoding = (b"Transfer-Encoding", b"transfer-encoding") => 7495137910697819204//8612619927895477042
    Upgrade = (b"Upgrade", b"upgrade") => 11782373995271654455//3830257985504030272
    UpgradeInsecureRequests = (b"Upgrade-Insecure-Requests", b"upgrade-insecure-requests") => 11536776535922301664//12060850129311366976
    UserAgent = (b"User-Agent", b"user-agent") => 9952940223324636988//3519543940131721058
    Vary = (b"Vary", b"vary") => 12247033862576493998//8817482389623931662
    Via = (b"Via", b"via") => 1872335714014322414//7229469575117716336
    WWWAuthenticate = (b"WWW-Authenticate", b"www-authenticate") => 8830284111271749131
    XContentTypeOptions = (b"X-Content-Type-Options", b"x-content-type-options") => 10317259392692853873//17298563304118097688
    XFrameOptions = (b"X-Frame-Options", b"x-frame-options") => 15858069221280842781//4381497337076230406
}
