# svg-metadata

## What is it?

This crate extracts metadata from SVG files.
Currently, it reads the following attributes:

* `viewBox`
* `width`
* `height`

You can add more!

## Example Usage

```rust
use svg_metadata::{Metadata, ViewBox};

fn main() {
    let svg = r#"<svg viewBox="0 1 99 100" xmlns="http://www.w3.org/2000/svg">
    <rect x="0" y="0" width="100%" height="100%"/>
    </svg>"#;

    let meta = Metadata::parse(svg.to_string()).unwrap();
    assert_eq!(
        meta.view_box(),
        Some(ViewBox {
            min_x: 0.0,
            min_y: 1.0,
            width: 99.0,
            height: 100.0
        })
    );
}
```