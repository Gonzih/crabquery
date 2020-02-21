# RQuery

Is a small and simple library to query HTML markup.
Supports more complicated css selectors than other similar libraries.

## Examples

```rust
use rquery::Document;

let doc = Document::from(
    "<div class='container'><a class='link button' id='linkmain'><span>text hi there</span></a></div>",
    );
let sel = doc.select("div.container > a.button.link");
let el = sel.first();

assert_eq!(el.attr("id"), Some("linkmain".to_string()));
```
