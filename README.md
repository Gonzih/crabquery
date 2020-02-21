# CrabQuery - like JQuery, but for Crabs

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

let sel = doc.select("div a span");
let el = sel.first().unwrap();

assert_eq!(el.text(), Some("text hi there".to_string()));
```
