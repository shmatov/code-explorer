use std::rc::Rc;
use syntax::codemap::{self, FileMap, CodeMap};
use syntax::parse::ParseSess;

use syntax::parse::token::Token as CompilerToken;
use syntax::parse::lexer::{Reader, StringReader, TokenAndSpan};


#[allow(dead_code)]
pub fn read_tokens(filemap: Rc<FileMap>) -> Vec<Token> {
    let sess = ParseSess::new();
    filemap.lines.borrow_mut().clear(); // this hack allows tokenize file one more time
    let mut lexer = StringReader::new(&sess.span_diagnostic, filemap.clone());

    let mut tokens = Vec::new();
    loop {
        let token = to_internal_token(lexer.next_token(), filemap.clone());
        if token.compiler_token == CompilerToken::Eof {
            return tokens;
        }
        tokens.push(token);
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Interval {
    pub lower_bound: usize, // inclusive
    pub upper_bound: usize, // inclusive
    filename: String,
}


#[derive(Clone, Debug)]
pub struct Token {
    pub compiler_token: CompilerToken,
    pub interval: Interval
}


pub fn to_internal_token(token_and_span: TokenAndSpan, filemap: Rc<FileMap>) -> Token {
    Token {
        compiler_token: token_and_span.tok,
        interval: to_interval(token_and_span.sp, filemap)
    }
}


pub fn to_interval(span: codemap::Span, filemap: Rc<FileMap>) -> Interval {
    use syntax::codemap::Pos;

    let lower_bound = span.lo.to_usize() - filemap.start_pos.to_usize();
    let mut upper_bound = span.hi.to_usize() - filemap.start_pos.to_usize();
    if upper_bound > 0 {
        upper_bound -= 1;
    }

    Interval {
        lower_bound: lower_bound,
        upper_bound: upper_bound,
        filename: filemap.name.clone(),
    }
}


pub trait IntervalToSnippet {
    fn interval_to_snippet(&self, &Interval) -> Option<String>;
}


impl IntervalToSnippet for FileMap {
    fn interval_to_snippet(&self, interval: &Interval) -> Option<String> {
        match (self.src.clone(), self.name == interval.filename) {
            (Some(src), true) => {
                let snippet = src[interval.lower_bound .. interval.upper_bound + 1].to_string();
                Some(escape_shippet(snippet))
            },
            _ => None
        }
    }
}


impl IntervalToSnippet for CodeMap {
    fn interval_to_snippet(&self, interval: &Interval) -> Option<String> {
        self.files.borrow().iter()
            .find(|filemap| filemap.name == interval.filename)
            .and_then(|filemap| filemap.interval_to_snippet(interval))
    }
}


fn escape_shippet(snippet: String) -> String {
    match &snippet[..] {
        "<" => "&lt;".to_string(),
        ">" => "&gt;".to_string(),
        _ => snippet
    }
}


#[cfg(test)]
mod tests {
    use syntax::codemap::CodeMap;
    use super::{read_tokens, IntervalToSnippet};

    const SOURCE: &'static str = "fn main() {}\n";

    #[test]
    fn test_read_tokens() {
        let codemap = CodeMap::new();
        let filemap = codemap.new_filemap("".into(), SOURCE.into());
        let tokens = read_tokens(filemap.clone());

        for token in tokens.iter() {
            println!(
                "[{}, {}] -> {:?} -> {:?}",
                token.interval.lower_bound, token.interval.upper_bound,
                token.compiler_token, filemap.interval_to_snippet(&token.interval)
            );
        }

        let source = tokens.iter()
            .map(|token| filemap.interval_to_snippet(&token.interval).unwrap())
            .collect::<Vec<_>>().concat();

        assert_eq!(source, SOURCE);
    }
}
