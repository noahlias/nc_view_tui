use std::path::PathBuf;

use cnc_gcode::{export_toolpath_obj, parse_file_with_options, ObjExportOptions, ParseOptions};

fn unique_temp_obj_path() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "cnc_gcode_export_test_{}_{}",
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("toolpath.obj")
}

#[test]
fn export_demo_file_to_obj() {
    let mut input = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    input.push("tests/fixtures/demo.nc");

    let parse_options = ParseOptions::with_ignore_missing(['E']).with_ignore_unknown_words(true);
    let toolpath = parse_file_with_options(&input, parse_options).unwrap();

    let obj_path = unique_temp_obj_path();
    export_toolpath_obj(&toolpath, &obj_path, &ObjExportOptions::default()).unwrap();

    let mtl_path = obj_path.with_extension("mtl");
    assert!(obj_path.exists());
    assert!(mtl_path.exists());

    let obj_text = std::fs::read_to_string(&obj_path).unwrap();
    let face_count = obj_text
        .lines()
        .filter(|line| line.starts_with("f "))
        .count();
    assert!(obj_text.contains("mtllib toolpath.mtl"));
    assert!(obj_text.contains("usemtl feed_path"));
    assert!(obj_text.lines().any(|line| line.starts_with("v ")));
    assert!(face_count > 0);

    let mtl_text = std::fs::read_to_string(&mtl_path).unwrap();
    assert!(mtl_text.contains("newmtl feed_path"));

    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(mtl_path);
}
