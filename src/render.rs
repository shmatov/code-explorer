use std::iter::FromIterator;
use lexer::{Token, IntervalToSnippet};
use custom_collections::{Stack, Queue};
use syntax::codemap::FileMap;


#[derive(Debug)]
pub struct Chunk {
    pub position: usize,
    pub text: String
}

impl Chunk {
    pub fn new(position: usize, text: String) -> Chunk {
        Chunk { position: position, text: text}
    }
}


#[derive(Debug)]
pub struct Wrapper {
    pub prefix: Chunk,
    pub postfix: Chunk
}


impl Wrapper {
    pub fn new(prefix: Chunk, postfix: Chunk) -> Wrapper {
        Wrapper { prefix: prefix, postfix: postfix}
    }
}


#[allow(dead_code)]
pub fn render(filemap: &FileMap, tokens: Vec<Token>, mut wrappers: Vec<Wrapper>) -> String {
    wrappers.sort_by(|a, b| {
        // left and longest go first
        (a.prefix.position, b.postfix.position)
            .cmp(&(b.prefix.position, a.postfix.position))
    });

    println!("total wrappers: {}", wrappers.len());

    //let mut intervals = Vec::new();

    let mut wrappers = Queue::from_iter(wrappers);
    let mut postfixes = Stack::new();

    let mut buffer = String::new();
    for token in tokens {
        while wrappers.peek().map_or(false, |x| x.prefix.position == token.interval.lower_bound) {
            let wrapper = wrappers.dequeue().expect("wrappers.dequeue()");
            buffer.push_str(&wrapper.prefix.text);
            postfixes.push(wrapper.postfix);
        }

        buffer.push_str(&filemap.interval_to_snippet(&token.interval).expect("token.snippet"));

        //intervals.push((token.interval.lower_bound, token.interval.upper_bound));

        while postfixes.peek().map_or(false, |x| x.position == token.interval.upper_bound) {
            let postfix = postfixes.pop().expect("postfixes.pop()");
            buffer.push_str(&postfix.text);
        }
    }

    println!("not applied wrappers: {}", wrappers.len());
    println!("partialy applied wrappers: {}", postfixes.len());
    //for wrapper in wrappers.0 {
        //println!("wrapper. prefix: {}, postfix: {}", wrapper.prefix.position, wrapper.postfix.position);
    //}
    //for interval in intervals {
        //println!("interval. {:?}", interval);
    //}
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
