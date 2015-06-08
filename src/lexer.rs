use std::rc::Rc;
use syntax::codemap::{self, FileMap};
use syntax::parse::ParseSess;

use syntax::parse::token::Token as CompilerToken;
use syntax::parse::lexer::{Reader, StringReader, TokenAndSpan};


#[allow(dead_code)]
pub fn read_tokens(filemap: Rc<FileMap>) -> Vec<Token> {
    let sess = ParseSess::new();
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


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    pub lower_bound: usize, // inclusive
    pub upper_bound: usize, // inclusive
    filename: String,
    snippet: Option<String>
}


#[derive(Clone)]
pub struct Token {
    pub compiler_token: CompilerToken,
    pub span: Span
}


pub trait GetSnippet {
    fn snippet(&self) -> Option<String>;
}


impl GetSnippet for Span {
    fn snippet(&self) -> Option<String> { self.snippet.clone() }
}


impl GetSnippet for Token {
    fn snippet(&self) -> Option<String> { self.span.snippet() }
}


pub fn to_internal_token(token_and_span: TokenAndSpan, filemap: Rc<FileMap>) -> Token {
    Token {
        compiler_token: token_and_span.tok,
        span: to_internal_span(token_and_span.sp, filemap)
    }
}


pub fn to_internal_span(span: codemap::Span, filemap: Rc<FileMap>) -> Span {
    use syntax::codemap::Pos;

    let lower_bound = span.lo.to_usize();
    let mut upper_bound = span.hi.to_usize();
    if upper_bound > 0 {
        upper_bound -= 1;
    }
    let snippet = filemap.src.clone().map(|src| src[lower_bound .. upper_bound + 1].to_string());

    Span {
        lower_bound: lower_bound,
        upper_bound: upper_bound,
        filename: filemap.name.clone(),
        snippet: snippet
    }
}


#[cfg(test)]
mod tests {
    use syntax::codemap::CodeMap;
    use super::{read_tokens, GetSnippet};

    const SOURCE: &'static str = "fn main() {}\n";

    #[test]
    fn test_read_tokens() {
        let codemap = CodeMap::new();
        let filemap = codemap.new_filemap("".into(), SOURCE.into());
        let tokens = read_tokens(filemap);

        for token in tokens.iter() {
            println!(
                "[{}, {}] -> {:?} -> {:?}",
                token.span.lower_bound, token.span.upper_bound,
                token.compiler_token, token.snippet()
            );
        }

        let source = tokens.iter()
            .map(|token| token.snippet().unwrap())
            .collect::<Vec<_>>().concat();

        assert_eq!(source, SOURCE);
    }
}
