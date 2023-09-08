# svg-metadata

[![CI](https://github.com/mre/svg-metadata/actions/workflows/rust.yml/badge.svg)](https://github.com/mre/svg-metadata/actions/workflows/rust.yml)
[![Documentation](https://docs.rs/svg_metadata/badge.svg)](https://docs.rs/svg_metadata/)

## What is it?

This crate extracts metadata from SVG files.
Currently it reads the following attributes:

- `viewBox`
- `width`
- `height`

You can add more!

## Usage Example

```rust
use svg_metadata::{Metadata, ViewBox};

fn main() {
    let svg = r#"
        <svg viewBox="0 1 99 100" xmlns="http://www.w3.org/2000/svg">
            <rect x="0" y="0" width="100%" height="100%"/>
        </svg>
    "#;

    let meta = Metadata::parse(svg).unwrap();
    assert_eq!(
        meta.view_box,
        Some(ViewBox {
            min_x: 0.0,
            min_y: 1.0,
            width: 99.0,
            height: 100.0
        })
    );
}
```

(You can also parse files directly with `parse_file()`.)

## Credits

The SVG fixtures used for testing are provided by

- [Openclipart](https://en.wikipedia.org/wiki/Openclipart)
- [SVG Specification Examples](https://www.w3.org/TR/SVG2/)
- [W3C SVG Working Group](https://www.w3.org/Graphics/SVG/)

under their respective licenses.
