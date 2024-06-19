mod model;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{char, digit0, digit1, multispace0};
use nom::combinator::{map, opt, recognize};
use nom::multi::many0;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;
use xml_nom::model::QName;
use xml_nom::{ncname, qname};

// -----------------------------------------------------------------------------------------------

pub fn parse(input: &str) -> IResult<&str, model::Expr> {
    expr(input)
}

// -----------------------------------------------------------------------------------------------

/// Step | RelativeLocationPath '/' Step | RelativeLocationPath '//' Step
///
/// [\[3\] RelativeLocationPath](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-RelativeLocationPath)
///
/// [\[11\] AbbreviatedRelativeLocationPath](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-AbbreviatedRelativeLocationPath)
fn relative_location_path(input: &str) -> IResult<&str, model::RelativeLocationPath> {
    map(
        tuple((
            step,
            many0(tuple((
                delimited(multispace0, alt((tag("//"), tag("/"))), multispace0),
                step,
            ))),
        )),
        |(f, r)| {
            let r = r
                .into_iter()
                .map(|(o, v)| (model::LocationPathOperator::from(o), v))
                .collect();
            model::RelativeLocationPath::from((f, r))
        },
    )(input)
}

/// AxisSpecifier NodeTest Predicate* | '.' | '..'
///
/// [\[4\] Step](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-Step)
///
/// [\[12\] AbbreviatedStep](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-AbbreviatedStep)
fn step(input: &str) -> IResult<&str, model::Step> {
    alt((
        map(tag(".."), |_| model::Step::Parent),
        map(char('.'), |_| model::Step::Current),
        map(
            tuple((
                axis_specifier,
                preceded(multispace0, node_test),
                many0(preceded(multispace0, predicate)),
            )),
            model::Step::from,
        ),
    ))(input)
}

/// AxisName '::' | '@'?
///
/// [\[5\] AxisSpecifier](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-AxisSpecifier)
///
/// [\[13\] AbbreviatedAxisSpecifier](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-AbbreviatedAxisSpecifier)
fn axis_specifier(input: &str) -> IResult<&str, model::AxisSpecifier> {
    alt((
        map(
            terminated(axis_name, tuple((multispace0, tag("::")))),
            model::AxisSpecifier::from,
        ),
        map(opt(char('@')), |_| model::AxisSpecifier::default()),
    ))(input)
}

/// 'ancestor' | 'ancestor-or-self' | 'attribute' | 'child' | 'descendant' | 'descendant-or-self' |
/// 'following' | 'following-sibling' | 'namespace' | 'parent' | 'preceding' | 'preceding-sibling' |
/// 'self'
///
/// [\[6\] AxisName](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-AxisName)
fn axis_name(input: &str) -> IResult<&str, model::AxisName> {
    map(
        alt((
            tag("ancestor-or-self"),
            tag("ancestor"),
            tag("attribute"),
            tag("child"),
            tag("descendant-or-self"),
            tag("descendant"),
            tag("following-sibling"),
            tag("following"),
            tag("namespace"),
            tag("parent"),
            tag("preceding-sibling"),
            tag("preceding"),
            tag("self"),
        )),
        model::AxisName::from,
    )(input)
}

/// NameTest | NodeType '(' ')' | 'processing-instruction' '(' Literal ')'
///
/// [\[7\] NodeTest](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-NodeTest)
fn node_test(input: &str) -> IResult<&str, model::NodeTest> {
    alt((
        map(
            delimited(
                tuple((
                    tag("processing-instruction"),
                    multispace0,
                    char('('),
                    multispace0,
                )),
                literal,
                tuple((multispace0, char(')'))),
            ),
            model::NodeTest::from,
        ),
        map(
            terminated(
                node_type,
                tuple((multispace0, char('('), multispace0, char(')'))),
            ),
            model::NodeTest::from,
        ),
        map(name_test, model::NodeTest::from),
    ))(input)
}

/// '[' PredicateExpr ']'
///
/// [\[8\] Predicate](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-Predicate)
fn predicate(input: &str) -> IResult<&str, model::PredicateExpr> {
    delimited(
        tuple((char('['), multispace0)),
        predicate_expr,
        tuple((multispace0, char(']'))),
    )(input)
}

/// Expr
///
/// [\[9\] PredicateExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-PredicateExpr)
fn predicate_expr(input: &str) -> IResult<&str, model::PredicateExpr> {
    expr(input)
}

/// OrExpr
///
/// [[14] Expr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-Expr)
fn expr(input: &str) -> IResult<&str, model::Expr> {
    or_expr(input)
}

