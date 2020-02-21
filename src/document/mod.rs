use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use markup5ever::{Attribute, QualName};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::cell::Ref;
use std::collections::HashMap;
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
    /// Create document from a string slice
    fn from(input: &str) -> Self {
        let doc = parse_document(RcDom::default(), default_parse_opts())
            .from_utf8()
            .read_from(&mut input.as_bytes())
            .expect("could not parse html input");

        Self { doc }
    }
}

impl From<String> for Document {
    /// Create document from String
    fn from(input: String) -> Self {
        Self::from(&input[..])
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
    /// Select elements using given css selector
    pub fn select(&self, selector: &str) -> Vec<Element> {
        let sel = Selector::from(selector);
        sel.find(self.doc.document.children.borrow())
    }
} //}}}

#[derive(Debug, PartialEq, Clone)]
enum AttributeSpec {
    /// Implementation of [attribute] selector
    Present,
    /// Implementation of [attribute="value"] selector
    Exact(String),
    // Implementation of [attribute~="value"] selector
    // ContainsWord(String, String),
    // Implementation of [attribute|="value"] selector
    // StartsWord(String, String),
    /// Implementation of [attribute^="value"] selector
    Starts(String),
    /// Implementation of [attribute$="value"] selector
    Ends(String),
    /// Implementation of [attribute*="value"] selector
    Contains(String),
}

impl AttributeSpec {
    fn matches(&self, other: String) -> bool {
        use AttributeSpec::*;

        match self {
            Present => true,
            Exact(v) => &other == v,
            Starts(v) => other.starts_with(v),
            Ends(v) => other.ends_with(v),
            Contains(v) => other.contains(v),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Matcher {
    //{{{
    tag: Vec<String>,
    class: Vec<String>,
    id: Vec<String>,
    attribute: HashMap<String, AttributeSpec>,
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
            match c {
                '>' => {
                    return Self {
                        tag: vec![],
                        class: vec![],
                        id: vec![],
                        attribute: HashMap::new(),
                        direct_match: true,
                    };
                }
                '#' | '.' | '[' => {
                    segments.push(buf);
                    buf = "".to_string();
                }
                ']' => {
                    segments.push(buf);
                    buf = "".to_string();
                    continue;
                }
                _ => {}
            };

            buf.push(c);
        }
        segments.push(buf);

        let mut res = Self {
            tag: vec![],
            class: vec![],
            id: vec![],
            attribute: HashMap::new(),
            direct_match: false,
        };

        for segment in segments {
            match segment.chars().next() {
                Some('#') => res.id.push(segment[1..].to_string()),
                Some('.') => res.class.push(segment[1..].to_string()),
                Some('[') => res.add_data_attribute(segment[1..].to_string()),
                None => {}
                _ => res.tag.push(segment),
            }
        }

        res
    }
}

impl Matcher {
    fn add_data_attribute(&mut self, spec: String) {
        use AttributeSpec::*;

        let parts = spec.split('=').collect::<Vec<_>>();

        if parts.len() == 1 {
            let k = parts[0];
            self.attribute.insert(k.to_string(), Present);
            return;
        }

        let v = parts[1].trim_matches('"').to_string();
        let k = parts[0];
        let k = k[..k.len() - 1].to_string();

        match parts[0].chars().last() {
            Some('^') => {
                self.attribute.insert(k, Starts(v));
            }
            Some('$') => {
                self.attribute.insert(k, Ends(v));
            }
            Some('*') => {
                self.attribute.insert(k, Contains(v));
            }
            Some(_) => {
                let k = parts[0].to_string();
                self.attribute.insert(k, Exact(v));
            }
            None => {
                panic!("Colud not parse attribute spec \"{}\"", spec);
            }
        }
    }

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

        let mut attr_match = true;
        for (k, v) in &self.attribute {
            if let Some(value) = get_attr(&attrs, &k[..]) {
                if !v.matches(value) {
                    attr_match = false;
                    break;
                }
            }
        }

        let name = name.local.to_string();
        self.tag.iter().any(|tag| &name == tag) && id_match && class_match && attr_match
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
    /// Get value of an attribue
    pub fn attr(&self, name: &str) -> Option<String> {
        match self.handle.data {
            NodeData::Element { ref attrs, .. } => get_attr(&attrs.borrow(), name),
            _ => None,
        }
    }

    /// Get tag value
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
    }

    // Matcher tests{{{
    #[test]
    fn test_matcher_tag() {
        let m = Matcher::from("a");
        assert_eq!(m.tag, vec!["a".to_string()],);
    }

