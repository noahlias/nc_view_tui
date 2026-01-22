use std::path::PathBuf;

use cnc_gcode::{parse_file_with_options, ParseOptions};

#[test]
fn parse_demo_file() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/demo.nc");

    let options = ParseOptions::with_ignore_missing(['E']).with_ignore_unknown_words(true);
    let toolpath = parse_file_with_options(&path, options).unwrap();

    assert!(!toolpath.segments.is_empty());
    assert_eq!(toolpath.line_segment_ends.len(), 7);
}
