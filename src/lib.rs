//! `svg_metadata` is a Rust crate for parsing metadata information of SVG files.  
//! In can be useful for getting information from SVG graphics without using
//! a full-blown parser.  
//!
//! As such, it has a very narrow scope and only provides access to the fields
//! defined below.

#[cfg(doctest)]
doctest!("../README.md");

use std::convert::{AsRef, TryFrom};
use std::fs;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use regex::Regex;

mod error;
use crate::error::Metadata as MetadataError;

/// Regex to split a list of elements in the viewBox
static VBOX_ELEMENTS: Lazy<Regex> = Lazy::new(|| Regex::new(r",?\s+").unwrap());

/// Regex to extract dimension information (e.g. 100em)
static DIMENSION: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\+|-]?\d+\.?\d*)(\D\D?)?").unwrap());

/// Specifies the dimensions of an SVG image.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ViewBox {
    /// The x coordinate of the left edge of the viewBox
    pub min_x: f64,
    /// The y coordinate of the top edge of the viewBox
    pub min_y: f64,
    /// The width of the viewBox
    pub width: f64,
    /// The height of the viewBox
    pub height: f64,
}

/// Supported units for dimensions
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[non_exhaustive]
pub enum Unit {
    /// The default font size - usually the height of a character.
    Em,
    /// The height of the character x
    Ex,
    /// Pixels
    Px,
    /// Points (1 / 72 of an inch)
    Pt,
    /// Picas (1 / 6 of an inch)
    Pc,
    /// Centimeters
    Cm,
    /// Millimeters
    Mm,
    /// Inches
    In,
    /// Percent
    Percent,
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
            "%" => Unit::Percent,
            _ => return Err(MetadataError::new(&format!("Unknown unit: {s}"))),
        };
        Ok(unit)
    }
}

/// Specifies the width of an SVG image.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Width {
    /// The width of the image
    pub width: f64,
    /// The unit of the width
    pub unit: Unit,
}

/// Parse a dimension string and return the value and unit
fn parse_dimension(s: &str) -> Result<(f64, Unit), MetadataError> {
    let caps = DIMENSION
        .captures(s)
        .ok_or_else(|| MetadataError::new("Cannot read dimensions"))?;

    let val: &str = caps
        .get(1)
        .ok_or_else(|| MetadataError::new("No width specified"))?
        .as_str();
    let unit = caps.get(2).map_or("em", |m| m.as_str());

    Ok((val.parse::<f64>()?, Unit::try_from(unit)?))
}

impl TryFrom<&str> for Width {
    type Error = MetadataError;
    fn try_from(s: &str) -> Result<Width, MetadataError> {
        let (width, unit) = parse_dimension(s)?;
        Ok(Width { width, unit })
    }
}

/// Specifies the height of an SVG image.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Height {
    /// The height of the image
    pub height: f64,
    /// The unit of the height
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
        let elements: Vec<&str> = VBOX_ELEMENTS.split(s).collect();

        if elements.len() != 4 {
            return Err(MetadataError::new(&format!(
                "Invalid view_box: Expected four elements, got {}",
                elements.len()
            )));
        }

        let min_x = elements[0].parse::<f64>()?;
        let min_y = elements[1].parse::<f64>()?;
        let width = elements[2].parse::<f64>()?;
        let height = elements[3].parse::<f64>()?;

        Ok(Self {
            min_x,
            min_y,
            width,
            height,
        })
    }
}

/// Contains all metadata that was extracted from an SVG image.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Metadata {
    /// The viewBox of the SVG image
    /// A viewBox is a rectangle that defines the dimensions of the image.
    /// For more information see: <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/viewBox>
    pub view_box: Option<ViewBox>,
    /// The width of the SVG image
    pub width: Option<Width>,
    /// The height of the SVG image
    pub height: Option<Height>,
}

