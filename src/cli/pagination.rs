//! Shared pagination and formatting helpers for CLI list commands.

use colored::Colorize;

/// The maximum page size the Bitbucket API accepts.
pub(crate) const MAX_LIMIT: u32 = 100;

/// Cap a requested limit at [`MAX_LIMIT`], returning the effective value and
/// whether it was reduced.
pub(crate) fn cap_limit(limit: u32) -> (u32, bool) {
    if limit > MAX_LIMIT {
        (MAX_LIMIT, true)
    } else {
        (limit, false)
    }
}

/// Cap `--limit` at [`MAX_LIMIT`], printing a notice (in table mode only) when
/// the requested value is reduced. This is the single limit policy for every
/// paginated command.
pub(crate) fn effective_limit(limit: u32) -> u32 {
    let (capped, was_capped) = cap_limit(limit);
    if was_capped && !super::output_json() {
        println!(
            "{} --limit capped at {} (the Bitbucket API maximum)",
            "ℹ".blue(),
            MAX_LIMIT
        );
    }
    capped
}

/// Format a byte count into a short human-readable string (decimal KB/MB/…).
pub(crate) fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.1} {}", size, UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_LIMIT, cap_limit, format_size};

    #[test]
    fn cap_limit_passes_through_at_or_below_max() {
        assert_eq!(cap_limit(25), (25, false));
        assert_eq!(cap_limit(MAX_LIMIT), (MAX_LIMIT, false));
    }

    #[test]
    fn cap_limit_reduces_above_max() {
        assert_eq!(cap_limit(101), (MAX_LIMIT, true));
        assert_eq!(cap_limit(1000), (MAX_LIMIT, true));
    }

    #[test]
    fn format_size_uses_bytes_below_1k() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn format_size_scales_to_larger_units() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(5 * 1024 * 1024 * 1024), "5.0 GB");
    }
}
