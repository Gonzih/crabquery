# CrabQuery - like JQuery, but for Crabs

[![CI][ci-badge]][ci-url]
[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/crabquery.svg
[crates-url]: https://crates.io/crates/crabquery
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[ci-badge]: https://github.com/Gonzih/crabquery/workflows/CI/badge.svg
[ci-url]: https://github.com/Gonzih/crabquery/actions

Small and simple library to query HTML markup for your web scraping needs.

Based on servo libraries.
Supports more complicated CSS selectors than other similar libraries.

## Examples

```rust
use crabquery::Document;

let doc = Document::from(
    "<div class='container'>
       <a class='link button' id='linkmain'>
         <span>text hi there</span>
       </a>
     </div>",
);

let sel = doc.select("div.container > a.button.link[id=\"linkmain\"]");
let el = sel.first().unwrap();

assert_eq!(el.attr("id"), Some("linkmain".to_string()));

let sel = doc.select("div > a > span");
let el = sel.first().unwrap();

assert_eq!(el.text(), Some("text hi there".to_string()));
```
