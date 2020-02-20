#[macro_use]
use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use markup5ever::{Attribute, QualName};
use markup5ever_rcdom::{Handle, Node, NodeData, RcDom};
use std::cell::Ref;
use std::default::Default;
use std::rc::Rc;

pub struct Document {
    doc: RcDom,
}

fn default_parse_opts() -> ParseOpts {
    ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl From<&str> for Document {
    fn from(input: &str) -> Self {
        let doc = parse_document(RcDom::default(), default_parse_opts())
            .from_utf8()
            .read_from(&mut input.as_bytes())
            .expect("could not parse html input");

        Document { doc }
    }
}

impl From<String> for Document {
    fn from(input: String) -> Self {
        let doc = parse_document(RcDom::default(), default_parse_opts())
            .from_utf8()
            .read_from(&mut input.as_bytes())
            .expect("could not parse html input");

        Document { doc }
    }
}

// impl From<&[u8]> for Document {
//     fn from(input: &[u8]) -> Self {
//         let doc = parse_document(RcDom::default(), default_parse_opts())
//             .from_utf8()
//             .read_from(&mut input)
//             .expect("could not parse html input");

//             Document { doc }
//     }
// }

struct Selector {
    next: Option<Rc<Selector>>,
    css: String,
}

impl From<&str> for Selector {
    fn from(input: &str) -> Self {
        let mut sepcss = input.split_whitespace();

        Selector {
            css: sepcss.next().unwrap().to_string(),
            next: None,
        }
    }
}

impl Selector {
    fn matches(&self, name: &QualName, attrs: Ref<'_, Vec<Attribute>>) -> bool {
        name.local.to_string() == self.css
    }

    fn find(&self, elements: Ref<'_, Vec<Handle>>) -> Vec<Element> {
        let mut res = vec![];

        for el in elements.iter() {
            match el.data {
                NodeData::Element {
                    ref name,
                    ref attrs,
                    ..
                } if self.matches(name, attrs.borrow()) => {
                    res.push(Element {
                        handle: Rc::clone(&el),
                    });
                }
                _ => res.append(&mut self.find(el.children.borrow())),
            };
        }

        res
    }
}

impl Document {
    pub fn select(&self, selector: &str) -> Vec<Element> {
        let sel = Selector::from(selector);
        sel.find(self.doc.document.children.borrow())
    }
}

pub struct Element {
    handle: Handle,
}

impl Element {
    pub fn attr(&self, name: &str) -> Option<String> {
        match self.handle.data {
            NodeData::Element { ref attrs, .. } => attrs
                .borrow()
                .iter()
                .filter(|attr| attr.name.local.to_string() == name)
                .take(1)
                .map(|attr| attr.value.to_string().clone())
                .collect::<Vec<_>>()
                .pop(),
            _ => None,
        }
    }

    pub fn tag(&self) -> Option<String> {
        match self.handle.data {
            NodeData::Element { ref name, .. } => Some(name.local.to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        Document::from("<a>hi there</a>");
        assert!(true);
    }

    // Element tests
    #[test]
    fn test_el_tag() {
        let doc = Document::from("<a class='link'>hi there</a>");
        let sel = doc.select("a");
        let el = sel.first().unwrap();
        assert_eq!(el.tag(), Some("a".to_string()));
    }

    #[test]
    fn test_el_attr_class() {
        let doc = Document::from("<a class='link'>hi there</a>");
        let sel = doc.select("a");
        let el = sel.first().unwrap();
        assert_eq!(el.attr("class"), Some("link".to_string()));
    }

    #[test]
    fn test_el_attr_id() {
        let doc = Document::from("<a class='link' id=linkilink>hi there</a>");
        let sel = doc.select("a");
        let el = sel.first().unwrap();
        assert_eq!(el.attr("id"), Some("linkilink".to_string()));
    }
}
