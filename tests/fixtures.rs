//! This crate contains smoke tests for the `svg_metadata` crate.
//!
//! We parse a set of (W3C) SVG fixtures and check that the metadata can be
//! parsed.

use std::fs;
use svg_metadata::Metadata;

#[test]
fn test_fixtures() {
    let paths = fs::read_dir("./fixtures").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        println!("Parsing {}", path.display());

        let meta = Metadata::parse_file(path).unwrap();
        println!("Metadata: {meta:?}");
    }
}
