use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use markup5ever::{Attribute, QualName};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::cell::Ref;
use std::default::Default;
use std::rc::Rc;

pub struct Document {
    //{{{
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

        Self { doc }
    }
}

impl From<String> for Document {
    fn from(input: String) -> Self {
        let doc = parse_document(RcDom::default(), default_parse_opts())
            .from_utf8()
            .read_from(&mut input.as_bytes())
            .expect("could not parse html input");

        Self { doc }
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

impl Document {
    pub fn select(&self, selector: &str) -> Vec<Element> {
        let sel = Selector::from(selector);
        sel.find(self.doc.document.children.borrow())
    }
} //}}}

#[derive(Debug, PartialEq, Clone)]
struct Matcher {
    //{{{
    tag: Vec<String>,
    class: Vec<String>,
    id: Vec<String>,
    direct_match: bool,
}

impl From<String> for Matcher {
    fn from(input: String) -> Self {
        Self::from(&input[..])
    }
}

impl From<&str> for Matcher {
    fn from(input: &str) -> Self {
        let mut segments = vec![];
        let mut buf = "".to_string();

        for c in input.chars() {
            if c == '>' {
                return Self {
                    tag: vec![],
                    class: vec![],
                    id: vec![],
                    direct_match: true,
                };
            }
            if c == '#' || c == '.' || c == '[' {
                segments.push(buf);
                buf = "".to_string();
            }
            buf.push(c);
        }
        segments.push(buf);

        let mut res = Self {
            tag: vec![],
            class: vec![],
            id: vec![],
            direct_match: false,
        };

        for segment in segments {
            if segment.len() > 0 {
                match segment.chars().next() {
                    Some('#') => res.id.push(segment[1..].to_string()),
                    Some('.') => res.class.push(segment[1..].to_string()),
                    None => {}
                    _ => res.tag.push(segment),
                }
            }
        }

        res
    }
}

impl Matcher {
    fn matches(&self, name: &QualName, attrs: Ref<'_, Vec<Attribute>>) -> bool {
        let mut id_match = true;
        if let Some(el_id) = get_attr(&attrs, "id") {
            let el_ids: Vec<_> = el_id.split_whitespace().collect();
            id_match = self.id.iter().all(|id| el_ids.iter().any(|eid| eid == id))
        }

        let mut class_match = true;
        if let Some(el_class) = get_attr(&attrs, "class") {
            let el_classes: Vec<_> = el_class.split_whitespace().collect();

            class_match = self
                .class
                .iter()
                .all(|class| el_classes.iter().any(|eclass| eclass == class))
        }

        let name = name.local.to_string();
        println!(
            "for {} matches {} && {} && {}",
            &name,
            self.tag.iter().any(|tag| &name == tag),
            id_match,
            class_match
        );
        self.tag.iter().any(|tag| &name == tag) && id_match && class_match
    }
}
//}}}

#[derive(Debug, PartialEq)]
struct Selector {
    //{{{
    matchers: Vec<Matcher>,
}

impl From<&str> for Selector {
    fn from(input: &str) -> Self {
        let matchers: Vec<_> = input.split_whitespace().map(Matcher::from).collect();

        Selector { matchers }
    }
}

fn get_attr(attrs: &Ref<'_, Vec<Attribute>>, name: &str) -> Option<String> {
    attrs
        .iter()
        .filter(|attr| &attr.name.local == name)
        .take(1)
        .map(|attr| attr.value.to_string())
        .collect::<Vec<_>>()
        .pop()
}

impl Selector {
    fn find_nodes(
        &self,
        matcher: &Matcher,
        elements: Vec<Handle>,
        direct_match: bool,
    ) -> Vec<Handle> {
        let mut acc = vec![];

        for el in elements.iter() {
            if !direct_match {
                let children: Vec<_> = el.children.borrow().iter().map(Rc::clone).collect();
                acc.append(&mut self.find_nodes(matcher, children, false));
            }

            match el.data {
                NodeData::Element {
                    ref name,
                    ref attrs,
                    ..
                } if matcher.matches(name, attrs.borrow()) => {
                    acc.push(Rc::clone(&el));
                }
                _ => {}
            };
        }

        acc
    }

    fn find(&self, elements: Ref<'_, Vec<Handle>>) -> Vec<Element> {
        let mut elements: Vec<_> = elements.iter().map(Rc::clone).collect();
        println!("matchers: {:?}", self.matchers.clone());

        let mut direct_match = false;

        for matcher in &self.matchers {
            if matcher.direct_match {
                direct_match = true;
                elements = elements
                    .iter()
                    .map(|el| {
                        el.children
                            .borrow()
                            .iter()
                            .map(Rc::clone)
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .collect();
                continue;
            }
            elements = self.find_nodes(matcher, elements, direct_match);
            direct_match = false;
        }

        elements.iter().map(Element::from).collect()
    }
} //}}}

pub struct Element {
    //{{{
    handle: Handle,
}

impl From<Handle> for Element {
    fn from(e: Handle) -> Self {
        Self::from(&e)
    }
}

impl From<&Handle> for Element {
    fn from(e: &Handle) -> Self {
        Element {
            handle: Rc::clone(e),
        }
    }
}

impl Element {
    pub fn attr(&self, name: &str) -> Option<String> {
        match self.handle.data {
            NodeData::Element { ref attrs, .. } => get_attr(&attrs.borrow(), name),
            _ => None,
        }
    }

    pub fn tag(&self) -> Option<String> {
        match self.handle.data {
            NodeData::Element { ref name, .. } => Some(name.local.to_string()),
            _ => None,
        }
    }
} //}}}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_from_str() {
        Document::from("<a>hi there</a>");
        assert!(true);
    }

    // Matcher tests{{{
    #[test]
    fn test_matcher_tag() {
        let m = Matcher::from("a");
        assert_eq!(
            m,
            Matcher {
                tag: vec!["a".to_string()],
                class: vec![],
                id: vec![],
                direct_match: false,
            }
        );
    }

    #[test]
    fn test_matcher_complex() {
        let m = Matcher::from("a.link.another_class#idofel.klass");
        assert_eq!(
            m,
            Matcher {
                tag: vec!["a".to_string()],
                class: vec![
                    "link".to_string(),
                    "another_class".to_string(),
                    "klass".to_string()
                ],
                id: vec!["idofel".to_string()],
                direct_match: false,
            }
        );
    }

    #[test]
    fn test_matcher_direct_match() {
        let m = Matcher::from(">");
        assert_eq!(
            m,
            Matcher {
                tag: vec![],
                class: vec![],
                id: vec![],
                direct_match: true,
            }
        );
    } //}}}

    // // Selector tests{{{
    // #[test]
    // fn test_selector_parse_simple() {
    //     let sel = Selector::from("a");
    //     assert_eq!(
    //         sel,
    //         Selector {
    //             css: "a".to_string(),
    //             next: None
    //         }
    //     );
    // }

    // #[test]
    // fn test_selector_parse_simple_with_class() {
    //     let sel = Selector::from("a.link");
    //     assert_eq!(
    //         sel,
    //         Selector {
    //             css: "a.link".to_string(),
    //             next: None
    //         }
    //     );
    // } //}}}

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

    #[test]
    fn test_el_attr_double_id() {
        let doc = Document::from("<a class='link' id='linkone linkmain'>hi there</a>");
        let sel = doc.select("a#linkone#linkmain");
        let el = sel.first().unwrap();
        assert_eq!(el.attr("class"), Some("link".to_string()));
    }

    #[test]
    fn test_el_attr_double_class() {
        let doc = Document::from("<a class='link button' id='linkmain'>hi there</a>");
        let sel = doc.select("a.link.button");
        let el = sel.first().unwrap();
        assert_eq!(el.attr("id"), Some("linkmain".to_string()));
    }

    #[test]
    fn test_el_attr_double_class_reverse_order() {
        let doc = Document::from("<a class='link button' id='linkmain'>hi there</a>");
        let sel = doc.select("a.button.link");
        let el = sel.first().unwrap();
        assert_eq!(el.attr("id"), Some("linkmain".to_string()));
    }

    #[test]
    fn test_el_nested_selection() {
        let doc = Document::from(
            "<div class='container'><a class='link button' id='linkmain'>hi there</a></div>",
        );
        let sel = doc.select("div.container a.button.link");
        println!("found {:#?}", sel.len());
        println!(
            "found {:#?}",
            sel.iter().map(|e| e.tag()).collect::<Vec<_>>()
        );
        let el = sel.first().unwrap();
        assert_eq!(el.attr("id"), Some("linkmain".to_string()));
    }

    #[test]
    fn test_el_nested_selection_with_el_in_between() {
        let doc = Document::from(
            "<div class='container'><span>text</span><a class='link button' id='linkmain'>hi there</a></div>",
        );
        let sel = doc.select("div.container a.button.link");
        let el = sel.first().unwrap();
        assert_eq!(el.attr("id"), Some("linkmain".to_string()));
    }

    #[test]
    fn test_el_double_nested_selection() {
        let doc = Document::from(
            "<div class='container'><span>text<a class='link button' id='linkmain'>hi there</a></span></div>",
        );
        let sel = doc.select("div.container a.button.link");
        let el = sel.first().unwrap();
        assert_eq!(el.attr("id"), Some("linkmain".to_string()));
    }

    #[test]
    fn test_el_double_nested_direct_child_no_match() {
        let doc = Document::from(
            "<div class='container'><span>text<a class='link button' id='linkmain'>hi there</a></span></div>",
        );
        let sel = doc.select("div.container > a.button.link");
        let el = sel.first();
        assert!(el.is_none());
    }

    #[test]
    fn test_el_double_nested_direct_child_match() {
        let doc = Document::from(
            "<div class='container'><a class='link button' id='linkmain'><span>text hi there</span></a></div>",
        );
        let sel = doc.select("div.container > a.button.link");
        let el = sel.first();
        assert!(el.is_some());
    }
}
