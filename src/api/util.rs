//! Shared low-level helpers for the API layer.

/// Percent-encode a single URL path segment per RFC 3986, escaping everything
/// outside the unreserved set (`A-Z a-z 0-9 - _ . ~`). Used to safely embed
/// branch names, tags, artifact names, brace-wrapped UUIDs, and account IDs
/// into request paths.
pub(crate) fn urlencode_segment(s: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => {
                out.push('%');
                out.push(HEX[(b >> 4) as usize] as char);
                out.push(HEX[(b & 0xf) as usize] as char);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::urlencode_segment;

    #[test]
    fn leaves_unreserved_untouched() {
        assert_eq!(
            urlencode_segment("Feature.branch-1_v~2"),
            "Feature.branch-1_v~2"
        );
    }

    #[test]
    fn encodes_slashes_and_braces() {
        assert_eq!(urlencode_segment("feature/login"), "feature%2Flogin");
        assert_eq!(urlencode_segment("{uuid}"), "%7Buuid%7D");
    }

    #[test]
    fn encodes_space_and_colon_and_utf8() {
        assert_eq!(urlencode_segment("a b:c"), "a%20b%3Ac");
        assert_eq!(urlencode_segment("é"), "%C3%A9");
    }
}
