#[macro_use]
extern crate lazy_static;

use roxmltree::Document;
use std::convert::TryFrom;
use regex::Regex;

mod error;
use crate::error::MetadataError;

lazy_static! {
    // Initialize the regex to split a list of elements in the viewBox
    static ref VBOX_ELEMENTS: Regex = Regex::new(r",?\s+").unwrap();
}

#[derive(Debug, PartialEq)]
/// Specifies the dimensions of an SVG image.
pub struct ViewBox {
    pub min_x: f32,
    pub min_y: f32,
    pub width: f32,
    pub height: f32,
}

impl TryFrom<&str> for ViewBox {
    type Error = MetadataError;
    fn try_from(s: &str) -> Result<ViewBox, MetadataError> {
        let elem: Vec<&str> = VBOX_ELEMENTS.split(s).collect();

        if elem.len() != 4 {
            return Err(MetadataError::new(&format!(
                "Invalid view_box: Expected four elements, got {}",
                elem.len()
            )));
        }
        let min_x = elem[0].parse::<f32>()?;
        let min_y = elem[1].parse::<f32>()?;
        let width = elem[2].parse::<f32>()?;
        let height = elem[3].parse::<f32>()?;

        Ok(ViewBox {
            min_x,
            min_y,
            width,
            height,
        })
    }
}

#[derive(Debug, PartialEq)]
/// Contains all metadata that was
/// extracted from an SVG image.
pub struct Metadata {
    pub view_box: Option<ViewBox>,
}

impl Metadata {
    /// Parse an SVG file and extract metadata from it.
    pub fn parse(input: String) -> Result<Metadata, MetadataError> {
        let doc = Document::parse(&input)?;
        let svg_elem = doc.root_element();
        let view_box = match svg_elem.attribute("viewBox") {
            Some(val) => ViewBox::try_from(val).ok(),
            None => None,
        };

        Ok(Metadata { view_box })
    }

    pub fn view_box(self) -> Option<ViewBox> {
        self.view_box
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_box_separators() {
        // Values can be separated by whitespace and/or a comma
        let cases = vec!["0 1 99 100", "0, 1, 99, 100", "0, 1  99 100"];
        for case in cases {
            assert_eq!(
                ViewBox::try_from(case).unwrap(),
                ViewBox {
                    min_x: 0.0,
                    min_y: 1.0,
                    width: 99.0,
                    height: 100.0
                }
            )
        }
    }

    #[test]
    fn test_view_box_negative() {
        assert_eq!(
            ViewBox::try_from("-0, 1, -99.00001, -100.3").unwrap(),
            ViewBox {
                min_x: 0.0,
                min_y: 1.0,
                width: -99.00001,
                height: -100.3
            }
        )
    }

    #[test]
    fn test_metadata() {
        // separated by whitespace and/or a comma
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
}