    #[test]
    fn test_matcher_complex() {
        let m = Matcher::from("a.link.another_class#idofel.klass");
        assert_eq!(m.tag, vec!["a".to_string()]);
        assert_eq!(
            m.class,
            vec![
                "link".to_string(),
                "another_class".to_string(),
                "klass".to_string()
            ]
        );
        assert_eq!(m.id, vec!["idofel".to_string()]);
    }

    #[test]
    fn test_matcher_direct_match() {
        let m = Matcher::from(">");
        assert_eq!(m.direct_match, true);
    }

    #[test]
    fn test_matcher_data_attribute_present() {
        let m = Matcher::from("a[target]");
        let mut attr = HashMap::new();
        attr.insert("target".to_string(), AttributeSpec::Present);
        assert_eq!(m.attribute, attr);
    }

    #[test]
    fn test_matcher_data_attribute_exact() {
        let m = Matcher::from("a[target=\"_blank\"]");
        let mut attr = HashMap::new();
        attr.insert(
            "target".to_string(),
            AttributeSpec::Exact("_blank".to_string()),
        );
        assert_eq!(m.attribute, attr);
    }

    #[test]
    fn test_matcher_data_attribute_starts() {
        let m = Matcher::from("a[target^=\"_blank\"]");
        let mut attr = HashMap::new();
        attr.insert(
            "target".to_string(),
            AttributeSpec::Starts("_blank".to_string()),
        );
        assert_eq!(m.attribute, attr);
    }

    #[test]
    fn test_matcher_data_attribute_ends() {
        let m = Matcher::from("a[target$=\"_blank\"]");
        let mut attr = HashMap::new();
        attr.insert(
            "target".to_string(),
            AttributeSpec::Ends("_blank".to_string()),
        );
        assert_eq!(m.attribute, attr);
    }

    #[test]
    fn test_matcher_data_attribute_contains() {
        let m = Matcher::from("a[target*=\"_blank\"]");
        let mut attr = HashMap::new();
        attr.insert(
            "target".to_string(),
            AttributeSpec::Contains("_blank".to_string()),
        );
        assert_eq!(m.attribute, attr);
    }

    //}}}

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

    // Element tests{{{
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
            "<div class='container'>
               <a class='link button' id='linkmain'>
                 <span>text hi there</span>
               </a>
             </div>",
        );
        let sel = doc.select("div.container > a.button.link");
        let el = sel.first();
        assert!(el.is_some());
    }

    #[test]
    fn test_simple_multiple_a() {
        let doc = Document::from(
            "<div class='container'>
               <a class='link button' id='linkmain'>
                 <span>text hi there</span>
               </a>
               <span>text hi there <a href='blob'>two</a></span>
             </div>",
        );
        let sel = doc.select("a");
        assert_eq!(sel.len(), 2);
    }

    #[test]
    fn test_simple_multiple_a_in_div() {
        let doc = Document::from(
            "<div class='container'>
               <a class='link button' id='linkmain'>
                 <span>text hi there</span>
               </a>
             </div>
             <div>
               <span>text hi there
                 <a class='blob'>two</a>
               </span>
             </div>
             ",
        );
        let sel = doc.select("div a");
        assert_eq!(sel.len(), 2);
    }

    #[test]
    fn test_simple_attribute_present() {
        let doc = Document::from(
            "<div>
               <span>text hi there
                 <a data-counter='blob'>two</a>
               </span>
             </div>",
        );
        let sel = doc.select("div > span > a[data-counter]");
        assert_eq!(sel.len(), 1);
    }

    #[test]
    fn test_simple_attribute_starts() {
        let doc = Document::from(
            "<div>
               <span>text hi there
                 <a data-counter='blobovo'>two</a>
               </span>
             </div>",
        );
        let sel = doc.select("div > span > a[data-counter^=\"blob\"]");
        assert_eq!(sel.len(), 1);
    }

    #[test]
    fn test_simple_attribute_ends() {
        let doc = Document::from(
            "<div>
               <span>text hi there
                 <a data-counter='blobovo'>two</a>
               </span>
             </div>",
        );
        let sel = doc.select("div > span > a[data-counter$=\"ovo\"]");
        assert_eq!(sel.len(), 1);
    }

    #[test]
    fn test_simple_attribute_contains() {
        let doc = Document::from(
            "<div>
               <span>text hi there
                 <a data-counter='blobovo'>two</a>
               </span>
             </div>",
        );
        let sel = doc.select("div > span > a[data-counter*=\"obo\"]");
        assert_eq!(sel.len(), 1);
    }

    //}}}
}
