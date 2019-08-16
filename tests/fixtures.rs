// extern crate we're testing, same as any other code would do.
extern crate svg_metadata;

use std::fs;
use svg_metadata::Metadata;

#[test]
fn test_fixtures() {
    let paths = fs::read_dir("./fixtures").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        println!("Parsing {}", path.display());

        let meta = Metadata::parse_file(path).unwrap();
        println!("Metadata: {:?}", meta);
    }
}
