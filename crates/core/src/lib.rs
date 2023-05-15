use std::{collections::HashSet, string::FromUtf8Error};

use tree_sitter::{Node, Parser, Query, QueryCursor};

mod fields {
    pub const BODY: u16 = 5;
    pub const OBJECT: u16 = 28;
    pub const PROPERTY: u16 = 35;
}

mod kinds {
    pub const FUNC_EXPR: u16 = 218;
    pub const FUNC_DECL: u16 = 219;
    pub const GEN_EXPR: u16 = 220;
    pub const GEN_DECL: u16 = 221;
    pub const FUNC_ARROW: u16 = 222;

    #[inline(always)]
    pub fn is_func(x: u16) -> bool {
        matches!(x, FUNC_EXPR | FUNC_DECL | GEN_EXPR | GEN_DECL | FUNC_ARROW)
    }
}

mod literals {
    pub const EARLY_RETURN_BEGIN: &str = "try {";
    pub const EARLY_RETURN_END: &str =
        "} catch (e) { if (EARLY_RETURN in e) return e[EARLY_RETURN]; throw e; }";

    pub const UNWRAP_BEGIN: &str = "__unwrap(";
    pub const UNWRAP_END: &str = ")";

    pub const IMPORT: &str = "import { EARLY_RETURN, __unwrap } from \"foo\";\n";

    pub const MECHANISM: &str = r#"
const EARLY_RETURN = Symbol();
const __unwrap = x => {
    if (x.isOk) return x.value;
    throw {[EARLY_RETURN]: x};
};
"#;
}

#[derive(Clone, Debug)]
enum StringOp<'a> {
    Insert(usize, &'a str),
    Delete(usize, usize),
}

impl<'a> StringOp<'a> {
    fn start(&self) -> usize {
        match self {
            Self::Insert(i, _) => *i,
            Self::Delete(i, _) => *i,
        }
    }

    fn apply(&self, buf: &mut Vec<u8>, offset: isize) -> isize {
        match self {
            Self::Insert(i, x) => {
                let i = (*i as isize + offset) as usize;
                buf.splice(i..i, x.as_bytes().iter().copied());
            }
            Self::Delete(from, to) => {
                let from = (*from as isize + offset) as usize;
                let to = (*to as isize + offset) as usize;
                buf.splice(from..to, std::iter::empty());
            }
        };

        match self {
            Self::Insert(_, s) => s.len() as isize,
            Self::Delete(from, to) => *from as isize - *to as isize,
        }
    }
}

struct StringModifier<'a> {
    ops: Vec<StringOp<'a>>,
}

impl<'a> StringModifier<'a> {
    fn new() -> Self {
        Self { ops: Vec::new() }
    }

    fn insert(&mut self, index: usize, s: &'a str) {
        self.ops.push(StringOp::Insert(index, s));
    }

    fn delete(&mut self, from: usize, to: usize) {
        self.ops.push(StringOp::Delete(from, to));
    }

    fn modify(mut self, s: &str) -> Result<String, FromUtf8Error> {
        let mut res = s.as_bytes().to_vec();

        self.ops.sort_unstable_by_key(|x| x.start());

        let mut offset = 0;
        for op in self.ops {
            offset += op.apply(&mut res, offset);
        }

        String::from_utf8(res)
    }
}

// TODO: Handle case where arrow function returns an expression, not a statement block.
//       Example: () => a.$ + b.$
fn insert_early_return_mechanism(function_decl: Node) -> (usize, usize) {
    let body = function_decl.child_by_field_id(fields::BODY).unwrap();
    let l_brace = body.child(0).unwrap();
    let r_brace = body.child(body.child_count() - 1).unwrap();

    (l_brace.end_byte(), r_brace.start_byte())
}

pub fn process(input: &str) -> Result<String, FromUtf8Error> {
    let lang = tree_sitter_typescript::language_typescript();
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(input, None).unwrap();

    let query = r#"
        (member_expression
            property: (
                (property_identifier) @prop
                (#eq? @prop "$")
            )
        ) @member
    "#;
    let query = Query::new(lang, query).unwrap();
    let mut cursor = QueryCursor::new();

    let mut modifier = StringModifier::new();
    let mut func_visited = HashSet::new();

    for match_ in cursor.matches(&query, tree.root_node(), input.as_bytes()) {
        let member_expr = match_.captures.first().unwrap().node;

        let object = member_expr.child_by_field_id(fields::OBJECT).unwrap();
        let dot = member_expr.child(1).unwrap();
        let prop = member_expr.child_by_field_id(fields::PROPERTY).unwrap();

        modifier.insert(object.start_byte(), literals::UNWRAP_BEGIN);
        modifier.insert(object.end_byte(), literals::UNWRAP_END);
        modifier.delete(dot.start_byte(), dot.end_byte());
        modifier.delete(prop.start_byte(), prop.end_byte());

        let mut curr = member_expr;
        while let Some(parent) = curr.parent() {
            curr = parent;
            if kinds::is_func(curr.kind_id()) && !func_visited.contains(&curr.id()) {
                func_visited.insert(curr.id());
                let (begin, end) = insert_early_return_mechanism(curr);
                modifier.insert(begin, literals::EARLY_RETURN_BEGIN);
                modifier.insert(end, literals::EARLY_RETURN_END);
                break;
            }
        }
    }

    if !func_visited.is_empty() {
        modifier.insert(0, literals::MECHANISM);
    }

    modifier.modify(input)
}