/// VariableReference | '(' Expr ')' | Literal | Number | FunctionCall
///
/// [\[15\] PrimaryExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-PrimaryExpr)
fn primary_expr(input: &str) -> IResult<&str, model::PrimaryExpr> {
    alt((
        map(variable_reference, model::PrimaryExpr::from),
        map(
            delimited(
                tuple((char('('), multispace0)),
                expr,
                tuple((multispace0, char(')'))),
            ),
            model::PrimaryExpr::from,
        ),
        map(literal, model::PrimaryExpr::from),
        map(number, model::PrimaryExpr::number),
        map(function_call, model::PrimaryExpr::from),
    ))(input)
}

/// FunctionName '(' ( Argument ( ',' Argument )* )? ')'
///
/// [\[16\] FunctionCall](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-FunctionCall)
fn function_call(input: &str) -> IResult<&str, model::FunctionCall> {
    map(
        tuple((
            function_name,
            delimited(
                tuple((multispace0, char('('), multispace0)),
                opt(tuple((
                    argument,
                    many0(preceded(
                        tuple((multispace0, char(','), multispace0)),
                        argument,
                    )),
                ))),
                tuple((multispace0, char(')'))),
            ),
        )),
        |(name, args)| {
            let args = args
                .map(|(f, mut r)| {
                    r.insert(0, f);
                    r
                })
                .unwrap_or_default();
            model::FunctionCall::from((name, args))
        },
    )(input)
}

/// Expr
///
/// [\[17\] Argument](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-Argument)
fn argument(input: &str) -> IResult<&str, model::Argument> {
    expr(input)
}

/// PathExpr | UnionExpr '|' PathExpr
///
/// [\[18\] UnionExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-UnionExpr)
fn union_expr(input: &str) -> IResult<&str, model::UnionExpr> {
    map(
        tuple((
            path_expr,
            many0(preceded(
                tuple((multispace0, tag("|"), multispace0)),
                path_expr,
            )),
        )),
        model::UnionExpr::from,
    )(input)
}

/// RelativeLocationPath |
/// '/' RelativeLocationPath? |
/// '//' RelativeLocationPath |
/// FilterExpr |
/// FilterExpr '/' RelativeLocationPath |
/// FilterExpr '//' RelativeLocationPath
///
/// [\[1\] LocationPath](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-LocationPath)
///
/// [\[2\] AbsoluteLocationPath](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-AbsoluteLocationPath)
///
/// [\[10\] AbbreviatedAbsoluteLocationPath](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-AbbreviatedAbsoluteLocationPath)
///
/// [\[19\] PathExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-UnionExpr)
fn path_expr(input: &str) -> IResult<&str, model::PathExpr> {
    alt((
        map(
            tuple((
                filter_expr,
                delimited(multispace0, alt((tag("//"), tag("/"))), multispace0),
                relative_location_path,
            )),
            |(filter, op, path)| {
                model::PathExpr::from((
                    Some((Some(filter), model::LocationPathOperator::from(op))),
                    path,
                ))
            },
        ),
        map(filter_expr, model::PathExpr::from),
        map(
            tuple((
                terminated(alt((tag("//"), tag("/"))), multispace0),
                relative_location_path,
            )),
            |(op, path)| {
                model::PathExpr::from((Some((None, model::LocationPathOperator::from(op))), path))
            },
        ),
        map(filter_expr, model::PathExpr::from),
        map(relative_location_path, model::PathExpr::from),
        map(char('/'), |_| model::PathExpr::Root),
    ))(input)
}

/// PrimaryExpr | FilterExpr Predicate
///
/// [\[20\] FilterExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-FilterExpr)
fn filter_expr(input: &str) -> IResult<&str, model::FilterExpr> {
    map(
        tuple((primary_expr, many0(preceded(multispace0, predicate)))),
        model::FilterExpr::from,
    )(input)
}

/// AndExpr | OrExpr 'or' AndExpr
///
/// [\[21\] OrExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-OrExpr)
fn or_expr(input: &str) -> IResult<&str, model::OrExpr> {
    map(
        tuple((
            and_expr,
            many0(preceded(
                tuple((multispace0, tag("or"), multispace0)),
                and_expr,
            )),
        )),
        model::OrExpr::from,
    )(input)
}

