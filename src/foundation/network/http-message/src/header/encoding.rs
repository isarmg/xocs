use super::{HeaderMap, ACCEPT_ENCODING};

/// Whether a request permits either of the supported content representations.
///
/// This records acceptability rather than preference. A responder can prefer
/// gzip whenever it is allowed, and use identity only when that is allowed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcceptedEncodings {
    /// A representation carrying `Content-Encoding: gzip` is acceptable.
    pub gzip: bool,
    /// An unencoded representation is acceptable.
    pub identity: bool,
}

/// Determine whether gzip and identity representations are acceptable.
///
/// An absent or empty `Accept-Encoding` field permits identity only. Explicit
/// coding weights override `*`; identity is permitted unless explicitly
/// excluded, including by `*;q=0` when identity has no explicit weight.
/// Repeated codings across all field lines use their lowest weight. Coding and
/// parameter names are case insensitive, and quality values use the HTTP
/// grammar (`0` or `1` with at most three fractional digits).
///
/// Unsupported codings are ignored. Invalid field text or malformed parameters
/// on gzip, identity or `*` reject both supported representations, so ambiguous
/// exclusions cannot accidentally permit a response the request forbids.
pub fn accepted_encodings(headers: &HeaderMap) -> AcceptedEncodings {
    let reject = AcceptedEncodings {
        gzip: false,
        identity: false,
    };
    let mut gzip = None;
    let mut identity = None;
    let mut wildcard = None;
    for field in headers.get_all(ACCEPT_ENCODING) {
        let Ok(value) = field.to_str() else {
            return reject;
        };
        for entry in value.split(',') {
            let mut parts = entry.split(';');
            let coding = parts
                .next()
                .expect("invariant: str::split always yields its first segment")
                .trim();
            let target = if coding.eq_ignore_ascii_case("gzip") {
                &mut gzip
            } else if coding.eq_ignore_ascii_case("identity") {
                &mut identity
            } else if coding == "*" {
                &mut wildcard
            } else {
                continue;
            };
            let weight = if let Some(parameter) = parts.next() {
                let Some((name, value)) = parameter.trim().split_once('=') else {
                    return reject;
                };
                if !name.trim().eq_ignore_ascii_case("q") || parts.next().is_some() {
                    return reject;
                }
                let Some(weight) = quality(value.trim()) else {
                    return reject;
                };
                weight
            } else {
                1000
            };
            *target = Some(target.map_or(weight, |previous: u16| previous.min(weight)));
        }
    }
    AcceptedEncodings {
        gzip: gzip.or(wildcard).is_some_and(|weight| weight > 0),
        identity: identity.unwrap_or(if wildcard == Some(0) { 0 } else { 1000 }) > 0,
    }
}

fn quality(value: &str) -> Option<u16> {
    let Some((whole, fraction)) = value.split_once('.') else {
        return match value {
            "0" => Some(0),
            "1" => Some(1000),
            _ => None,
        };
    };
    if fraction.len() > 3 || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    match whole {
        "1" => fraction.bytes().all(|byte| byte == b'0').then_some(1000),
        "0" => {
            let mut result = 0;
            let mut place = 100;
            for byte in fraction.bytes() {
                result += u16::from(byte - b'0') * place;
                place /= 10;
            }
            Some(result)
        }
        _ => None,
    }
}
