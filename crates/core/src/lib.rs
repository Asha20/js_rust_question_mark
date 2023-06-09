use std::borrow::Cow;

use replacer::QuestionMarkOperator;

mod replacer;
mod tokenizer;

pub fn process<'a>(s: &'a str, config: Config) -> Cow<'a, str> {
    let tokens = tokenizer::tokenize(s);
    let qm = QuestionMarkOperator::new(config);
    let result = replacer::process(s, &qm, tokens);
    result
}

pub use replacer::Config;

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, path::Path};

    use crate::{process, Config};

    fn check(input: &str, output: &str) {
        use dprint_plugin_typescript::configuration::ConfigurationBuilder;

        let config = Config {
            value_check: Cow::Borrowed("x.isOk"),
            unwrap: Cow::Borrowed("x.value"),
            mangle: false,
        };

        let result = process(input.trim(), config);
        let result = dprint_plugin_typescript::format_text(
            Path::new("input.ts"),
            &result,
            &ConfigurationBuilder::new().build(),
        )
        .unwrap();

        let output = dprint_plugin_typescript::format_text(
            Path::new("output.ts"),
            output,
            &ConfigurationBuilder::new().build(),
        )
        .unwrap();

        assert_eq!(result, output);
    }

    #[test]
    fn no_op() {
        check("foo;", "foo;");
    }

    #[test]
    fn single_op() {
        check(
            "foo.$",
            r#"
            const EARLY_RETURN = Symbol();
            const __unwrap = x => {
                if (x.isOk) return x.value;
                throw { [EARLY_RETURN]: x };
            };
            __unwrap(foo);
        "#,
        );
    }

    #[test]
    fn nested_expression() {
        check(
            "true ? foo.$.$ : bar.$.baz.$",
            r#"
            const EARLY_RETURN = Symbol();
            const __unwrap = x => {
                if (x.isOk) return x.value;
                throw { [EARLY_RETURN]: x };
            };
            true ? __unwrap(__unwrap(foo)) : __unwrap(__unwrap(bar).baz);
            "#,
        );
    }

    #[test]
    fn op_in_function() {
        check(
            r#"
            function foo(x) {
                return x.$;
            }
        "#,
            r#"
            const EARLY_RETURN = Symbol();
            const __unwrap = x => {
                if (x.isOk) return x.value;
                throw { [EARLY_RETURN]: x };
            };
            function foo(x) {
                try {
                    return __unwrap(x);
                } catch (e) {
                    if (EARLY_RETURN in e) return e[EARLY_RETURN];
                    throw e;
                }
            }
        "#,
        );
    }

    #[test]
    fn op_in_nested_function() {
        check(
            r#"
            function foo(x) {
                function bar(x) {
                    return x.$;
                }
                return bar(x);
            }
        "#,
            r#"
            const EARLY_RETURN = Symbol();
            const __unwrap = x => {
                if (x.isOk) return x.value;
                throw { [EARLY_RETURN]: x };
            };
            function foo(x) {
                function bar(x) {
                    try {
                        return __unwrap(x);
                    } catch (e) {
                        if (EARLY_RETURN in e) return e[EARLY_RETURN];
                        throw e;
                    }
                }
                return bar(x);
            }
        "#,
        );
    }

    #[test]
    fn arrow_function_that_returns_expr() {
        check(
            r#"
                () => a.$ + b.$
            "#,
            r#"
            const EARLY_RETURN = Symbol();
            const __unwrap = x => {
                if (x.isOk) return x.value;
                throw { [EARLY_RETURN]: x };
            };
            () => {
                try {
                    return __unwrap(a) + __unwrap(b);
                } catch (e) {
                    if (EARLY_RETURN in e) return e[EARLY_RETURN];
                    throw e;
                }
            }
            "#,
        )
    }

    #[test]
    fn inside_iife() {
        check(
            r#"
                (function main() {
                    () => a.$ + b.$;
                    () => c.$ + d.$;
                })();
            "#,
            r#"
            const EARLY_RETURN = Symbol();
            const __unwrap = x => {
                if (x.isOk) return x.value;
                throw { [EARLY_RETURN]: x };
            };
            (function main() {
                () => {
                    try {
                        return __unwrap(a) + __unwrap(b);
                    } catch (e) {
                        if (EARLY_RETURN in e) return e[EARLY_RETURN];
                        throw e;
                    }
                };
                () => {
                    try {
                        return __unwrap(c) + __unwrap(d);
                    } catch (e) {
                        if (EARLY_RETURN in e) return e[EARLY_RETURN];
                        throw e;
                    }
                };
            })();
            "#,
        )
    }
}