/// EqualityExpr | AndExpr 'and' EqualityExpr
///
/// [\[22\] AndExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-AndExpr)
fn and_expr(input: &str) -> IResult<&str, model::AndExpr> {
    map(
        tuple((
            equality_expr,
            many0(preceded(
                tuple((multispace0, tag("and"), multispace0)),
                equality_expr,
            )),
        )),
        model::AndExpr::from,
    )(input)
}

/// RelationalExpr | EqualityExpr '=' RelationalExpr | EqualityExpr '!=' RelationalExpr
///
/// [\[23\] EqualityExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-EqualityExpr)
fn equality_expr(input: &str) -> IResult<&str, model::EqualityExpr> {
    map(
        tuple((
            relation_expr,
            many0(tuple((
                delimited(multispace0, alt((tag("="), tag("!="))), multispace0),
                relation_expr,
            ))),
        )),
        |(f, r)| {
            let r = r
                .into_iter()
                .map(|(o, v)| (model::EqualityOperator::from(o), v))
                .collect();
            model::EqualityExpr::from((f, r))
        },
    )(input)
}

/// AdditiveExpr |
/// RelationalExpr '<' AdditiveExpr | RelationalExpr '>' AdditiveExpr |
/// RelationalExpr '<=' AdditiveExpr | RelationalExpr '>=' AdditiveExpr
///
/// [\[24\] RelationalExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-RelationalExpr)
fn relation_expr(input: &str) -> IResult<&str, model::RelationalExpr> {
    map(
        tuple((
            additive_expr,
            many0(tuple((
                delimited(
                    multispace0,
                    alt((tag("<="), tag(">="), tag("<"), tag(">"))),
                    multispace0,
                ),
                additive_expr,
            ))),
        )),
        |(f, r)| {
            let r = r
                .into_iter()
                .map(|(o, v)| (model::RelationalOperator::from(o), v))
                .collect();
            model::RelationalExpr::from((f, r))
        },
    )(input)
}

/// MultiplicativeExpr | AdditiveExpr '+' MultiplicativeExpr | AdditiveExpr '-' MultiplicativeExpr
///
/// [\[25\] AdditiveExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-AdditiveExpr)
fn additive_expr(input: &str) -> IResult<&str, model::AdditiveExpr> {
    map(
        tuple((
            multiplicative_expr,
            many0(tuple((
                delimited(multispace0, alt((tag("+"), tag("-"))), multispace0),
                multiplicative_expr,
            ))),
        )),
        |(f, r)| {
            let r = r
                .into_iter()
                .map(|(o, v)| (model::AdditiveOperator::from(o), v))
                .collect();
            model::AdditiveExpr::from((f, r))
        },
    )(input)
}

/// UnaryExpr |
/// MultiplicativeExpr MultiplyOperator UnaryExpr |
/// MultiplicativeExpr 'div' UnaryExpr |
/// MultiplicativeExpr 'mod' UnaryExpr
///
/// [\[26\] MultiplicativeExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-MultiplicativeExpr)
fn multiplicative_expr(input: &str) -> IResult<&str, model::MultiplicativeExpr> {
    map(
        tuple((
            unary_expr,
            many0(tuple((
                delimited(
                    multispace0,
                    alt((tag("*"), tag("div"), tag("mod"))),
                    multispace0,
                ),
                unary_expr,
            ))),
        )),
        |(f, r)| {
            let r = r
                .into_iter()
                .map(|(o, v)| (model::MultiplicativeOperator::from(o), v))
                .collect();
            model::MultiplicativeExpr::from((f, r))
        },
    )(input)
}

/// UnionExpr | '-' UnaryExpr
///
/// [\[27\] UnaryExpr](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-MultiplicativeExpr)
fn unary_expr(input: &str) -> IResult<&str, model::UnaryExpr> {
    map(
        tuple((many0(terminated(tag("-"), multispace0)), union_expr)),
        model::UnaryExpr::from,
    )(input)
}

/// '"' [^"]* '"' | "'" [^']* "'"
///
/// [\[29\] Literal](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-Literal)
fn literal(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(char('"'), take_till(|c| c == '"'), char('"')),
        delimited(char('\''), take_till(|c| c == '\''), char('\'')),
    ))(input)
}

/// [0-9]+ ('.' ([0-9]+)?)? | '.' [0-9]+
///
/// [\[30\] Number](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-Number)
///
/// [\[31\] Digits](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-Digits)
fn number(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(tuple((digit1, opt(tuple((char('.'), digit0)))))),
        recognize(tuple((char('.'), digit1))),
    ))(input)
}

