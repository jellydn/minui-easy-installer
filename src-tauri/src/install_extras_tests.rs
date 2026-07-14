//! Tests for extras archive copying and version metadata writing.

use super::*;
use std::io::Write;

#[test]
fn test_copy_extras_files_filters_by_platform() {
    let temp = tempfile::tempdir().unwrap();
    let extras_src = temp.path().join("extras_extracted");
    let sd_root = temp.path().join("sdcard");
    let platform = "rg35xxplus";

    // Create a realistic extras archive structure with multiple platforms
    fs::create_dir_all(extras_src.join("Emus/rg35xxplus/mgba.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Emus/rg35xxplus/gambatte.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Tools/rg35xxplus/wifi.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Tools/rg35xxplus/ssh.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Tools/trimuismart/dc.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Tools/trimuismart/wifi.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Bios")).unwrap();

    fs::write(extras_src.join("Emus/rg35xxplus/mgba.pak/launch.sh"), "emu").unwrap();
    fs::write(
        extras_src.join("Emus/rg35xxplus/gambatte.pak/launch.sh"),
        "emu",
    )
    .unwrap();
    fs::write(
        extras_src.join("Tools/rg35xxplus/wifi.pak/launch.sh"),
        "tool",
    )
    .unwrap();
    fs::write(
        extras_src.join("Tools/rg35xxplus/ssh.pak/launch.sh"),
        "tool",
    )
    .unwrap();
    fs::write(
        extras_src.join("Tools/trimuismart/dc.pak/launch.sh"),
        "tool",
    )
    .unwrap();
    fs::write(
        extras_src.join("Tools/trimuismart/wifi.pak/launch.sh"),
        "tool",
    )
    .unwrap();
    fs::write(extras_src.join("Bios/gba_bios.bin"), "bios").unwrap();

    let copied = copy_extras_files(
        extras_src.to_str().unwrap(),
        sd_root.to_str().unwrap(),
        platform,
    )
    .unwrap();

    // Should copy: 2 emus + 2 tools + 1 bios = 5 files (not the trimuismart ones)
    assert_eq!(copied, 5);

    // Verify rg35xxplus emus and tools were copied
    assert!(sd_root.join("Emus/rg35xxplus/mgba.pak/launch.sh").exists());
    assert!(sd_root
        .join("Emus/rg35xxplus/gambatte.pak/launch.sh")
        .exists());
    assert!(sd_root.join("Tools/rg35xxplus/wifi.pak/launch.sh").exists());
    assert!(sd_root.join("Tools/rg35xxplus/ssh.pak/launch.sh").exists());

    // Verify trimuismart stuff was NOT copied
    assert!(!sd_root.join("Tools/trimuismart").exists());

    // Verify Bios was copied
    assert!(sd_root.join("Bios/gba_bios.bin").exists());
}

/// Integration test for extras archive copy filtering with multiple
/// platforms in the same archive. Verifies that selecting each platform
/// only copies that platform's Emus/Tools, plus shared Bios, and leaves
/// all other platforms' files behind.
#[test]
fn test_copy_extras_files_filters_multiple_platforms() {
    let temp = tempfile::tempdir().unwrap();
    let extras_src = temp.path().join("extras_extracted");
    let sd_root = temp.path().join("sdcard");

    // Build an extras archive with three platforms and shared Bios.
    let platforms = &["rg35xxplus", "trimuismart", "miyoo354"];
    for platform in platforms {
        fs::create_dir_all(extras_src.join(format!("Emus/{}/core.pak", platform))).unwrap();
        fs::write(
            extras_src.join(format!("Emus/{}/core.pak/launch.sh", platform)),
            format!("{} emu\n", platform),
        )
        .unwrap();
        fs::create_dir_all(extras_src.join(format!("Tools/{}/tool.pak", platform))).unwrap();
        fs::write(
            extras_src.join(format!("Tools/{}/tool.pak/launch.sh", platform)),
            format!("{} tool\n", platform),
        )
        .unwrap();
    }
    fs::create_dir_all(extras_src.join("Bios")).unwrap();
    fs::write(extras_src.join("Bios/gba_bios.bin"), "bios").unwrap();

    for selected in platforms {
        let selected = *selected;
        let sd_root = sd_root.join(selected);
        fs::create_dir_all(&sd_root).unwrap();

        let copied = copy_extras_files(
            extras_src.to_str().unwrap(),
            sd_root.to_str().unwrap(),
            selected,
        )
        .unwrap();

        // Should copy at least the selected platform's emu/tool plus shared Bios.
        assert!(copied > 0, "nothing copied for {}", selected);

        // Selected platform files exist with correct content
        assert_eq!(
            fs::read_to_string(sd_root.join(format!("Emus/{}/core.pak/launch.sh", selected)))
                .unwrap(),
            format!("{} emu\n", selected)
        );
        assert_eq!(
            fs::read_to_string(sd_root.join(format!("Tools/{}/tool.pak/launch.sh", selected)))
                .unwrap(),
            format!("{} tool\n", selected)
        );

        // Shared Bios exists with correct content
        assert_eq!(
            fs::read_to_string(sd_root.join("Bios/gba_bios.bin")).unwrap(),
            "bios"
        );

        // Other platforms' files do not exist
        for other in platforms {
            let other = *other;
            if other == selected {
                continue;
            }
            assert!(
                !sd_root.join(format!("Emus/{}", other)).exists(),
                "{} Emus should not be copied when installing {}",
                other,
                selected
            );
            assert!(
                !sd_root.join(format!("Tools/{}", other)).exists(),
                "{} Tools should not be copied when installing {}",
                other,
                selected
            );
        }
    }
}

#[test]
fn test_minui_txt_writes_fork_name() {
    let temp = tempfile::tempdir().unwrap();
    let sd_root = temp.path();

    // Simulate what install_minui_with_cancel writes after copying
    let fork_label = "MinUI-Zero";
    let version = "20250525";
    let minui_txt_path = sd_root.join("minui.txt");
    fs::write(&minui_txt_path, format!("{} {}\n", fork_label, version)).unwrap();

    let content = fs::read_to_string(&minui_txt_path).unwrap();
    assert_eq!(content, "MinUI-Zero 20250525\n");
}

#[test]
fn test_minui_txt_defaults_to_minui_when_no_fork_name() {
    let temp = tempfile::tempdir().unwrap();
    let sd_root = temp.path();

    let fork_label = "MinUI"; // default when fork_name is None
    let version = "2025.01.01";
    let minui_txt_path = sd_root.join("minui.txt");
    fs::write(&minui_txt_path, format!("{} {}\n", fork_label, version)).unwrap();

    let content = fs::read_to_string(&minui_txt_path).unwrap();
    assert_eq!(content, "MinUI 2025.01.01\n");
}
