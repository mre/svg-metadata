#[macro_use]
extern crate lazy_static;

use roxmltree::Document;
use roxmltree::Error as XMLError;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::num::ParseFloatError;
use regex::Regex;

lazy_static! {
    // Initialize the regex to split a list of elements in the viewBox
    static ref VBOX_ELEMENTS: Regex = Regex::new(r",?\s+").unwrap();
}

#[derive(Debug)]
pub struct MetadataError {
    details: String,
}

impl MetadataError {
    fn new(msg: &str) -> MetadataError {
        MetadataError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for MetadataError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<ParseFloatError> for MetadataError {
    fn from(_: ParseFloatError) -> MetadataError {
        MetadataError::new("Cannot convert string to float")
    }
}

impl From<IoError> for MetadataError {
    fn from(e: IoError) -> MetadataError {
        MetadataError::new(e.description())
    }
}

impl From<XMLError> for MetadataError {
    fn from(e: XMLError) -> MetadataError {
        MetadataError::new(&e.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct ViewBox {
    min_x: f32,
    min_y: f32,
    width: f32,
    height: f32,
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
pub struct Metadata {
    view_box: Option<ViewBox>,
}

impl Metadata {
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
