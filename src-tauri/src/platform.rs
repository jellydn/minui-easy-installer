//! Shared device platform mapping for the MinUI installer.
//!
//! MinUI base archives contain shared folders (`Bios`, `Roms`, `Saves`,
//! `MinUI.zip`) plus a device-specific folder or file. This module keeps the
//! mapping from installer platform names to those archive items in a single
//! place so the install pipeline and post-install validation stay in sync.

/// Canonical mapping from installer platform name to the base-archive item
/// (folder or file) that must be copied to the SD card root.
///
/// Most platforms use a folder named exactly after the platform. The M17 is
/// the exception: it uses a single root script (`em_ui.sh`).
pub(crate) const DEVICE_BASE_MAPPINGS: &[(&str, &str)] = &[
    ("m17", "em_ui.sh"),
    ("miyoo", "miyoo"),
    ("miyoo354", "miyoo354"),
    ("miyoo355", "miyoo355"),
    ("miyoo285", "miyoo285"),
    ("trimui", "trimui"),
    ("rg35xx", "rg35xx"),
    ("rg35xxplus", "rg35xxplus"),
    ("gkdpixel", "gkdpixel"),
    ("magicx", "magicx"),
];

/// Returns the device-specific base archive item (folder or file) for a platform.
pub fn device_base_item(platform: &str) -> &str {
    for (p, item) in DEVICE_BASE_MAPPINGS {
        if *p == platform {
            return item;
        }
    }
    platform
}

/// Known device-specific folders/files that may appear at the SD card root.
///
/// These match the top-level items inside a MinUI base archive that are not
/// shared across devices. Keeping this list centralised lets validation warn
/// when more than one is present on an SD card.
pub const KNOWN_DEVICE_BASE_ITEMS: &[&str] = &[
    "miyoo",
    "miyoo354",
    "miyoo355",
    "miyoo285",
    "trimui",
    "rg35xx",
    "rg35xxplus",
    "gkdpixel",
    "magicx",
    "em_ui.sh",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_items_cover_all_mapped_outputs() {
        let known: std::collections::HashSet<_> =
            KNOWN_DEVICE_BASE_ITEMS.iter().copied().collect();
        for (_, item) in DEVICE_BASE_MAPPINGS {
            assert!(
                known.contains(item),
                "missing {item} in KNOWN_DEVICE_BASE_ITEMS"
            );
        }
    }

    #[test]
    fn every_known_item_is_reachable() {
        let reachable: std::collections::HashSet<_> = DEVICE_BASE_MAPPINGS
            .iter()
            .map(|(platform, _)| device_base_item(platform))
            .collect();
        for item in KNOWN_DEVICE_BASE_ITEMS {
            assert!(
                reachable.contains(item),
                "{item} in KNOWN_DEVICE_BASE_ITEMS is not produced by any device_base_item mapping"
            );
        }
    }
}
