use std::iter::FromIterator;
use lexer::{Token, IntervalToSnippet};
use custom_collections::{Stack, Queue};
use syntax::codemap::CodeMap;


#[derive(Debug)]
struct Chunk {
    position: usize,
    text: String
}


#[derive(Debug)]
struct Wrapper {
    prefix: Chunk,
    postfix: Chunk
}


#[allow(dead_code)]
fn render(codemap: &CodeMap, tokens: Vec<Token>, mut wrappers: Vec<Wrapper>) -> String {
    wrappers.sort_by(|a, b| {
        // left and longest go first
        (a.prefix.position, b.postfix.position)
            .cmp(&(b.prefix.position, a.postfix.position))
    });

    let mut wrappers = Queue::from_iter(wrappers);
    let mut postfixes = Stack::new();

    let mut buffer = String::new();
    for token in tokens {
        while wrappers.peek().map_or(false, |x| x.prefix.position == token.interval.lower_bound) {
            let wrapper = wrappers.dequeue().expect("wrappers.dequeue()");
            buffer.push_str(&wrapper.prefix.text);
            postfixes.push(wrapper.postfix);
        }

        buffer.push_str(&codemap.interval_to_snippet(&token.interval).expect("token.snippet"));

        while postfixes.peek().map_or(false, |x| x.position == token.interval.upper_bound) {
            let postfix = postfixes.pop().expect("postfixes.pop()");
            buffer.push_str(&postfix.text);
        }
    }
    buffer
}


#[cfg(test)]
mod tests {
    use syntax::codemap::CodeMap;
    use super::{render, Wrapper, Chunk};
    use lexer::{read_tokens};

    const SOURCE: &'static str = "fn main() {}\n";

    #[test]
    fn test() {
        let codemap = CodeMap::new();
        let filemap = codemap.new_filemap("".into(), SOURCE.into());
        let tokens = read_tokens(filemap);

        let wrappers = vec![
            Wrapper {
                prefix: Chunk { position: 0, text: "<:0:>".into() },
                postfix: Chunk { position: 1, text: "</:0:>".into() }
            },
            Wrapper {
                prefix: Chunk { position: 0, text: "<:1:>".into() },
                postfix: Chunk { position: 11, text: "</:1:>".into() }
            },
            Wrapper {
                prefix: Chunk { position: 10, text: "<:2:>".into() },
                postfix: Chunk { position: 11, text: "</:2:>".into() }
            },
        ];

        let result = render(&codemap, tokens, wrappers);
        assert_eq!(result, "<:1:><:0:>fn</:0:> main() <:2:>{}</:2:></:1:>\n")
    }
}
