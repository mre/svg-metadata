//! `svg_metadata` is a Rust crate for parsing metadata information of SVG files.  
//! In can be useful for getting information from SVG graphics without using
//! a full-blown parser.  
//!
//! As such, it has a very narrow scope and only provides access to the fields
//! defined below.

#[macro_use]
extern crate lazy_static;

use regex::Regex;
use roxmltree::Document;
use std::convert::{AsRef, TryFrom};
use std::fs;
use std::path::PathBuf;

mod error;
use crate::error::MetadataError;

lazy_static! {
    // Initialize the regex to split a list of elements in the viewBox
    static ref VBOX_ELEMENTS: Regex = Regex::new(r",?\s+").unwrap();

    // Extract dimension information (e.g. 100em)
    static ref DIMENSION: Regex = Regex::new(r"([\+|-]?\d+\.?\d*)(\D{2})?").unwrap();
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Specifies the dimensions of an SVG image.
pub struct ViewBox {
    pub min_x: f64,
    pub min_y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Supported units for dimensions
pub enum Unit {
    /// The default font size - usually the height of a character.
    Em,
    /// The height of the character x
    Ex,
    /// Pixels
    Px,
    /// Points (1 / 72 of an inch)
    Pt,
    ///	Picas (1 / 6 of an inch)
    Pc,
    /// Centimeters
    Cm,
    /// Millimeters
    Mm,
    /// Inches
    In,
}

impl TryFrom<&str> for Unit {
    type Error = MetadataError;
    fn try_from(s: &str) -> Result<Unit, MetadataError> {
        let unit = match s.to_lowercase().as_ref() {
            "em" => Unit::Em,
            "ex" => Unit::Ex,
            "px" => Unit::Px,
            "pt" => Unit::Pt,
            "pc" => Unit::Pc,
            "cm" => Unit::Cm,
            "mm" => Unit::Mm,
            "in" => Unit::In,
            _ => return Err(MetadataError::new(&format!("Unknown unit: {}", s))),
        };
        Ok(unit)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Specifies the width of an SVG image.
pub struct Width {
    pub width: f64,
    pub unit: Unit,
}

fn parse_dimension(s: &str) -> Result<(f64, Unit), MetadataError> {
    let caps = DIMENSION
        .captures(s)
        .ok_or(MetadataError::new("Cannot read dimensions"))?;

    let width: &str = caps
        .get(1)
        .ok_or(MetadataError::new("No width specified"))?
        .as_str();
    let unit = caps.get(2).map_or("em", |m| m.as_str());

    Ok((width.parse::<f64>()?, Unit::try_from(unit)?))
}

impl TryFrom<&str> for Width {
    type Error = MetadataError;
    fn try_from(s: &str) -> Result<Width, MetadataError> {
        let (width, unit) = parse_dimension(s)?;
        Ok(Width { width, unit })
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Specifies the height of an SVG image.
pub struct Height {
    pub height: f64,
    pub unit: Unit,
}

impl TryFrom<&str> for Height {
    type Error = MetadataError;
    fn try_from(s: &str) -> Result<Height, MetadataError> {
        let (height, unit) = parse_dimension(s)?;
        Ok(Height { height, unit })
    }
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
        let min_x = elem[0].parse::<f64>()?;
        let min_y = elem[1].parse::<f64>()?;
        let width = elem[2].parse::<f64>()?;
        let height = elem[3].parse::<f64>()?;

        Ok(ViewBox {
            min_x,
            min_y,
            width,
            height,
        })
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Contains all metadata that was
/// extracted from an SVG image.
pub struct Metadata {
    pub view_box: Option<ViewBox>,
    pub width: Option<Width>,
    pub height: Option<Height>,
}

impl Metadata {
    /// Parse an SVG file and extract metadata from it.
    pub fn parse_file<T: Into<PathBuf>>(path: T) -> Result<Metadata, MetadataError> {
        let input = fs::read_to_string(path.into())?;
        Self::parse(input)
    }

    /// Parse SVG data and extract metadata from it.
    pub fn parse<T: AsRef<str>>(input: T) -> Result<Metadata, MetadataError> {
        let doc = Document::parse(input.as_ref())?;
        let svg_elem = doc.root_element();
        let view_box = match svg_elem.attribute("viewBox") {
            Some(val) => ViewBox::try_from(val).ok(),
            None => None,
        };

        let width = match svg_elem.attribute("width") {
            Some(val) => Width::try_from(val).ok(),
            None => None,
        };

        let height = match svg_elem.attribute("height") {
            Some(val) => Height::try_from(val).ok(),
            None => None,
        };

        Ok(Metadata {
            view_box,
            width,
            height,
        })
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
    fn test_width() {
        let tests = vec![
            (
                "100em",
                Width {
                    width: 100.0,
                    unit: Unit::Em,
                },
            ),
            (
                "100",
                Width {
                    width: 100.0,
                    unit: Unit::Em,
                },
            ),
            (
                "-10.0px",
                Width {
                    width: -10.0,
                    unit: Unit::Px,
                },
            ),
            (
                "100em",
                Width {
                    width: 100.0,
                    unit: Unit::Em,
                },
            ),
        ];
        for (input, expected) in tests {
            assert_eq!(Width::try_from(input).unwrap(), expected);
        }
    }

    #[test]
    fn test_height() {
        let tests = vec![
            (
                "100em",
                Height {
                    height: 100.0,
                    unit: Unit::Em,
                },
            ),
            (
                "100",
                Height {
                    height: 100.0,
                    unit: Unit::Em,
                },
            ),
            (
                "-10.0px",
                Height {
                    height: -10.0,
                    unit: Unit::Px,
                },
            ),
            (
                "100em",
                Height {
                    height: 100.0,
                    unit: Unit::Em,
                },
            ),
        ];
        for (input, expected) in tests {
            assert_eq!(Height::try_from(input).unwrap(), expected);
        }
    }

    #[test]
    fn test_metadata() {
        // separated by whitespace and/or a comma
        let svg = r#"<svg viewBox="0 1 99 100" width="2em" height="10cm" xmlns="http://www.w3.org/2000/svg">
  <rect x="0" y="0" width="100%" height="100%"/>
</svg>"#;

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
        assert_eq!(
            meta.width,
            Some(Width {
                width: 2.0,
                unit: Unit::Em
            })
        );
        assert_eq!(
            meta.height,
            Some(Height {
                height: 10.0,
                unit: Unit::Cm
            })
        )
    }
}
