use self::tags::{TagType, A};
use std::collections::HashSet;
use std::iter::IntoIterator;


#[derive(Eq, PartialEq, Debug)]
pub struct Tag<T> {
    tag_type: T,
    classes: HashSet<String>,
    ids: HashSet<String>,
    name: Option<String>
}


impl<T: TagType> Tag<T> {
    pub fn new(tag_type: T) -> Tag<T> {
        Tag { tag_type: tag_type, ids: HashSet::new(), classes: HashSet::new(), name: None }
    }

    pub fn add_class<S: Into<String>>(mut self, class: S) -> Tag<T> {
        self.classes.insert(class.into());
        self
    }

    pub fn add_id<S: Into<String>>(mut self, id: S) -> Tag<T> {
        self.ids.insert(id.into());
        self
    }

    pub fn set_name<S: Into<String>>(mut self, name: S) -> Tag<T> {
        self.name = Some(name.into());
        self
    }

    pub fn render_open(&self) -> String {
        let attributes = self.render_attributes();
        format!(
            "<{tag}{delim}{attributes}>",
            tag = self.tag_type.keyword(),
            delim = if attributes.len() > 0 { " " } else { "" },
            attributes = attributes
        )
    }

    pub fn render_close(&self) -> String {
        format!("</{}>", self.tag_type.keyword())
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attributes = Vec::new();
        if self.classes.len() > 0 {
            attributes.push(("class".to_string(), concat_chunks(&self.classes, " ")));
        }
        if self.ids.len() > 0 {
            attributes.push(("id".to_string(), concat_chunks(&self.ids, " ")));
        }
        if let Some(ref name) = self.name {
            attributes.push(("name".to_string(), name.clone()));
        }
        attributes
    }

    fn render_attributes(&self) -> String {
        let attributes = self.attributes();
        let tag_type_attributes = self.tag_type.attributes();
        let iter = attributes.into_iter().chain(tag_type_attributes)
            .map(|(keyword, value)| format!("{}=\"{}\"", keyword, value));
        concat_chunks(iter, " ")
    }
}


impl Tag<A> {
    pub fn set_href<S: Into<String>>(mut self, href: S) -> Tag<A> {
        self.tag_type.href = Some(href.into());
        self
    }
}


fn concat_chunks<I, T>(chunks: I, separator: &str) -> String where I: IntoIterator<Item=T>, T: AsRef<str> {
    let mut buf = String::new();
    let mut iter = chunks.into_iter();
    if let Some(chunk) = iter.next() {
        buf.push_str(chunk.as_ref());
        for chunk in iter {
            buf.push_str(separator);
            buf.push_str(chunk.as_ref())
        }
    }
    buf
}


pub mod tags {
    use super::Tag;


    pub trait TagType {
        fn keyword(&self) -> String;
        fn attributes(&self) -> Vec<(String, String)> { Vec::new() }
    }


    #[derive(Debug)]
    pub struct Span;


    impl Span {
        pub fn new() -> Tag<Span> { Tag::new(Span) }
    }


    impl TagType for Span {
        fn keyword(&self) -> String { "span".into() }
    }


    #[derive(Debug)]
    pub struct Div;


    impl Div {
        pub fn new() -> Tag<Div> { Tag::new(Div) }
    }


    impl TagType for Div {
        fn keyword(&self) -> String { "div".into() }
    }


    #[derive(Debug)]
    pub struct A {
        pub href: Option<String>
    }


    impl A { pub fn new() -> Tag<A> { Tag::new(A { href: None }) }}


    impl TagType for A {
        fn keyword(&self) -> String { "a".to_string() }
        fn attributes(&self) -> Vec<(String, String)> {
            let mut attributes = Vec::new();
            if let &Some(ref href) = &self.href {
                attributes.push(("href".to_string(), href.clone()));
            }
            attributes
        }
    }
}


#[cfg(test)]
mod tests {
    use super::tags::{Span, A};

    #[test]
    fn test_simple_tag() {
        let tag = Span::new();
        assert_eq!("<span>", tag.render_open());
        assert_eq!("</span>", tag.render_close());
    }

    #[test]
    fn test_class_and_id() {
        let tag = Span::new().add_class("foo").add_id("bar");
        assert_eq!(r#"<span class="foo" id="bar">"#, tag.render_open());
        assert_eq!("</span>", tag.render_close());
    }

    #[test]
    fn test_href() {
        let tag = A::new().set_href("foo");
        assert_eq!(r#"<a href="foo">"#, tag.render_open());
        assert_eq!("</a>", tag.render_close());
    }
}
