#![recursion_limit="100"]
extern crate pest;

#[macro_use]
extern crate pest_derive;
extern crate serde;
extern crate serde_json;
#[macro_use(quick_error)] extern crate quick_error;

pub mod core;
pub mod function;
mod operator;
mod node;
mod tree;
mod expr;
mod builtin;

use std::collections::HashMap;
use serde_json::to_value as json_to_value;
use serde::Serialize;

pub use serde_json::Value;

// from baik
pub use expr::ExecOptions;
pub use function::Function;
pub use expr::Expr;
use operator::Operator;

pub fn to_value<S: Serialize>(v: S) -> Value {
    json_to_value(v).unwrap()
}

pub type Context = HashMap<String, Value>;
pub type Contexts = Vec<Context>;
pub type Functions = HashMap<String, Function>;

pub fn eval(expr: &str) -> Result<Value, Error> {
    Expr::new(expr).compile()?.exec()
}

pub type Compiled = Box<Fn(&[Context], &Functions) -> Result<Value, Error>>;

quick_error! {
    /// Expression parsing error
    #[derive(Debug, PartialEq)]
    pub enum Error {
        /// Unsupported operator yet.
        UnsupportedOperator(operator: String) {
            display("Unsupported operator: {:?}", operator)
        }
        /// This operator does not support execution.
        CanNotExec(operator: Operator) {
            display("This operator does not support execution: {:?}", operator)
        }
        /// Your expression may start with non-value operator like ( + * )
        StartWithNonValueOperator {
            display("Your expression may start with non-value operator like ( + * ).")
        }
        /// Unpaired brackets, left brackets count does not equal right brackets count
        UnpairedBrackets {
            display("Unpaired brackets, left brackets count does not equal right brackets count.")
        }
        /// Duplicate values node, you may have (2 3) but there is no operators between them
        DuplicateValueNode {
            display("Duplicate values node, you may have (2 3) but there is no operators between them.")
        }
        /// Duplicate operators node, you may have (+ +) but there is no values between them
        DuplicateOperatorNode {
            display("Duplicate operators node, you may have (+ +) but there is no values between them.")
        }
        /// You have a comma(,) , but there is no function in front of it.
        CommaNotWithFunction {
            display("You have a comma(,) , but there is no function in front of it.")
        }
        /// You have empty brackets () , but there is no function in front of it.
        BracketNotWithFunction {
            display("You have empty brackets () , but there is no function in front of it.")
        }
        /// Function not exists.
        FunctionNotExists(ident: String) {
            display("Function not exists: {}", ident)
        }
        /// Expected a boolean but the given value isn't.
        ExpectedBoolean(value: Value) {
            display("Expected a boolean, found: {}", value)
        }
        /// Expected ident.
        ExpectedIdentifier {
            display("Expected ident.")
        }
        /// Expected array.
        ExpectedArray {
            display("Expected array.")
        }
        /// Expected object.
        ExpectedObject {
            display("Expected object.")
        }
        /// Expect number.
        ExpectedNumber {
            display("Expected number.")
        }
        /// Failed to parse, no final expression.
        NoFinalNode {
            display("Failed to parse, no final expression.")
        }
        /// The number of arguments is greater than the maximum limit.
        ArgumentsGreater(max: usize) {
            display("The number of arguments is greater than the maximum limit: {}", max)
        }
        /// The number of arguments is less than the minimum limit.
        ArgumentsLess(min: usize) {
            display("The number of arguments is less than the minimum limit: {}", min)
        }
        /// This two value types are different or do not support mathematical calculations.
        UnsupportedTypes(a: String, b: String) {
            display("This two value types are different or do not support mathematical calculations: {}, {}", a, b)
        }
        /// Invalid range expression like `1..2..3`
        InvalidRange(ident: String) {
            display("Invalid range expression: {}", ident)
        }
        /// Can not add child node.
        CanNotAddChild {
            display("Can not add child node.")
        }
        /// Custom error.
        Custom(detail: String) {
            display("{}", detail)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use to_value;
    use Error;
    use Expr;
    use tree::Tree;
    use Value;
    use eval;
    use std::collections::HashMap;

    #[test]
    fn test_add() {
        assert_eq!(eval("2 + 3"), Ok(to_value(5)));
    }

    #[test]
    fn test_brackets_add() {
        assert_eq!(eval("(2 + 3) + (3 + 5)"), Ok(to_value(13)));
    }

    #[test]
    fn test_brackets_float_add() {
        assert_eq!(eval("(2 + 3.2) + (3 + 5)"), Ok(to_value(13.2)));
    }

    #[test]
    fn test_brackets_float_mul() {
        assert_eq!(eval("(2 + 3.2) * 5"), Ok(to_value(26.0)));
    }

    #[test]
    fn test_brackets_sub() {
        assert_eq!(eval("(4 - 3) * 5"), Ok(to_value(5)));
    }

    #[test]
    fn test_useless_brackets() {
        assert_eq!(eval("2 + 3 + (5)"), Ok(to_value(10)));
    }

    #[test]
    fn test_error_brackets_not_with_function() {
        assert_eq!(eval("5 + ()"), Err(Error::BracketNotWithFunction));
    }

    #[test]
    fn test_deep_brackets() {
        assert_eq!(eval("(2 + (3 + 4) + (6 + (6 + 7)) + 5)"), Ok(to_value(33)));
    }

    #[test]
    fn test_brackets_div() {
        assert_eq!(eval("(4 / (2 + 2)) * 5"), Ok(to_value(5.0)));
    }

    #[test]
    fn test_min() {
        assert_eq!(eval("min(30, 5, 245, 20)"), Ok(to_value(5)));
    }

    #[test]
    fn test_min_brackets() {
        assert_eq!(
            eval("(min(30, 5, 245, 20) * 10 + (5 + 5) * 5)"),
            Ok(to_value(100))
        );
    }

    #[test]
    fn test_min_and_mul() {
        assert_eq!(eval("min(30, 5, 245, 20) * 10"), Ok(to_value(50)));
    }

    #[test]
    fn test_max() {
        assert_eq!(eval("max(30, 5, 245, 20)"), Ok(to_value(245)));
    }

    #[test]
    fn test_max_brackets() {
        assert_eq!(
            eval("(max(30, 5, 245, 20) * 10 + (5 + 5) * 5)"),
            Ok(to_value(2500))
        );
    }

    #[test]
    fn test_max_and_mul() {
        assert_eq!(eval("max(30, 5, 245, 20) * 10"), Ok(to_value(2450)));
    }

    #[test]
    fn test_len_array() {
        assert_eq!(eval("panjang(untaian(2, 3, 4, 5, 6))"), Ok(to_value(5)));
    }

    #[test]
    fn test_null_and_number() {
        assert_eq!(eval("hos != 0"), Ok(to_value(true)));
        assert_eq!(eval("hos > 0"), Ok(to_value(false)));
    }

    #[test]
    fn test_len_string() {
        assert_eq!(eval("panjang('Halo Dunia!')"), Ok(to_value(11)));
    }

    #[test]
    fn test_len_object() {
        let mut object = HashMap::new();
        object.insert("field1", "value1");
        object.insert("field2", "value2");
        object.insert("field3", "value3");
        assert_eq!(
            Expr::new("panjang(object)").value("object", object).exec(),
            Ok(to_value(3))
        );
    }

    #[test]
    fn test_brackets_1() {
        assert_eq!(eval("(5) + (min(3, 4, 5)) + 20"), Ok(to_value(28)));
    }

    #[test]
    fn test_brackets_2() {
        assert_eq!(eval("(((5) / 5))"), Ok(to_value(1.0)));
    }

    #[test]
    fn test_string_add() {
        assert_eq!(eval(r#""Hello"+", world!""#), Ok(to_value("Hello, world!")));
    }

    #[test]
    fn test_equal() {
        assert_eq!(eval("1 == 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_not_equal() {
        assert_eq!(eval("1 != 2"), Ok(to_value(true)));
    }

    #[test]
    fn test_multiple_equal() {
        assert_eq!(eval("(1 == 2) == (2 == 3)"), Ok(to_value(true)));
    }

    #[test]
    fn test_multiple_not_equal() {
        assert_eq!(eval("(1 != 2) == (2 != 3)"), Ok(to_value(true)));
    }

    #[test]
    fn test_greater_than() {
        assert_eq!(eval("1 > 2"), Ok(to_value(false)));
        assert_eq!(eval("2 > 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_less_than() {
        assert_eq!(eval("2 < 1"), Ok(to_value(false)));
        assert_eq!(eval("1 < 2"), Ok(to_value(true)));
    }

    #[test]
    fn test_greater_and_less() {
        assert_eq!(eval("(2 > 1) == (1 < 2)"), Ok(to_value(true)));
    }

    #[test]
    fn test_ge() {
        assert_eq!(eval("2 >= 1"), Ok(to_value(true)));
        assert_eq!(eval("2 >= 2"), Ok(to_value(true)));
        assert_eq!(eval("2 >= 3"), Ok(to_value(false)));
    }

    #[test]
    fn test_le() {
        assert_eq!(eval("2 <= 1"), Ok(to_value(false)));
        assert_eq!(eval("2 <= 2"), Ok(to_value(true)));
        assert_eq!(eval("2 <= 3"), Ok(to_value(true)));
    }

    #[test]
    fn test_quotes() {
        assert_eq!(eval(r#""1><2" + "3<>4""#), Ok(to_value("1><23<>4")));
        assert_eq!(eval(r#""1==2" + "3--4""#), Ok(to_value("1==23--4")));
        assert_eq!(eval(r#""1!=2" + "3>>4""#), Ok(to_value("1!=23>>4")));
        assert_eq!(eval(r#""><1!=2" + "3>>4""#), Ok(to_value("><1!=23>>4")));
    }

    #[test]
    fn test_single_quote() {
        assert_eq!(eval(r#"'1><2' + '3<>4'"#), Ok(to_value("1><23<>4")));
        assert_eq!(eval(r#"'1==2' + '3--4'"#), Ok(to_value("1==23--4")));
        assert_eq!(eval(r#"'1!=2' + '3>>4'"#), Ok(to_value("1!=23>>4")));
        assert_eq!(eval(r#"'!=1<>2' + '3>>4'"#), Ok(to_value("!=1<>23>>4")));
    }

    #[test]
    fn test_single_and_double_quote() {
        assert_eq!(
            eval(r#"' """" ' + ' """" '"#),
            Ok(to_value(r#" """"  """" "#))
        );
    }

    #[test]
    fn test_double_and_single_quote() {
        assert_eq!(
            eval(r#"" '''' " + " '''' ""#),
            Ok(to_value(r#" ''''  '''' "#))
        );
    }

    #[test]
    fn test_array() {
        assert_eq!(eval("untaian(1, 2, 3, 4)"), Ok(to_value(vec![1, 2, 3, 4])));
    }

    #[test]
    fn test_range() {
        assert_eq!(eval("0..5"), Ok(to_value(vec![0, 1, 2, 3, 4])));
    }

    #[test]
    fn test_range_and_min() {
        assert_eq!(eval("min(0..5)"), Ok(to_value(0)));
    }

    #[test]
    fn test_rem_1() {
        assert_eq!(eval("2 % 2"), Ok(to_value(0)));
    }

    #[test]
    fn test_rem_2() {
        assert_eq!(eval("5 % 56 % 5"), Ok(to_value(0)));
    }

    #[test]
    fn test_rem_3() {
        assert_eq!(eval("5.5 % 23"), Ok(to_value(5.5)));
    }

    #[test]
    fn test_rem_4() {
        assert_eq!(eval("23 % 5.5"), Ok(to_value(1.0)));
    }

    #[test]
    fn test_and_1() {
        assert_eq!(eval("3 > 2 && 2 > 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_and_2() {
        assert_eq!(eval("3 == 2 && 2 == 1"), Ok(to_value(false)));
    }

    #[test]
    fn test_and_3() {
        assert_eq!(eval("3 > 2 && 2 == 1"), Ok(to_value(false)));
    }

    #[test]
    fn test_or_1() {
        assert_eq!(eval("3 > 2 || 2 > 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_or_2() {
        assert_eq!(eval("3 < 2 || 2 < 1"), Ok(to_value(false)));
    }

    #[test]
    fn test_or_3() {
        assert_eq!(eval("3 > 2 || 2 < 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_or_4() {
        assert_eq!(eval("3 < 2 || 2 > 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_not() {
        assert_eq!(eval("!false"), Ok(to_value(true)));
        assert_eq!(eval("!true"), Ok(to_value(false)));
        assert_eq!(eval("!(1 != 2)"), Ok(to_value(false)));
        assert_eq!(eval("!(1 == 2)"), Ok(to_value(true)));
        assert_eq!(eval("!(1 == 2) == true"), Ok(to_value(true)));
    }

    #[test]
    fn test_not_and_brackets() {
        assert_eq!(eval("(!(1 == 2)) == true"), Ok(to_value(true)));
    }

    #[test]
    fn test_object_access() {
        let mut object = HashMap::new();
        object.insert("foo", "Foo, hello world!");
        object.insert("bar", "Bar, hello world!");
        assert_eq!(
            Expr::new("object.foo == 'Foo, hello world!'")
                .value("object", object)
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_object_dynamic_access() {
        let mut object = HashMap::new();
        object.insert("foo", "Foo, hello world!");
        object.insert("bar", "Bar, hello world!");
        assert_eq!(
            Expr::new("object['foo'] == 'Foo, hello world!'")
                .value("object", object)
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_object_dynamic_access_2() {
        let mut object = HashMap::new();
        object.insert("foo", "Foo, hello world!");
        object.insert("bar", "Bar, hello world!");
        assert_eq!(
            Expr::new("object[foo] == 'Foo, hello world!'")
                .value("object", object)
                .value("foo", "foo")
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_path() {
        assert_eq!(Expr::new("untaian[2-2].foo[2-2]").exec(), Ok(Value::Null));
    }

    #[test]
    fn test_array_access() {
        let array = vec!["hello", "world", "!"];
        assert_eq!(
            Expr::new(
                "untaian[1-1] == 'hello' && untaian[1] == 'world' && untaian[2] == '!'",
            ).value("untaian", array)
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_builtin_is_empty() {
        assert_eq!(
            Expr::new("kosong(untaian)")
                .value("untaian", Vec::<String>::new())
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_builtin_min() {
        assert_eq!(
            Expr::new("min(untaian)")
                .value("untaian", vec![23, 34, 45, 2])
                .exec(),
            Ok(to_value(2))
        );
    }

    #[test]
    fn test_custom_function() {
        assert_eq!(
            Expr::new("output()")
                .function(
                    "output",
                    |_| Ok(to_value("This is custom function's output")),
                )
                .exec(),
            Ok(to_value("This is custom function's output"))
        );
    }

    #[test]
    fn test_error_start_with_non_value_operator() {
        let mut tree = Tree {
            raw: "+ + 5".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();
        tree.parse_operators().unwrap();

        assert_eq!(tree.parse_node(), Err(Error::StartWithNonValueOperator));
    }

    #[test]
    fn test_error_duplicate_operator() {
        let mut tree = Tree {
            raw: "5 + + 5".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();
        tree.parse_operators().unwrap();

        assert_eq!(tree.parse_node(), Err(Error::DuplicateOperatorNode));
    }

    #[test]
    fn test_error_duplicate_value() {
        let mut tree = Tree {
            raw: "2 + 6 5".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();
        tree.parse_operators().unwrap();

        assert_eq!(tree.parse_node(), Err(Error::DuplicateValueNode));
    }

    #[test]
    fn test_error_unpaired_brackets() {
        let mut tree = Tree {
            raw: "(2 + 3)) * 5".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();

        assert_eq!(tree.parse_operators(), Err(Error::UnpairedBrackets));
    }

    #[test]
    fn test_error_comma() {
        let mut tree = Tree {
            raw: ", 2 + 5".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();
        tree.parse_operators().unwrap();

        assert_eq!(tree.parse_node(), Err(Error::CommaNotWithFunction));
    }

    #[test]
    fn test_eval_issue_2() {
        assert_eq!(eval("2 * (4 + 0) + 4"), Ok(to_value(12)));
        assert_eq!(eval("2 * (2 + 2) + (1 + 3)"), Ok(to_value(12)));
        assert_eq!(eval("2 * (4) + (4)"), Ok(to_value(12)));
    }
}

#[cfg(all(feature = "unstable", test))]
mod benches {
    extern crate test;
    use eval;
    use tree::Tree;
    use Expr;

    #[bench]
    fn bench_deep_brackets(b: &mut test::Bencher) {
        b.iter(|| eval("(2 + (3 + 4) + (6 + (6 + 7)) + 5)"));
    }

    #[bench]
    fn bench_parse_pos(b: &mut test::Bencher) {
        let mut tree = Tree {
            raw: "(2 + (3 + 4) + (6 + (6 + 7)) + 5)".to_owned(),
            ..Default::default()
        };

        b.iter(|| tree.parse_pos().unwrap());
    }

    #[bench]
    fn bench_parse_operators(b: &mut test::Bencher) {
        let mut tree = Tree {
            raw: "(2 + (3 + 4) + (6 + (6 + 7)) + 5)".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();
        b.iter(|| tree.parse_operators().unwrap());
    }

    #[bench]
    fn bench_parse_nodes(b: &mut test::Bencher) {
        let mut tree = Tree {
            raw: "(2 + (3 + 4) + (6 + (6 + 7)) + 5)".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();
        tree.parse_operators().unwrap();
        b.iter(|| tree.parse_node().unwrap());
    }

    #[bench]
    fn bench_compile(b: &mut test::Bencher) {
        b.iter(|| {
            let mut tree = Tree {
                raw: "(2 + (3 + 4) + (6 + (6 + 7)) + 5)".to_owned(),
                ..Default::default()
            };
            tree.parse_pos().unwrap();
            tree.parse_operators().unwrap();
            tree.parse_node().unwrap();
            tree.compile().unwrap();
        });
    }

    #[bench]
    fn bench_exec(b: &mut test::Bencher) {
        let expr = Expr::new("(2 + (3 + 4) + (6 + (6 + 7)) + 5)")
            .compile()
            .unwrap();
        b.iter(|| expr.exec().unwrap())
    }

    #[bench]
    fn bench_eval(b: &mut test::Bencher) {
        b.iter(|| eval("(2 + (3 + 4) + (6 + (6 + 7)) + 5)"));
    }
}