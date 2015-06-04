use std::rc::Rc;
use syntax::codemap::FileMap;
use syntax::parse::ParseSess;

use syntax::parse::token::Token as CompilerToken;
use syntax::parse::lexer::{Reader, StringReader, TokenAndSpan};


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


#[derive(Clone)]
pub struct Span {
    lower_bound: usize, // inclusive
    upper_bound: usize, // inclusive
    filemap: Rc<FileMap>
}


#[derive(Clone)]
pub struct Token {
    compiler_token: CompilerToken,
    span: Span
}


pub trait GetSnippet {
    fn snippet(&self) -> Option<String>;
}


impl GetSnippet for Span {
    fn snippet(&self) -> Option<String> {
        self.filemap.src.clone()
            .map(|src| src[self.lower_bound .. self.upper_bound + 1].to_string())
    }
}


impl GetSnippet for Token {
    fn snippet(&self) -> Option<String> { self.span.snippet() }
}


fn to_internal_token(token_and_span: TokenAndSpan, filemap: Rc<FileMap>) -> Token {
    use syntax::codemap::Pos;

    Token {
        compiler_token: token_and_span.tok,
        span: Span {
            lower_bound: token_and_span.sp.lo.to_usize(),
            upper_bound: token_and_span.sp.hi.to_usize() - 1,
            filemap: filemap
        }
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
