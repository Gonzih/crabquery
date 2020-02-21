//! Library for quick and easy DOM search based on CSS queries for your scraping needs.
//!
//! Supported selectors are:
//! * tag based `span` or `a`
//! * class based `.button`
//! * id based `#mainbutton`
//! * direct child `>`
//! * attribute selectors `[href]`, `[href="specific-value"]`, `[href*="contains-str"]`,
//! `[href^="begins-with"]`,, `[href$="ends-with"]`
//! * all combinations of above like `div.container > form#feedback input.button`
#![crate_name = "crabquery"]

mod document;

pub use document::*;
