extern crate crabquery;

use crabquery::*;

#[test]
fn test_docs_rs_index() {
    let document = Document::from(include_str!("fixtures/docs_rs.html"));
    let found_elements = document.select("div.pure-u-sm-4-24");
    assert_eq!(found_elements.len(), 15);
}
