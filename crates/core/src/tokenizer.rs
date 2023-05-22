use std::{collections::HashSet, ops::Range};

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

pub enum Token {
    PropertyAccess {
        object: Range<usize>,
        dot: Range<usize>,
        property: Range<usize>,
    },
    Function {
        body: Range<usize>,
    },
}

// TODO: Handle case where arrow function returns an expression, not a statement block.
//       Example: () => a.$ + b.$
fn insert_early_return_mechanism(function_decl: Node) -> (usize, usize) {
    let body = function_decl.child_by_field_id(fields::BODY).unwrap();
    let l_brace = body.child(0).unwrap();
    let r_brace = body.child(body.child_count() - 1).unwrap();

    (l_brace.end_byte(), r_brace.start_byte())
}

pub fn tokenize(input: &str) -> Vec<Token> {
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

    let mut func_visited = HashSet::new();
    let mut tokens = Vec::new();

    for match_ in cursor.matches(&query, tree.root_node(), input.as_bytes()) {
        let member_expr = match_.captures.first().unwrap().node;

        let object = member_expr.child_by_field_id(fields::OBJECT).unwrap();
        let dot = member_expr.child(1).unwrap();
        let prop = member_expr.child_by_field_id(fields::PROPERTY).unwrap();

        tokens.push(Token::PropertyAccess {
            object: object.byte_range(),
            dot: dot.byte_range(),
            property: prop.byte_range(),
        });

        let mut curr = member_expr;
        while let Some(parent) = curr.parent() {
            curr = parent;
            if kinds::is_func(curr.kind_id()) && !func_visited.contains(&curr.id()) {
                func_visited.insert(curr.id());

                let (begin, end) = insert_early_return_mechanism(curr);
                tokens.push(Token::Function { body: begin..end });
                break;
            }
        }
    }

    tokens
}
