extern crate crabquery;

use crabquery::*;

#[test]
fn test_docs_rs_index() {
    let document = Document::from(include_str!("fixtures/docs_rs.html"));

    let els = document.select("div.pure-u-sm-4-24");
    assert_eq!(els.len(), 15);

    let els = document.select(".pure-u-sm-4-24");
    assert_eq!(els.len(), 15);

    let els = document.select("meta[name=\"generator\"]");
    assert_eq!(els.len(), 1);
}