/// QName - NodeType
///
/// [\[35\] FunctionName](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-FunctionName)
fn function_name(input: &str) -> IResult<&str, QName> {
    // TODO:
    qname(input)
}

/// '$' QName
///
/// [\[36\] VariableReference](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-VariableReference)
fn variable_reference(input: &str) -> IResult<&str, QName> {
    preceded(char('$'), qname)(input)
}

/// '*' | NCName ':' '*' | QName
///
/// [\[37\] NameTest](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-NameTest)
fn name_test(input: &str) -> IResult<&str, model::NameTest> {
    alt((
        map(char('*'), |_| model::NameTest::All),
        map(terminated(ncname, tag(":*")), model::NameTest::from),
        map(qname, model::NameTest::from),
    ))(input)
}

/// 'comment' | 'text' | 'processing-instruction' | 'node'
///
/// [\[38\] NodeType](https://triple-underscore.github.io/XML/xpath10-ja.html#NT-NodeType)
fn node_type(input: &str) -> IResult<&str, model::NodeType> {
    map(
        alt((
            tag("comment"),
            tag("text"),
            tag("processing-instruction"),
            tag("node"),
        )),
        model::NodeType::from,
    )(input)
}

// -----------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use xml_nom::model::{PrefixedName, QName};

    #[test]
    fn test_relative_location_path() {
        let (rest, ret) = relative_location_path(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::RelativeLocationPath::from(model::Step::Current), ret);

        let (rest, ret) = relative_location_path("./.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelativeLocationPath::from((
                model::Step::Current,
                vec![(model::LocationPathOperator::Child, model::Step::Current)]
            )),
            ret
        );

        let (rest, ret) = relative_location_path(". / .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelativeLocationPath::from((
                model::Step::Current,
                vec![(model::LocationPathOperator::Child, model::Step::Current)]
            )),
            ret
        );

        let (rest, ret) = relative_location_path(".//.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelativeLocationPath::from((
                model::Step::Current,
                vec![(
                    model::LocationPathOperator::Descendant,
                    model::Step::Current
                )]
            )),
            ret
        );

        let (rest, ret) = relative_location_path(". // .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelativeLocationPath::from((
                model::Step::Current,
                vec![(
                    model::LocationPathOperator::Descendant,
                    model::Step::Current
                )]
            )),
            ret
        );

        let (rest, ret) = relative_location_path(". / . // .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelativeLocationPath::from((
                model::Step::Current,
                vec![
                    (model::LocationPathOperator::Child, model::Step::Current),
                    (
                        model::LocationPathOperator::Descendant,
                        model::Step::Current
                    ),
                ]
            )),
            ret
        );
    }

    #[test]
    fn test_step() {
        let (rest, ret) = step("a").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Step::Test(
                model::AxisSpecifier::default(),
                model::NodeTest::from(model::NameTest::from(QName::from("a"))),
                vec![]
            ),
            ret
        );

        let (rest, ret) = step("child::a[.]").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Step::Test(
                model::AxisSpecifier::Name(model::AxisName::Child),
                model::NodeTest::from(model::NameTest::from(QName::from("a"))),
                vec![expr_current()]
            ),
            ret
        );

        let (rest, ret) = step("child :: a [.] [..]").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Step::Test(
                model::AxisSpecifier::Name(model::AxisName::Child),
                model::NodeTest::from(model::NameTest::from(QName::from("a"))),
                vec![expr_current(), expr_parent(),]
            ),
            ret
        );

        let (rest, ret) = step(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Step::Current, ret);

        let (rest, ret) = step("..").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Step::Parent, ret);
    }

    #[test]
    fn test_axis_specifier() {
        let (rest, ret) = axis_specifier("child::").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisSpecifier::Name(model::AxisName::Child), ret);

        let (rest, ret) = axis_specifier("child ::").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisSpecifier::Name(model::AxisName::Child), ret);

        let (rest, ret) = axis_specifier("@").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisSpecifier::Abbreviated, ret);

        let (rest, ret) = axis_specifier("").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisSpecifier::Abbreviated, ret);
    }

    #[test]
    fn test_axis_name() {
        let (rest, ret) = axis_name("ancestor").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::Ancestor, ret);

        let (rest, ret) = axis_name("ancestor-or-self").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::AncestorOrSelf, ret);

        let (rest, ret) = axis_name("attribute").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::Attribute, ret);

        let (rest, ret) = axis_name("child").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::Child, ret);

        let (rest, ret) = axis_name("descendant").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::Descendant, ret);

        let (rest, ret) = axis_name("descendant-or-self").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::DescendantOrSelf, ret);

        let (rest, ret) = axis_name("following").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::Following, ret);

        let (rest, ret) = axis_name("following-sibling").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::FollowingSibling, ret);

        let (rest, ret) = axis_name("namespace").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::Namespace, ret);

        let (rest, ret) = axis_name("parent").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::Parent, ret);

        let (rest, ret) = axis_name("preceding").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::Preceding, ret);

        let (rest, ret) = axis_name("preceding-sibling").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::PrecedingSibling, ret);

        let (rest, ret) = axis_name("self").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::AxisName::Current, ret);

        let _err = axis_name("unknown").err().unwrap();
    }

    #[test]
    fn test_node_test() {
        let (rest, ret) = node_test("*").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NodeTest::Name(model::NameTest::default()), ret);

        let (rest, ret) = node_test("comment()").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NodeTest::Type(model::NodeType::Comment), ret);

        let (rest, ret) = node_test("text ( )").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NodeTest::Type(model::NodeType::Text), ret);

        let (rest, ret) = node_test("processing-instruction(\"a\")").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NodeTest::PI("a"), ret);

        let (rest, ret) = node_test("processing-instruction ( \"a\" )").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NodeTest::PI("a"), ret);
    }

    #[test]
    fn test_predicate() {
        let (rest, ret) = predicate("[.]").unwrap();
        assert_eq!("", rest);
        assert_eq!(expr_current(), ret);

        let (rest, ret) = predicate("[ . ]").unwrap();
        assert_eq!("", rest);
        assert_eq!(expr_current(), ret);
    }

    #[test]
    fn test_primary_expr() {
        let (rest, ret) = primary_expr("$a").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::PrimaryExpr::from(QName::from("a")), ret);

        let (rest, ret) = primary_expr("(.)").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::PrimaryExpr::from(expr_current()), ret);

        let (rest, ret) = primary_expr("( . )").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::PrimaryExpr::from(expr_current()), ret);

        let (rest, ret) = primary_expr("'a'").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::PrimaryExpr::from("a"), ret);

        let (rest, ret) = primary_expr("1").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::PrimaryExpr::number("1"), ret);

        let (rest, ret) = primary_expr("a()").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::PrimaryExpr::from(model::FunctionCall::from(QName::from("a"))),
            ret
        );
    }

    #[test]
    fn test_function_call() {
        let (rest, ret) = function_call("a()").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::FunctionCall::from(QName::from("a")), ret);

        let (rest, ret) = function_call("a ( )").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::FunctionCall::from(QName::from("a")), ret);

        let (rest, ret) = function_call("a(.)").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::FunctionCall::from((QName::from("a"), vec![expr_current()])),
            ret
        );

        let (rest, ret) = function_call("a ( . )").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::FunctionCall::from((QName::from("a"), vec![expr_current()])),
            ret
        );

        let (rest, ret) = function_call("a(.,.)").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::FunctionCall::from((QName::from("a"), vec![expr_current(), expr_current(),])),
            ret
        );

        let (rest, ret) = function_call("a( . , . )").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::FunctionCall::from((QName::from("a"), vec![expr_current(), expr_current(),])),
            ret
        );
    }

    #[test]
    fn test_union_expr() {
        let (rest, ret) = union_expr(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(union_expr_current(), ret);

        let (rest, ret) = union_expr(".|.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::UnionExpr::from((path_expr_current(), vec![path_expr_current()])),
            ret
        );

        let (rest, ret) = union_expr(". | .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::UnionExpr::from((path_expr_current(), vec![path_expr_current()])),
            ret
        );

        let (rest, ret) = union_expr(". | . | .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::UnionExpr::from((
                path_expr_current(),
                vec![path_expr_current(), path_expr_current()]
            )),
            ret
        );
    }

    #[test]
    fn test_path_expr() {
        let (rest, ret) = path_expr("/").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::PathExpr::Root, ret);

        let (rest, ret) = path_expr("\"a\"").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::PathExpr::from(model::FilterExpr::from(model::PrimaryExpr::from("a"))),
            ret
        );

        let (rest, ret) = path_expr(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(path_expr_current(), ret);

        let (rest, ret) = path_expr("/ .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::PathExpr::from((
                Some((None, model::LocationPathOperator::Child)),
                model::RelativeLocationPath::from(model::Step::Current),
            )),
            ret
        );

        let (rest, ret) = path_expr("// .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::PathExpr::from((
                Some((None, model::LocationPathOperator::Descendant)),
                model::RelativeLocationPath::from(model::Step::Current),
            )),
            ret
        );

        let (rest, ret) = path_expr("\"a\" / .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::PathExpr::from((
                Some((
                    Some(model::FilterExpr::from(model::PrimaryExpr::from("a"))),
                    model::LocationPathOperator::Child
                )),
                model::RelativeLocationPath::from(model::Step::Current),
            )),
            ret
        );

        let (rest, ret) = path_expr("\"a\" // .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::PathExpr::from((
                Some((
                    Some(model::FilterExpr::from(model::PrimaryExpr::from("a"))),
                    model::LocationPathOperator::Descendant
                )),
                model::RelativeLocationPath::from(model::Step::Current),
            )),
            ret
        );
    }

    #[test]
    fn test_filter_expr() {
        let (rest, ret) = filter_expr("\"a\"").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::FilterExpr::from(model::PrimaryExpr::from("a")), ret);

        let (rest, ret) = filter_expr("\"a\"[.]").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::FilterExpr::from((model::PrimaryExpr::from("a"), vec![expr_current()])),
            ret
        );

        let (rest, ret) = filter_expr("\"a\"[.][.]").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::FilterExpr::from((
                model::PrimaryExpr::from("a"),
                vec![expr_current(), expr_current()]
            )),
            ret
        );
    }

    #[test]
    fn test_or_expr() {
        let (rest, ret) = or_expr(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(expr_current(), ret);

        let (rest, ret) = or_expr(".or.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::OrExpr::from((and_expr_current(), vec![and_expr_current()])),
            ret
        );

        let (rest, ret) = or_expr(". or .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::OrExpr::from((and_expr_current(), vec![and_expr_current()])),
            ret
        );

        let (rest, ret) = or_expr(". or . or .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::OrExpr::from((
                and_expr_current(),
                vec![and_expr_current(), and_expr_current()]
            )),
            ret
        );
    }

    #[test]
    fn test_and_expr() {
        let (rest, ret) = and_expr(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(and_expr_current(), ret);

        let (rest, ret) = and_expr(".and.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::AndExpr::from((equal_expr_current(), vec![equal_expr_current()])),
            ret
        );

        let (rest, ret) = and_expr(". and .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::AndExpr::from((equal_expr_current(), vec![equal_expr_current()])),
            ret
        );

        let (rest, ret) = and_expr(". and . and .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::AndExpr::from((
                equal_expr_current(),
                vec![equal_expr_current(), equal_expr_current()]
            )),
            ret
        );
    }

    #[test]
    fn test_equality_expr() {
        let (rest, ret) = equality_expr(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(equal_expr_current(), ret);

        let (rest, ret) = equality_expr(".=.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::EqualityExpr::from((
                relation_expr_current(),
                vec![(model::EqualityOperator::Equal, relation_expr_current())]
            )),
            ret
        );

        let (rest, ret) = equality_expr(". = .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::EqualityExpr::from((
                relation_expr_current(),
                vec![(model::EqualityOperator::Equal, relation_expr_current())]
            )),
            ret
        );

        let (rest, ret) = equality_expr(".!=.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::EqualityExpr::from((
                relation_expr_current(),
                vec![(model::EqualityOperator::NotEqual, relation_expr_current())]
            )),
            ret
        );

        let (rest, ret) = equality_expr(". != .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::EqualityExpr::from((
                relation_expr_current(),
                vec![(model::EqualityOperator::NotEqual, relation_expr_current())]
            )),
            ret
        );

        let (rest, ret) = equality_expr(". = . != .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::EqualityExpr::from((
                relation_expr_current(),
                vec![
                    (model::EqualityOperator::Equal, relation_expr_current()),
                    (model::EqualityOperator::NotEqual, relation_expr_current()),
                ]
            )),
            ret
        );
    }

    #[test]
    fn test_relation_expr() {
        let (rest, ret) = relation_expr(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(relation_expr_current(), ret);

        let (rest, ret) = relation_expr(".<.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelationalExpr::from((
                additive_expr_current(),
                vec![(model::RelationalOperator::LessThan, additive_expr_current())]
            )),
            ret
        );

        let (rest, ret) = relation_expr(". < .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelationalExpr::from((
                additive_expr_current(),
                vec![(model::RelationalOperator::LessThan, additive_expr_current())]
            )),
            ret
        );

        let (rest, ret) = relation_expr(".>.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelationalExpr::from((
                additive_expr_current(),
                vec![(
                    model::RelationalOperator::GreaterThan,
                    additive_expr_current()
                )]
            )),
            ret
        );

        let (rest, ret) = relation_expr(". > .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelationalExpr::from((
                additive_expr_current(),
                vec![(
                    model::RelationalOperator::GreaterThan,
                    additive_expr_current()
                )]
            )),
            ret
        );

        let (rest, ret) = relation_expr(".<=.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelationalExpr::from((
                additive_expr_current(),
                vec![(
                    model::RelationalOperator::LessEqual,
                    additive_expr_current()
                )]
            )),
            ret
        );

        let (rest, ret) = relation_expr(". <= .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelationalExpr::from((
                additive_expr_current(),
                vec![(
                    model::RelationalOperator::LessEqual,
                    additive_expr_current()
                )]
            )),
            ret
        );

        let (rest, ret) = relation_expr(".>=.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelationalExpr::from((
                additive_expr_current(),
                vec![(
                    model::RelationalOperator::GreaterEqual,
                    additive_expr_current()
                )]
            )),
            ret
        );

        let (rest, ret) = relation_expr(". >= .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelationalExpr::from((
                additive_expr_current(),
                vec![(
                    model::RelationalOperator::GreaterEqual,
                    additive_expr_current()
                )]
            )),
            ret
        );

        let (rest, ret) = relation_expr(". < . > .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::RelationalExpr::from((
                additive_expr_current(),
                vec![
                    (model::RelationalOperator::LessThan, additive_expr_current()),
                    (
                        model::RelationalOperator::GreaterThan,
                        additive_expr_current()
                    ),
                ]
            )),
            ret
        );
    }

    #[test]
    fn test_additive_expr() {
        let (rest, ret) = additive_expr(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(additive_expr_current(), ret);

        let (rest, ret) = additive_expr(".+.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::AdditiveExpr::from((
                multiplicative_expr_current(),
                vec![(model::AdditiveOperator::Add, multiplicative_expr_current())]
            )),
            ret
        );

        let (rest, ret) = additive_expr(". + .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::AdditiveExpr::from((
                multiplicative_expr_current(),
                vec![(model::AdditiveOperator::Add, multiplicative_expr_current())]
            )),
            ret
        );

        let (rest, ret) = additive_expr(".-.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::AdditiveExpr::from((
                multiplicative_expr_current(),
                vec![(model::AdditiveOperator::Sub, multiplicative_expr_current())]
            )),
            ret
        );

        let (rest, ret) = additive_expr(". - .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::AdditiveExpr::from((
                multiplicative_expr_current(),
                vec![(model::AdditiveOperator::Sub, multiplicative_expr_current())]
            )),
            ret
        );

        let (rest, ret) = additive_expr(". + . - .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::AdditiveExpr::from((
                multiplicative_expr_current(),
                vec![
                    (model::AdditiveOperator::Add, multiplicative_expr_current()),
                    (model::AdditiveOperator::Sub, multiplicative_expr_current()),
                ]
            )),
            ret
        );
    }

    #[test]
    fn test_multiplicative_expr() {
        let (rest, ret) = multiplicative_expr(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(multiplicative_expr_current(), ret);

        let (rest, ret) = multiplicative_expr(".*.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::MultiplicativeExpr::from((
                unary_expr_current(),
                vec![(model::MultiplicativeOperator::Mul, unary_expr_current())]
            )),
            ret
        );

        let (rest, ret) = multiplicative_expr(". * .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::MultiplicativeExpr::from((
                unary_expr_current(),
                vec![(model::MultiplicativeOperator::Mul, unary_expr_current())]
            )),
            ret
        );

        let (rest, ret) = multiplicative_expr(".div.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::MultiplicativeExpr::from((
                unary_expr_current(),
                vec![(model::MultiplicativeOperator::Div, unary_expr_current())]
            )),
            ret
        );

        let (rest, ret) = multiplicative_expr(". div .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::MultiplicativeExpr::from((
                unary_expr_current(),
                vec![(model::MultiplicativeOperator::Div, unary_expr_current())]
            )),
            ret
        );

        let (rest, ret) = multiplicative_expr(".mod.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::MultiplicativeExpr::from((
                unary_expr_current(),
                vec![(model::MultiplicativeOperator::Mod, unary_expr_current())]
            )),
            ret
        );

        let (rest, ret) = multiplicative_expr(". mod .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::MultiplicativeExpr::from((
                unary_expr_current(),
                vec![(model::MultiplicativeOperator::Mod, unary_expr_current())]
            )),
            ret
        );

        let (rest, ret) = multiplicative_expr(". * . div .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::MultiplicativeExpr::from((
                unary_expr_current(),
                vec![
                    (model::MultiplicativeOperator::Mul, unary_expr_current()),
                    (model::MultiplicativeOperator::Div, unary_expr_current()),
                ]
            )),
            ret
        );
    }

    #[test]
    fn test_unary_expr() {
        let (rest, ret) = unary_expr(".").unwrap();
        assert_eq!("", rest);
        assert_eq!(unary_expr_current(), ret);

        let (rest, ret) = unary_expr("-.").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::UnaryExpr::from((vec!["-"], union_expr_current())),
            ret
        );

        let (rest, ret) = unary_expr("- .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::UnaryExpr::from((vec!["-"], union_expr_current())),
            ret
        );

        let (rest, ret) = unary_expr("- - .").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::UnaryExpr::from((vec!["-", "-"], union_expr_current())),
            ret
        );
    }

    #[test]
    fn test_literal() {
        let (rest, ret) = literal("\"a\"").unwrap();
        assert_eq!("", rest);
        assert_eq!("a", ret);

        let (rest, ret) = literal("'a'").unwrap();
        assert_eq!("", rest);
        assert_eq!("a", ret);
    }

    #[test]
    fn test_number() {
        let (rest, ret) = number("1").unwrap();
        assert_eq!("", rest);
        assert_eq!("1", ret);

        let (rest, ret) = number("1.").unwrap();
        assert_eq!("", rest);
        assert_eq!("1.", ret);

        let (rest, ret) = number("1.2").unwrap();
        assert_eq!("", rest);
        assert_eq!("1.2", ret);

        let (rest, ret) = number(".1").unwrap();
        assert_eq!("", rest);
        assert_eq!(".1", ret);
    }

    #[test]
    fn test_name_test() {
        let (rest, ret) = name_test("*").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NameTest::All, ret);

        let (rest, ret) = name_test("a:*").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NameTest::Namespace("a"), ret);

        let (rest, ret) = name_test("a").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NameTest::QName(QName::from("a")), ret);

        let (rest, ret) = name_test("a:b").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::NameTest::QName(QName::from(PrefixedName::from(("a", "b")))),
            ret
        );

        let _err = name_test("+").err().unwrap();
    }

    #[test]
    fn test_node_type() {
        let (rest, ret) = node_type("comment").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NodeType::Comment, ret);

        let (rest, ret) = node_type("text").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NodeType::Text, ret);

        let (rest, ret) = node_type("processing-instruction").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NodeType::PI, ret);

        let (rest, ret) = node_type("node").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::NodeType::Node, ret);

        let _err = node_type("unknown").err().unwrap();
    }

    fn and_expr_current<'a>() -> model::AndExpr<'a> {
        model::AndExpr::from(equal_expr_current())
    }

    fn equal_expr_current<'a>() -> model::EqualityExpr<'a> {
        model::EqualityExpr::from(relation_expr_current())
    }

    fn relation_expr_current<'a>() -> model::RelationalExpr<'a> {
        model::RelationalExpr::from(additive_expr_current())
    }

    fn additive_expr_current<'a>() -> model::AdditiveExpr<'a> {
        model::AdditiveExpr::from(multiplicative_expr_current())
    }

    fn multiplicative_expr_current<'a>() -> model::MultiplicativeExpr<'a> {
        model::MultiplicativeExpr::from(unary_expr_current())
    }

    fn unary_expr_current<'a>() -> model::UnaryExpr<'a> {
        model::UnaryExpr::from(union_expr_current())
    }

    fn union_expr_current<'a>() -> model::UnionExpr<'a> {
        model::UnionExpr::from(path_expr_current())
    }

    fn path_expr_current<'a>() -> model::PathExpr<'a> {
        model::PathExpr::from(model::RelativeLocationPath::from(model::Step::Current))
    }

    fn expr_current<'a>() -> model::Expr<'a> {
        model::Expr::from(and_expr_current())
    }

    fn expr_parent<'a>() -> model::Expr<'a> {
        model::Expr::from(model::AndExpr::from(model::EqualityExpr::from(
            model::RelationalExpr::from(model::AdditiveExpr::from(
                model::MultiplicativeExpr::from(model::UnaryExpr::from(model::UnionExpr::from(
                    model::PathExpr::from(model::RelativeLocationPath::from(model::Step::Parent)),
                ))),
            )),
        )))
    }
}
