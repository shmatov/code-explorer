use lexer::{Token, GetSnippet};
use custom_collections::{Stack, Queue};


#[derive(Debug)]
struct Chunk {
    position: usize,
    text: String
}


#[derive(Debug)]
struct Wrapper {
    before: Chunk,
    after: Chunk
}


#[allow(dead_code)]
fn render(tokens: Vec<Token>, mut wrappers: Vec<Wrapper>) -> String {
    wrappers.sort_by(|a, b| {
        // left and longest go first
        (a.before.position, -(a.after.position as isize)).cmp(&(b.before.position, -(b.after.position as isize)))
    });

    let mut queue = Queue::new();
    for wrapper in wrappers {
        queue.enqueue(wrapper);
    }

    let mut stack = Stack::new();

    let mut buf = String::new();
    for token in tokens {
        // try prepend
        while queue.peek().map_or(false, |x| x.before.position == token.span.lower_bound) {
            let wrapper = queue.dequeue().expect("queue.pop()");
            buf.push_str(&wrapper.before.text);
            stack.push(wrapper);
        }

        // append token
        buf.push_str(&token.snippet().expect("token.snippet().pop()"));

        // try append
        while stack.peek().map_or(false, |x| x.after.position == token.span.upper_bound) {
            let wrapper = stack.pop().expect("stack.pop()");
            buf.push_str(&wrapper.after.text);
        }
    }
    buf
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
                before: Chunk { position: 0, text: "<:0:>".into() },
                after: Chunk { position: 1, text: "</:0:>".into() }
            },
            Wrapper {
                before: Chunk { position: 0, text: "<:1:>".into() },
                after: Chunk { position: 11, text: "</:1:>".into() }
            },
            Wrapper {
                before: Chunk { position: 10, text: "<:2:>".into() },
                after: Chunk { position: 11, text: "</:2:>".into() }
            },
        ];

        let result = render(tokens, wrappers);
        assert_eq!(result, "<:1:><:0:>fn</:0:> main() <:2:>{}</:2:></:1:>\n")
    }
}