impl Metadata {
    /// Parse an SVG file and extract metadata from it.
    ///
    /// # Example
    ///
    /// ```rust
    /// use svg_metadata::{Metadata, ViewBox};
    ///
    /// let meta = Metadata::parse_file("fixtures/test.svg").unwrap();
    ///     assert_eq!(
    ///     meta.view_box,
    ///     Some(ViewBox {
    ///         min_x: 0.0,
    ///         min_y: 0.0,
    ///         width: 96.0,
    ///         height: 105.0
    ///     })
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the SVG data is invalid.
    pub fn parse_file<T: Into<PathBuf>>(path: T) -> Result<Metadata, MetadataError> {
        let input = fs::read_to_string(path.into())?;
        Self::parse(input)
    }

    /// Parse SVG data and extract metadata from it.
    ///
    /// # Example
    ///
    /// ```rust
    /// use svg_metadata::{Metadata, ViewBox, Width, Height, Unit};
    ///
    /// let svg = r#"<svg viewBox="0 1 99 100" width="2em" height="10cm" xmlns="http://www.w3.org/2000/svg">
    ///  <rect x="0" y="0" width="100%" height="100%"/>
    /// </svg>"#;
    ///
    /// let meta = Metadata::parse(svg).unwrap();
    /// assert_eq!(
    ///    meta.view_box,
    ///    Some(ViewBox {
    ///      min_x: 0.0,
    ///      min_y: 1.0,
    ///      width: 99.0,
    ///      height: 100.0
    ///    })
    /// );
    /// assert_eq!(
    ///   meta.width,
    ///   Some(Width {
    ///     width: 2.0,
    ///     unit: Unit::Em
    ///   })
    /// );
    /// assert_eq!(
    ///  meta.height,
    ///  Some(Height {
    ///    height: 10.0,
    ///    unit: Unit::Cm
    ///   })
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the SVG data is invalid.
    pub fn parse<T: AsRef<str>>(input: T) -> Result<Metadata, MetadataError> {
        let doc = roxmltree::Document::parse_with_options(
            input.as_ref(),
            roxmltree::ParsingOptions {
                // Allow DTDs (e.g. `<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN"`)
                // See [`roxmltree` docs](https://docs.rs/roxmltree/latest/roxmltree/struct.ParsingOptions.html#structfield.allow_dtd)
                // for more info
                allow_dtd: true,
                ..Default::default()
            },
        )?;

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

    /// Returns the value of the `width` attribute.
    ///
    /// If the width is set to 100% then this refers to
    /// the width of the viewbox.
    #[must_use]
    pub fn width(&self) -> Option<f64> {
        let width = self.width?;

        if width.unit == Unit::Percent {
            if let Some(view_box) = self.view_box {
                return Some(width.width / 100.0 * view_box.width);
            }
        }

        Some(width.width)
    }

    /// Returns the value of the `height` attribute.
    ///
    /// If the height is set to 100% then this refers to
    /// the height of the viewbox.
    #[must_use]
    pub fn height(&self) -> Option<f64> {
        let height = self.height?;

        if height.unit == Unit::Percent {
            if let Some(view_box) = self.view_box {
                return Some(height.height / 100.0 * view_box.height);
            }
        }

        Some(height.height)
    }

    /// Return `view_box`
    #[must_use]
    pub const fn view_box(&self) -> Option<ViewBox> {
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
            );
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
        );
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
        ];
        for (input, expected) in tests {
            assert_eq!(Height::try_from(input).unwrap(), expected);
        }
    }

    #[test]
    fn test_width_height_percent() {
        let svg = r#"<svg viewBox="0 1 99 100" width="100%" height="100%" xmlns="http://www.w3.org/2000/svg">
  <rect x="0" y="0" width="100%" height="100%"/>
</svg>"#;

        let meta = Metadata::parse(svg).unwrap();
        assert_eq!(meta.width(), Some(99.0));
        assert_eq!(meta.height(), Some(100.0));

        let svg = r#"<svg viewBox="0 1 80 200" width="50%" height="20%" xmlns="http://www.w3.org/2000/svg"></svg>"#;

        let meta = Metadata::parse(svg).unwrap();
        assert_eq!(meta.width(), Some(40.0));
        assert_eq!(meta.height(), Some(40.0));
    }

    #[test]
    fn test_metadata_unit() {
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
            meta.view_box(),
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
        );
    }
}

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doctest!("../README.md");
