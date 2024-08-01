pub mod error;
pub mod func;
pub mod model;

use super::expr::model as expr;
use model::AsValue;
use std::collections::HashSet;
use xml_dom::{self as dom, AsNode, AsStringValue, Node};
use xml_nom::{self as nom};

pub fn document(
    expr: &expr::Expr,
    document: dom::XmlDocument,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    eval_expr(expr, document.as_node(), context)
}

// -----------------------------------------------------------------------------------------------

fn eval_expr(
    expr: &expr::Expr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    eval_or_expr(expr, node, context)
}

fn eval_or_expr(
    or: &expr::OrExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let first = or.operands().first().unwrap();
    let mut op1 = eval_and_expr(first, node.clone(), context)?;

    for and in or.operands().iter().skip(1) {
        if bool::try_from(&op1)? {
            return Ok(true.as_value());
        }

        let op2 = bool::try_from(&eval_and_expr(and, node.clone(), context)?)?;
        op1 = op2.as_value();
    }

    Ok(op1)
}

fn eval_and_expr(
    and: &expr::AndExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let first = and.operands().first().unwrap();
    let mut op1 = eval_eq_expr(first, node.clone(), context)?;

    for eq in and.operands().iter().skip(1) {
        if !bool::try_from(&op1)? {
            return Ok(false.as_value());
        }

        let op2 = bool::try_from(&eval_eq_expr(eq, node.clone(), context)?)?;
        op1 = op2.as_value();
    }

    Ok(op1)
}

fn eval_eq_expr(
    eq: &expr::EqualityExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let mut op1 = eval_relational_expr(eq.operand(), node.clone(), context)?;
    for (op, op2) in eq.operations() {
        let op2 = eval_relational_expr(op2, node.clone(), context)?;
        let ret = match op {
            expr::EqualityOperator::Equal => equal_value(&op1, &op2),
            expr::EqualityOperator::NotEqual => not_equal_value(&op1, &op2),
        }?;

        op1 = ret.as_value();
    }
    Ok(op1)
}

fn eval_relational_expr(
    rel: &expr::RelationalExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let mut op1 = eval_add_expr(rel.operand(), node.clone(), context)?;
    for (op, op2) in rel.operations() {
        let op2 = eval_add_expr(op2, node.clone(), context)?;
        let ret = match op {
            expr::RelationalOperator::GreaterEqual => greater_eq_value(&op1, &op2),
            expr::RelationalOperator::GreaterThan => greater_than_value(&op1, &op2),
            expr::RelationalOperator::LessEqual => less_eq_value(&op1, &op2),
            expr::RelationalOperator::LessThan => less_than_value(&op1, &op2),
        }?;

        op1 = ret.as_value();
    }
    Ok(op1)
}

fn eval_add_expr(
    add: &expr::AdditiveExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let mut op1 = eval_mul_expr(add.operand(), node.clone(), context)?;
    for (op, op2) in add.operations() {
        let op2 = eval_mul_expr(op2, node.clone(), context)?;
        op1 = match op {
            expr::AdditiveOperator::Add => op1 + op2,
            expr::AdditiveOperator::Sub => op1 - op2,
        };
    }
    Ok(op1)
}

fn eval_mul_expr(
    mul: &expr::MultiplicativeExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let mut op1 = eval_unary_expr(mul.operand(), node.clone(), context)?;
    for (op, op2) in mul.operations() {
        let op2 = eval_unary_expr(op2, node.clone(), context)?;
        op1 = match op {
            expr::MultiplicativeOperator::Mul => op1 * op2,
            expr::MultiplicativeOperator::Div => op1 / op2,
            expr::MultiplicativeOperator::Mod => op1 % op2,
        };
    }
    Ok(op1)
}

fn eval_unary_expr(
    uni: &expr::UnaryExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let value = eval_union_expr(uni.value(), node.clone(), context)?;
    let inv = uni.inv().len() % 2;
    if inv == 0 {
        Ok(value)
    } else {
        Ok(-value)
    }
}

fn eval_union_expr(
    uni: &expr::UnionExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let mut nodes = vec![];

    let mut value = if let Some(first) = uni.operands().first() {
        eval_path_expr(first, node.clone(), context)?
    } else {
        return Ok(nodes.as_value());
    };

    let rest = uni.operands().iter().skip(1);
    if rest.len() == 0 {
        if let model::Value::Node(mut nodes) = value {
            let mut set = HashSet::new();
            nodes.retain(|v| set.insert(v.order()));

            return Ok(nodes.as_value());
        } else {
            return Ok(value);
        }
    } else if let model::Value::Node(mut n) = value {
        nodes.append(&mut n);
    } else {
        return Err(error::Error::InvalidType);
    }

    for op in rest {
        value = eval_path_expr(op, node.clone(), context)?;

        if let model::Value::Node(mut n) = value {
            nodes.append(&mut n);
        } else {
            return Err(error::Error::InvalidType);
        };
    }

    let mut set = HashSet::new();
    nodes.retain(|v| set.insert(v.order()));

    Ok(nodes.as_value())
}

fn eval_path_expr(
    path: &expr::PathExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let nodes = match path {
        expr::PathExpr::Filter(filter) => eval_filter_expr(filter, node.clone(), context)?,
        expr::PathExpr::Path(filter, location) => {
            eval_filtered_loc_expr(filter, location, node.clone(), context)?.as_value()
        }
        expr::PathExpr::Root => match node {
            dom::XmlNode::Document(_) => vec![node].as_value(),
            _ => vec![node.owner_document().unwrap().as_node()].as_value(),
        },
    };

    Ok(nodes)
}

fn eval_filter_expr(
    filter: &expr::FilterExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let value = eval_primary_expr(filter.primary(), node.clone(), context)?;
    if filter.predicates().is_empty() {
        return Ok(value);
    }

    let mut nodes = if let model::Value::Node(n) = value {
        n
    } else {
        return Err(error::Error::InvalidType);
    };

    for predicate in filter.predicates() {
        context.push_size(nodes.len());
        let mut filtered = vec![];
        for (position, n) in nodes.into_iter().enumerate() {
            context.push_position(position + 1);
            if eval_predicate(predicate, n.clone(), context)? {
                filtered.push(n);
            }
            context.pop_position();
        }
        nodes = filtered;
        context.pop_size();
    }

    Ok(nodes.as_value())
}

fn eval_primary_expr(
    primary: &expr::PrimaryExpr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    match primary {
        expr::PrimaryExpr::Expr(expr) => eval_expr(expr, node, context),
        expr::PrimaryExpr::Function(func) => eval_func_expr(func, node, context),
        expr::PrimaryExpr::Literal(literal) => Ok(literal.to_string().as_value()),
        expr::PrimaryExpr::Number(number) => Ok(number.parse::<f64>().unwrap().as_value()),
        expr::PrimaryExpr::Variable(_) => unimplemented!("Not support `VariableReference`."),
    }
}

fn eval_filtered_loc_expr(
    filter: &Option<(Option<expr::FilterExpr>, expr::LocationPathOperator)>,
    location: &expr::RelativeLocationPath,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<Vec<dom::XmlNode>> {
    let nodes = if let Some((filter, op)) = filter {
        if let Some(filter) = filter {
            let value = eval_filter_expr(filter, node.clone(), context)?;
            let nodes = if let model::Value::Node(n) = value {
                n
            } else {
                return Err(error::Error::InvalidType);
            };
            // FIXME:
            match op {
                expr::LocationPathOperator::Current => {
                    nodes.iter().flat_map(|n| child(n.clone())).collect()
                }
                expr::LocationPathOperator::DescendantOrSelfNode => {
                    nodes.iter().flat_map(|n| descendant(n.clone())).collect()
                }
            }
        } else {
            let root = match node {
                dom::XmlNode::Document(_) => node,
                _ => node.owner_document().unwrap().as_node(),
            };
            match op {
                expr::LocationPathOperator::Current => vec![root],
                expr::LocationPathOperator::DescendantOrSelfNode => descendant_and_self(root),
            }
        }
    } else {
        vec![node]
    };

    let mut collected = vec![];
    for n in nodes {
        collected.append(&mut eval_loc_expr(location, n.clone(), context)?);
    }

    collected.sort_by_cached_key(|v| v.order());

    Ok(collected)
}

fn eval_loc_expr(
    location: &expr::RelativeLocationPath,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<Vec<dom::XmlNode>> {
    let mut nodes = eval_step_expr(location.operand(), node, context)?;

    if !location.operations().is_empty() {
        for (op, oprand) in location.operations() {
            let mut collected = vec![];
            match op {
                expr::LocationPathOperator::Current => {
                    for n in nodes {
                        collected.append(&mut eval_step_expr(oprand, n.clone(), context)?);
                    }
                }
                expr::LocationPathOperator::DescendantOrSelfNode => {
                    for n in nodes.iter().flat_map(|n| descendant_and_self(n.clone())) {
                        collected.append(&mut eval_step_expr(oprand, n.clone(), context)?);
                    }
                }
            }
            //collected.dedup();
            nodes = collected;
        }
    }

    Ok(nodes)
}

fn eval_step_expr(
    step: &expr::Step,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<Vec<dom::XmlNode>> {
    match step {
        expr::Step::Current => Ok(vec![node]),
        expr::Step::Parent => match node {
            dom::XmlNode::Document(_) => Ok(vec![]),
            _ => Ok(vec![node.parent_node().unwrap()]),
        },
        expr::Step::Test(axis, test, predicate) => {
            eval_axis_node_test(axis, test, predicate, node, context)
        }
    }
}

fn eval_axis_node_test(
    axis: &expr::AxisSpecifier,
    test: &expr::NodeTest,
    predicates: &[expr::Expr],
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<Vec<dom::XmlNode>> {
    let mut nodes = match axis {
        expr::AxisSpecifier::Abbreviated(v) => match v.as_str() {
            "@" => attributes(node),
            _ => child(node),
        },
        expr::AxisSpecifier::Name(specifier) => match specifier {
            expr::AxisName::Ancestor => ancestor(node),
            expr::AxisName::AncestorOrSelf => ancestor_and_self(node),
            expr::AxisName::Attribute => attributes(node),
            expr::AxisName::Child => child(node),
            expr::AxisName::Descendant => descendant(node),
            expr::AxisName::DescendantOrSelf => descendant_and_self(node),
            expr::AxisName::Following => following(node),
            expr::AxisName::FollowingSibling => following_sibling(node),
            expr::AxisName::Namespace => namespace(node),
            expr::AxisName::Parent => vec![node.parent_node().unwrap()],
            expr::AxisName::Preceding => preceding(node),
            expr::AxisName::PrecedingSibling => preceding_sibling(node),
            expr::AxisName::Current => vec![node],
        },
    };

    nodes.retain(|n| eval_node_test(test, n.clone(), context));

    match axis {
        expr::AxisSpecifier::Abbreviated(_) => {
            nodes.sort_by_cached_key(|v| v.order());
        }
        expr::AxisSpecifier::Name(specifier) => match specifier {
            expr::AxisName::Ancestor
            | expr::AxisName::AncestorOrSelf
            | expr::AxisName::Preceding
            | expr::AxisName::PrecedingSibling => {
                nodes.sort_by_cached_key(|v| -v.order());
            }
            _ => {
                nodes.sort_by_cached_key(|v| v.order());
            }
        },
    }

    for predicate in predicates {
        context.push_size(nodes.len());
        let mut filtered = vec![];
        for (position, n) in nodes.into_iter().enumerate() {
            context.push_position(position + 1);
            if eval_predicate(predicate, n.clone(), context)? {
                filtered.push(n);
            }
            context.pop_position();
        }
        nodes = filtered;
        context.pop_size();
    }

    Ok(nodes)
}

fn eval_node_test(test: &expr::NodeTest, node: dom::XmlNode, _: &mut model::Context) -> bool {
    match test {
        expr::NodeTest::Name(name) => match name {
            expr::NameTest::All => true,
            expr::NameTest::Namespace(_) => unimplemented!("Not support `Namespace`."),
            expr::NameTest::QName(qname) => match qname {
                nom::model::QName::Prefixed(p) => node.node_name() == p.local_part, // FIXME: namespace
                nom::model::QName::Unprefixed(p) => node.node_name() == *p,
            },
        },
        expr::NodeTest::PI(_) => unimplemented!("Not support `processing-instruction`."),
        expr::NodeTest::Type(ty) => match ty {
            expr::NodeType::Comment => node.node_type() == dom::NodeType::Comment,
            expr::NodeType::Node => true,
            expr::NodeType::PI => node.node_type() == dom::NodeType::PI,
            expr::NodeType::Text => node.node_type() == dom::NodeType::Text,
        },
    }
}

fn eval_predicate(
    predicate: &expr::Expr,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<bool> {
    let value = eval_expr(predicate, node, context)?;
    match value {
        model::Value::Number(v) => Ok(v as usize == context.get_position()),
        _ => Ok(bool::try_from(&value)?),
    }
}

fn eval_func_expr(
    func: &expr::FunctionCall,
    node: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    let name = match func.name() {
        nom::model::QName::Prefixed(p) => p.local_part, // FIXME: namespace
        nom::model::QName::Unprefixed(u) => u,
    };

    let table = func::table();
    let entry = table
        .iter()
        .find(|v| v.name() == name)
        .ok_or_else(|| error::Error::NotFoundFunction(name.to_string()))?;

    if func.args().len() < entry.min_args() || entry.max_args() < func.args().len() {
        return Err(error::Error::InvalidArgumentCount(name.to_string()));
    }

    let mut args = vec![];
    for i in func.args() {
        args.push(eval_expr(i, node.clone(), context)?)
    }

    entry.exec(args, node.clone(), context)
}

// -----------------------------------------------------------------------------------------------

fn ancestor(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![];

    let mut parent = node.parent_node();
    while let Some(p) = parent {
        nodes.push(p.clone());
        parent = p.parent_node();
    }

    nodes
}

fn ancestor_and_self(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![node.clone()];
    nodes.append(&mut ancestor(node));
    nodes
}

fn attributes(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![];

    if let Some(attrs) = node.attributes() {
        for attr in attrs.iter() {
            nodes.push(attr);
        }
    }

    nodes
}

fn child(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![];

    for c in node.child_nodes().iter() {
        nodes.push(c.clone());
    }

    nodes
}

fn descendant(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![];

    for child in node.child_nodes().iter() {
        nodes.push(child.clone());

        let mut desc = descendant(child);
        nodes.append(&mut desc);
    }

    nodes
}

fn descendant_and_self(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![node.clone()];
    nodes.append(&mut descendant(node));
    nodes
}

fn following(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![];

    for n in following_sibling(node) {
        nodes.append(&mut descendant_and_self(n));
    }

    nodes
}

fn following_sibling(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![];

    let mut next = node.next_sibling();
    while let Some(n) = next {
        nodes.push(n.clone());
        next = n.next_sibling();
    }

    nodes
}

fn namespace(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![];

    if let dom::XmlNode::Element(element) = node {
        // FIXME:
        for ns in element.in_scope_namespace().unwrap() {
            nodes.push(ns.as_node());
        }
    }

    nodes
}

fn preceding(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![];

    for p in preceding_sibling(node) {
        let mut desc = descendant_and_self(p);
        desc.reverse();
        nodes.append(&mut desc);
    }

    nodes
}

fn preceding_sibling(node: dom::XmlNode) -> Vec<dom::XmlNode> {
    let mut nodes = vec![];

    let mut prev = node.previous_sibling();
    while let Some(p) = prev {
        nodes.push(p.clone());
        prev = p.previous_sibling();
    }

    nodes
}

// -----------------------------------------------------------------------------------------------

fn equal_value(a: &model::Value, b: &model::Value) -> error::Result<bool> {
    if a.is_node() || b.is_node() {
        let (node, value) = if a.is_node() { (a, b) } else { (b, a) };
        let nodes = if let model::Value::Node(n) = node {
            n
        } else {
            unreachable!()
        };
        equal_node(value, nodes)
    } else if a.is_bool() || b.is_bool() {
        Ok(a == &bool::try_from(b)?)
    } else if a.is_number() || b.is_number() {
        Ok(a == &f64::try_from(b)?)
    } else {
        Ok(a == &String::try_from(b)?)
    }
}

fn equal_node(a: &model::Value, b: &[dom::XmlNode]) -> error::Result<bool> {
    match a {
        model::Value::Boolean(a) => Ok(*a != b.is_empty()),
        model::Value::Node(values) => {
            for i in b {
                for j in values {
                    if equal_value(
                        &j.as_string_value()?.as_value(),
                        &i.as_string_value()?.as_value(),
                    )? {
                        return Ok(true);
                    }
                }
            }

            Ok(false)
        }
        model::Value::Number(a) => equal_node_number(a, b),
        model::Value::Text(a) => equal_node_text(a, b),
    }
}

fn equal_node_number(a: &f64, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() == *a {
            return Ok(true);
        }
    }

    Ok(false)
}

fn equal_node_text(a: &String, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() == *a {
            return Ok(true);
        }
    }

    Ok(false)
}

// -----------------------------------------------------------------------------------------------

fn not_equal_value(a: &model::Value, b: &model::Value) -> error::Result<bool> {
    if a.is_node() || b.is_node() {
        let (node, value) = if a.is_node() { (a, b) } else { (b, a) };
        let nodes = if let model::Value::Node(n) = node {
            n
        } else {
            unreachable!()
        };
        not_equal_node(value, nodes)
    } else if a.is_bool() || b.is_bool() {
        Ok(a != &bool::try_from(b)?)
    } else if a.is_number() || b.is_number() {
        Ok(a != &f64::try_from(b)?)
    } else {
        Ok(a != &String::try_from(b)?)
    }
}

fn not_equal_node(a: &model::Value, b: &[dom::XmlNode]) -> error::Result<bool> {
    match a {
        model::Value::Boolean(a) => Ok(*a == b.is_empty()),
        model::Value::Node(values) => {
            for i in b {
                for j in values {
                    if not_equal_value(
                        &j.as_string_value()?.as_value(),
                        &i.as_string_value()?.as_value(),
                    )? {
                        return Ok(true);
                    }
                }
            }

            Ok(false)
        }
        model::Value::Number(a) => not_equal_node_number(a, b),
        model::Value::Text(a) => not_equal_node_text(a, b),
    }
}

fn not_equal_node_number(a: &f64, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() != *a {
            return Ok(true);
        }
    }

    Ok(false)
}

fn not_equal_node_text(a: &String, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() != *a {
            return Ok(true);
        }
    }

    Ok(false)
}

// -----------------------------------------------------------------------------------------------

fn greater_eq_value(a: &model::Value, b: &model::Value) -> error::Result<bool> {
    match b {
        model::Value::Node(nodes) => greater_eq_node(a, nodes),
        _ => match a {
            model::Value::Node(nodes) => less_eq_node(b, nodes),
            _ => Ok(f64::try_from(a)? >= f64::try_from(b)?),
        },
    }
}

fn greater_eq_node(a: &model::Value, b: &[dom::XmlNode]) -> error::Result<bool> {
    match a {
        model::Value::Boolean(a) => Ok(*a >= !b.is_empty()),
        model::Value::Node(values) => {
            for i in b {
                for j in values {
                    if greater_eq_value(
                        &j.as_string_value()?.as_value(),
                        &i.as_string_value()?.as_value(),
                    )? {
                        return Ok(true);
                    }
                }
            }

            Ok(false)
        }
        model::Value::Number(a) => greater_eq_node_number(a, b),
        model::Value::Text(a) => greater_eq_node_text(a, b),
    }
}

fn greater_eq_node_number(a: &f64, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() <= *a {
            return Ok(true);
        }
    }

    Ok(false)
}

fn greater_eq_node_text(a: &String, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() <= *a {
            return Ok(true);
        }
    }

    Ok(false)
}

// -----------------------------------------------------------------------------------------------

fn greater_than_value(a: &model::Value, b: &model::Value) -> error::Result<bool> {
    match b {
        model::Value::Node(nodes) => greater_than_node(a, nodes),
        _ => match a {
            model::Value::Node(nodes) => less_than_node(b, nodes),
            _ => Ok(f64::try_from(a)? > f64::try_from(b)?),
        },
    }
}

fn greater_than_node(a: &model::Value, b: &[dom::XmlNode]) -> error::Result<bool> {
    match a {
        model::Value::Boolean(a) => Ok(*a & b.is_empty()),
        model::Value::Node(values) => {
            for i in b {
                for j in values {
                    if greater_than_value(
                        &j.as_string_value()?.as_value(),
                        &i.as_string_value()?.as_value(),
                    )? {
                        return Ok(true);
                    }
                }
            }

            Ok(false)
        }
        model::Value::Number(a) => greater_than_node_number(a, b),
        model::Value::Text(a) => greater_than_node_text(a, b),
    }
}

fn greater_than_node_number(a: &f64, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() < *a {
            return Ok(true);
        }
    }

    Ok(false)
}

fn greater_than_node_text(a: &String, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() < *a {
            return Ok(true);
        }
    }

    Ok(false)
}

// -----------------------------------------------------------------------------------------------

fn less_eq_value(a: &model::Value, b: &model::Value) -> error::Result<bool> {
    match b {
        model::Value::Node(nodes) => less_eq_node(a, nodes),
        _ => match a {
            model::Value::Node(nodes) => greater_eq_node(b, nodes),
            _ => Ok(f64::try_from(a)? <= f64::try_from(b)?),
        },
    }
}

fn less_eq_node(a: &model::Value, b: &[dom::XmlNode]) -> error::Result<bool> {
    match a {
        model::Value::Boolean(a) => Ok(*a <= !b.is_empty()),
        model::Value::Node(values) => {
            for i in b {
                for j in values {
                    if less_eq_value(
                        &j.as_string_value()?.as_value(),
                        &i.as_string_value()?.as_value(),
                    )? {
                        return Ok(true);
                    }
                }
            }

            Ok(false)
        }
        model::Value::Number(a) => less_eq_node_number(a, b),
        model::Value::Text(a) => less_eq_node_text(a, b),
    }
}

fn less_eq_node_number(a: &f64, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() >= *a {
            return Ok(true);
        }
    }

    Ok(false)
}

fn less_eq_node_text(a: &String, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() >= *a {
            return Ok(true);
        }
    }

    Ok(false)
}

// -----------------------------------------------------------------------------------------------

fn less_than_value(a: &model::Value, b: &model::Value) -> error::Result<bool> {
    match b {
        model::Value::Node(nodes) => less_than_node(a, nodes),
        _ => match a {
            model::Value::Node(nodes) => greater_than_node(b, nodes),
            _ => Ok(f64::try_from(a)? < f64::try_from(b)?),
        },
    }
}

fn less_than_node(a: &model::Value, b: &[dom::XmlNode]) -> error::Result<bool> {
    match a {
        model::Value::Boolean(a) => Ok(!(*a) & !b.is_empty()),
        model::Value::Node(values) => {
            for i in b {
                for j in values {
                    if less_than_value(
                        &j.as_string_value()?.as_value(),
                        &i.as_string_value()?.as_value(),
                    )? {
                        return Ok(true);
                    }
                }
            }

            Ok(false)
        }
        model::Value::Number(a) => less_than_node_number(a, b),
        model::Value::Text(a) => less_than_node_text(a, b),
    }
}

fn less_than_node_number(a: &f64, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() > *a {
            return Ok(true);
        }
    }

    Ok(false)
}

fn less_than_node_text(a: &String, b: &[dom::XmlNode]) -> error::Result<bool> {
    for i in b {
        if i.as_string_value()?.as_value() > *a {
            return Ok(true);
        }
    }

    Ok(false)
}

// -----------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::parse;
    use xml_dom::Document;

    #[test]
    fn test_absolute_location_path_root() {
        let (rest, expr) = parse("/").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(doc.as_node(), nodes[0]);
    }

    #[test]
    fn test_absolute_location_path_root_relative() {
        let (rest, expr) = parse("/root").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let root = doc
            .get_elements_by_tag_name("root")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(root, nodes[0]);
    }

    #[test]
    fn test_absolute_location_path_abbr_absolute() {
        let (rest, expr) = parse("//root").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let root = doc
            .get_elements_by_tag_name("root")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(root, nodes[0]);
    }

    #[test]
    fn test_relative_location_path_step() {
        let (rest, expr) = parse("root").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let root = doc
            .get_elements_by_tag_name("root")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(root, nodes[0]);
    }

    #[test]
    fn test_relative_location_path_step_step() {
        let (rest, expr) = parse("root/e2").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(e2, nodes[0]);
    }

    #[test]
    fn test_relative_location_path_step_abbr_relative() {
        let (rest, expr) = parse("root//ee2").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let ee2 = doc
            .get_elements_by_tag_name("ee2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(ee2, nodes[0]);
    }

    #[test]
    fn test_step_axis_ancestor() {
        let (rest, expr) = parse("root//ee2/ancestor::root").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let root = doc
            .get_elements_by_tag_name("root")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(root, nodes[0]);
    }

    #[test]
    fn test_step_axis_ancestor_or_self() {
        let (rest, expr) = parse("root//ee2/ancestor-or-self::root").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let root = doc
            .get_elements_by_tag_name("root")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(root, nodes[0]);
    }

    #[test]
    fn test_step_axis_attribute() {
        let (rest, expr) = parse("root/e2[attribute::a]").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2 a='b'><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(e2, nodes[0]);
    }

    #[test]
    fn test_step_axis_child() {
        let (rest, expr) = parse("root/child::e2").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(e2, nodes[0]);
    }

    #[test]
    fn test_step_axis_descendant() {
        let (rest, expr) = parse("root/descendant::ee2").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let ee2 = doc
            .get_elements_by_tag_name("ee2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(ee2, nodes[0]);
    }

    #[test]
    fn test_step_axis_descendant_or_self() {
        let (rest, expr) = parse("root/descendant-or-self::ee2").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let ee2 = doc
            .get_elements_by_tag_name("ee2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(ee2, nodes[0]);
    }

    #[test]
    fn test_step_axis_following() {
        let (rest, expr) = parse("root/e2/following::ee3").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let ee3 = doc
            .get_elements_by_tag_name("ee3")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(ee3, nodes[0]);
    }

    #[test]
    fn test_step_axis_following_sibling() {
        let (rest, expr) = parse("root/e2/following-sibling::e3").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e3 = doc
            .get_elements_by_tag_name("e3")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(e3, nodes[0]);
    }

    #[test]
    fn test_step_axis_namespace() {
        let (rest, expr) = parse("root/e2[namespace::a]").unwrap();
        assert_eq!("", rest);

        let (rest, tree) =
            xml_parser::document("<root><e2 /><a:e2 xmlns:a='http://test/a' /><e2 /></root>")
                .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .nth(1)
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(e2, nodes[0]);
    }

    #[test]
    fn test_step_axis_parent() {
        let (rest, expr) = parse("root/e2/parent::*").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let root = doc
            .get_elements_by_tag_name("root")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(root, nodes[0]);
    }

    #[test]
    fn test_step_axis_preceding() {
        let (rest, expr) = parse("root/e2/preceding::ee1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let ee1 = doc
            .get_elements_by_tag_name("ee1")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(ee1, nodes[0]);
    }

    #[test]
    fn test_step_axis_preceding_sibling() {
        let (rest, expr) = parse("root/e2/preceding-sibling::e1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e1 = doc
            .get_elements_by_tag_name("e1")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(e1, nodes[0]);
    }

    #[test]
    fn test_step_axis_self() {
        let (rest, expr) = parse("root/e2/self::*").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(e2, nodes[0]);
    }

    #[test]
    fn test_step_axis_abbr_element() {
        let (rest, expr) = parse("root/e2/ee2").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let ee2 = doc
            .get_elements_by_tag_name("ee2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(ee2, nodes[0]);
    }

    #[test]
    fn test_step_axis_abbr_attr() {
        let (rest, expr) = parse("root/e2[@a]").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2 a='b'><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(e2, nodes[0]);
    }

    #[test]
    fn test_step_abbr_self() {
        let (rest, expr) = parse("root/e2/.").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(e2, nodes[0]);
    }

    #[test]
    fn test_step_abbr_parent() {
        let (rest, expr) = parse("root/e2/..").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let root = doc
            .get_elements_by_tag_name("root")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(root, nodes[0]);
    }

    #[test]
    fn test_func_last() {
        let (rest, expr) = parse("last()").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let mut ctx = model::Context::default();
        ctx.push_size(1);
        let r = document(&expr, doc.clone(), &mut ctx).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(1f64, ret);
    }

    #[test]
    fn test_func_position() {
        let (rest, expr) = parse("position()").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let mut ctx = model::Context::default();
        ctx.push_position(1);
        let r = document(&expr, doc.clone(), &mut ctx).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(1f64, ret);
    }

    #[test]
    fn test_func_count() {
        let (rest, expr) = parse("count(/root/*)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(3f64, ret);
    }

    #[test]
    fn test_func_local_name() {
        let (rest, expr) = parse("local-name(/root)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!("root", ret);
    }

    #[test]
    fn test_func_namespace_uri() {
        let (rest, expr) = parse("namespace-uri(/root)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root xmlns='http://test/'></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!("http://test/", ret);
    }

    #[test]
    fn test_func_name() {
        let (rest, expr) = parse("name(/root)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) =
            xml_parser::document("<a:root xmlns:a='http://test/a'></a:root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!("a:root", ret);
    }

    #[test]
    fn test_func_string() {
        let (rest, expr) = parse("string(/root)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root>text1</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!("text1", ret);
    }

    #[test]
    fn test_func_concat() {
        let (rest, expr) = parse("concat(/root, '2')").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root>text1</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!("text12", ret);
    }

    #[test]
    fn test_func_starts_with() {
        let (rest, expr) = parse("starts-with(/root, 'te')").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root>text1</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(n) = r {
            n
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_func_contains() {
        let (rest, expr) = parse("contains(/root, 'ex')").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root>text1</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(n) = r {
            n
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_func_substring_before() {
        let (rest, expr) = parse("substring-before(/root, 'ex')").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root>text1</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!("t", ret);
    }

    #[test]
    fn test_func_substring_after() {
        let (rest, expr) = parse("substring-after(/root, 'ex')").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root>text1</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!("t1", ret);
    }

    #[test]
    fn test_func_substring() {
        let (rest, expr) = parse("substring(/root, 2, 3)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root>text1</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!("ext", ret);
    }

    #[test]
    fn test_func_string_length() {
        let (rest, expr) = parse("string-length(/root)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root>text1</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(5f64, ret);
    }

    #[test]
    fn test_func_normalize_space() {
        let (rest, expr) = parse("normalize-space(/root)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root> te  x t   1 </root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!("te x t 1", ret);
    }

    #[test]
    fn test_func_translate() {
        let (rest, expr) = parse("translate(/root, 'abc-', 'ABC')").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root>--abcd--</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!("ABCd", ret);
    }

    #[test]
    fn test_func_boolean() {
        let (rest, expr) = parse("boolean(true())").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root />").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(n) = r {
            n
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_func_not() {
        let (rest, expr) = parse("not(false())").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root />").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(n) = r {
            n
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_func_true() {
        let (rest, expr) = parse("true()").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root />").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(n) = r {
            n
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_func_false() {
        let (rest, expr) = parse("false()").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root />").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(n) = r {
            n
        } else {
            unreachable!()
        };
        assert!(!ret);
    }

    #[test]
    fn test_func_lang() {
        let (rest, expr) = parse("root[lang('ja')]").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root xml:lang='ja'/>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let root = doc
            .get_elements_by_tag_name("root")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(root, nodes[0]);
    }

    #[test]
    fn test_func_number() {
        let (rest, expr) = parse("number(1)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root />").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(1f64, ret);
    }

    #[test]
    fn test_func_sum() {
        let (rest, expr) = parse("sum(/root/e)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root><e>1</e><e>3</e><e>5</e></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(9f64, ret);
    }

    #[test]
    fn test_func_floor() {
        let (rest, expr) = parse("floor(3.6)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root />").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(3f64, ret);
    }

    #[test]
    fn test_func_ceiling() {
        let (rest, expr) = parse("ceiling(3.6)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root />").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(4f64, ret);
    }

    #[test]
    fn test_func_round() {
        let (rest, expr) = parse("round(3.6)").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root />").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(4f64, ret);
    }

    #[test]
    fn test_or_expr_true() {
        let (rest, expr) = parse("1 or 0").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_or_expr_false() {
        let (rest, expr) = parse("0 or 0").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(!ret);
    }

    #[test]
    fn test_and_expr_true() {
        let (rest, expr) = parse("1 and 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_and_expr_false() {
        let (rest, expr) = parse("1 and 0").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(!ret);
    }

    #[test]
    fn test_eq_expr_true() {
        let (rest, expr) = parse("1 = 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_eq_expr_false() {
        let (rest, expr) = parse("1 = 0").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(!ret);
    }

    #[test]
    fn test_not_eq_expr_true() {
        let (rest, expr) = parse("1 != 0").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_not_eq_expr_false() {
        let (rest, expr) = parse("1 != 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(!ret);
    }

    #[test]
    fn test_lt_expr_true() {
        let (rest, expr) = parse("0 < 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_lt_expr_false() {
        let (rest, expr) = parse("1 < 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(!ret);
    }

    #[test]
    fn test_gt_expr_true() {
        let (rest, expr) = parse("1 > 0").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_gt_expr_false() {
        let (rest, expr) = parse("1 > 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(!ret);
    }

    #[test]
    fn test_le_expr_true() {
        let (rest, expr) = parse("1 <= 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_le_expr_false() {
        let (rest, expr) = parse("2 <= 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(!ret);
    }

    #[test]
    fn test_ge_expr_true() {
        let (rest, expr) = parse("1 >= 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(ret);
    }

    #[test]
    fn test_ge_expr_false() {
        let (rest, expr) = parse("1 >= 2").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Boolean(b) = r {
            b
        } else {
            unreachable!()
        };
        assert!(!ret);
    }

    #[test]
    fn test_add_expr() {
        let (rest, expr) = parse("2 + 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(3f64, ret);
    }

    #[test]
    fn test_sub_expr() {
        let (rest, expr) = parse("2 - 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(1f64, ret);
    }

    #[test]
    fn test_mul_expr() {
        let (rest, expr) = parse("2 * 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(2f64, ret);
    }

    #[test]
    fn test_div_expr() {
        let (rest, expr) = parse("2 div 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(2f64, ret);
    }

    #[test]
    fn test_mod_expr() {
        let (rest, expr) = parse("2 mod 1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(0f64, ret);
    }

    #[test]
    fn test_neg_expr() {
        let (rest, expr) = parse("-1").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Number(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(-1f64, ret);
    }

    #[test]
    fn test_text() {
        let (rest, expr) = parse("'text'").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let ret = if let model::Value::Text(t) = r {
            t
        } else {
            unreachable!()
        };
        assert_eq!("text", ret);
    }

    // TODO: NameTest

    #[test]
    fn test_node_type_comment() {
        let (rest, expr) = parse("root/e2/comment()").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><!--a--><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let comment = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .first_child()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(comment, nodes[0]);
    }

    #[test]
    fn test_node_type_text() {
        let (rest, expr) = parse("root/e2/ee2/text()").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let text = doc
            .get_elements_by_tag_name("ee2")
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .first_child()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(text, nodes[0]);
    }

    // TODO: predicate

    // TODO: NodeType: processing-instruction
    // TODO: NodeType: processing-instruction()

    #[test]
    fn test_node_type_node() {
        let (rest, expr) = parse("root/e2/node()").unwrap();
        assert_eq!("", rest);

        let (rest, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let ee2 = doc
            .get_elements_by_tag_name("ee2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let r = document(&expr, doc.clone(), &mut model::Context::default()).unwrap();
        let nodes = if let model::Value::Node(n) = r {
            n
        } else {
            unreachable!()
        };
        assert_eq!(ee2, nodes[0]);
    }

    #[test]
    fn test_ancestor() {
        let (_, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let root = doc
            .get_elements_by_tag_name("root")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let n = ancestor(e2);
        assert_eq!(vec![root, doc.as_node()], n);
    }

    #[test]
    fn test_ancestor_and_self() {
        let (_, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let root = doc
            .get_elements_by_tag_name("root")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let n = ancestor_and_self(e2.clone());
        assert_eq!(vec![e2, root, doc.as_node()], n);
    }

    #[test]
    fn test_attributes() {
        let (_, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let n = attributes(e2);
        assert!(n.is_empty());
    }

    #[test]
    fn test_child() {
        let (_, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let ee2 = e2.first_child().unwrap();

        let n = child(e2);
        assert_eq!(vec![ee2], n);
    }

    #[test]
    fn test_descendant() {
        let (_, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let ee2 = e2.first_child().unwrap();
        let text2 = ee2.first_child().unwrap();

        let n = descendant(e2);
        assert_eq!(vec![ee2, text2], n);
    }

    #[test]
    fn test_descendant_and_self() {
        let (_, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let ee2 = e2.first_child().unwrap();
        let text2 = ee2.first_child().unwrap();

        let n = descendant_and_self(e2.clone());
        assert_eq!(vec![e2, ee2, text2], n);
    }

    #[test]
    fn test_following() {
        let (_, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let e3 = doc
            .get_elements_by_tag_name("e3")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let ee3 = doc
            .get_elements_by_tag_name("ee3")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let text3 = ee3.first_child().unwrap();

        let n = following(e2);
        assert_eq!(vec![e3, ee3, text3], n);
    }

    #[test]
    fn test_following_sibling() {
        let (_, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let e3 = doc
            .get_elements_by_tag_name("e3")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let n = following_sibling(e2);
        assert_eq!(vec![e3], n);
    }

    #[test]
    fn test_preceding() {
        let (_, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e1 = doc
            .get_elements_by_tag_name("e1")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let ee1 = doc
            .get_elements_by_tag_name("ee1")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let text1 = ee1.first_child().unwrap();
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let n = preceding(e2);
        assert_eq!(vec![text1, ee1, e1], n);
    }

    #[test]
    fn test_preceding_sibling() {
        let (_, tree) = xml_parser::document(
            "<root><e1><ee1>1</ee1></e1><e2><ee2>2</ee2></e2><e3><ee3>3</ee3></e3></root>",
        )
        .unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let e1 = doc
            .get_elements_by_tag_name("e1")
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let e2 = doc
            .get_elements_by_tag_name("e2")
            .unwrap()
            .iter()
            .next()
            .unwrap();

        let n = preceding_sibling(e2);
        assert_eq!(vec![e1], n);
    }

    #[test]
    fn test_bool_eq_bool_bool() {
        let a = false.as_value();
        let b = false.as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());

        let a = false.as_value();
        let b = true.as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = false.as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = true.as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_eq_bool_node() {
        let a = false.as_value();
        let b = vec![].as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());

        let a = false.as_value();
        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = vec![].as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = true.as_value();
        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_eq_bool_number() {
        let a = false.as_value();
        let b = 0f64.as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());

        let a = false.as_value();
        let b = 1f64.as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = 0f64.as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = 1f64.as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_eq_bool_text() {
        let a = false.as_value();
        let b = "".as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());

        let a = false.as_value();
        let b = "1".as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = "".as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = "1".as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_cmp_bool_bool() {
        let a = false.as_value();
        let b = false.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = false.as_value();
        let b = true.as_value();
        assert!(!greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(less_than_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = false.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(greater_than_value(&a, &b).unwrap());
        assert!(!less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = true.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_cmp_bool_node() {
        let a = false.as_value();
        let b = vec![].as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = false.as_value();
        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(!greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(less_than_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = vec![].as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(greater_than_value(&a, &b).unwrap());
        assert!(!less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = true.as_value();
        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_cmp_bool_number() {
        let a = false.as_value();
        let b = 0f64.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = false.as_value();
        let b = 1f64.as_value();
        assert!(!greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(less_than_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = 0f64.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(greater_than_value(&a, &b).unwrap());
        assert!(!less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = 1f64.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_cmp_bool_text() {
        let a = false.as_value();
        let b = "0".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = false.as_value();
        let b = "1".as_value();
        assert!(!greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(less_than_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = "0".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(greater_than_value(&a, &b).unwrap());
        assert!(!less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = true.as_value();
        let b = "1".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_eq_node_node() {
        let (_, tree) = xml_parser::document("<root>a</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let (_, tree) = xml_parser::document("<root>a</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>a</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let (_, tree) = xml_parser::document("<root>b</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>b</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let (_, tree) = xml_parser::document("<root>a</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>b</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let (_, tree) = xml_parser::document("<root>b</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_eq_node_number() {
        let (_, tree) = xml_parser::document("<root>0</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = 0f64.as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>0</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = 1f64.as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = 0f64.as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = 1f64.as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_eq_node_text() {
        let (_, tree) = xml_parser::document("<root>a</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = "a".as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>a</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = "b".as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>b</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = "a".as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>b</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = "b".as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_cmp_node_node() {
        let (_, tree) = xml_parser::document("<root>0</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let (_, tree) = xml_parser::document("<root>0</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>0</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(!greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(less_than_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let (_, tree) = xml_parser::document("<root>0</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(greater_than_value(&a, &b).unwrap());
        assert!(!less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let b = doc.as_node().as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_cmp_node_number() {
        let (_, tree) = xml_parser::document("<root>0</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = 0f64.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>0</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = 1f64.as_value();
        assert!(!greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(less_than_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = 0f64.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(greater_than_value(&a, &b).unwrap());
        assert!(!less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = 1f64.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_cmp_node_text() {
        let (_, tree) = xml_parser::document("<root>0</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = "0".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>0</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = "1".as_value();
        assert!(!greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(less_than_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = "0".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(greater_than_value(&a, &b).unwrap());
        assert!(!less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let (_, tree) = xml_parser::document("<root>1</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let a = doc.as_node().as_value();
        let b = "1".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_eq_number_number() {
        let a = 0f64.as_value();
        let b = 0f64.as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());

        let a = 0f64.as_value();
        let b = 1f64.as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = 1f64.as_value();
        let b = 0f64.as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = 1f64.as_value();
        let b = 1f64.as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_eq_number_text() {
        let a = 0f64.as_value();
        let b = "0".as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());

        let a = 0f64.as_value();
        let b = "1".as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = 1f64.as_value();
        let b = "0".as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = 1f64.as_value();
        let b = "1".as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_cmp_number_number() {
        let a = 0f64.as_value();
        let b = 0f64.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = 0f64.as_value();
        let b = 1f64.as_value();
        assert!(!greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(less_than_value(&a, &b).unwrap());

        let a = 1f64.as_value();
        let b = 0f64.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(greater_than_value(&a, &b).unwrap());
        assert!(!less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = 1f64.as_value();
        let b = 1f64.as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_cmp_number_text() {
        let a = 0f64.as_value();
        let b = "0".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = 0f64.as_value();
        let b = "1".as_value();
        assert!(!greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(less_than_value(&a, &b).unwrap());

        let a = 1f64.as_value();
        let b = "0".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(greater_than_value(&a, &b).unwrap());
        assert!(!less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = 1f64.as_value();
        let b = "1".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_eq_text_text() {
        let a = "".as_value();
        let b = "".as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());

        let a = "".as_value();
        let b = "1".as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = "1".as_value();
        let b = "".as_value();
        assert!(!equal_value(&a, &b).unwrap());
        assert!(not_equal_value(&a, &b).unwrap());

        let a = "1".as_value();
        let b = "1".as_value();
        assert!(equal_value(&a, &b).unwrap());
        assert!(!not_equal_value(&a, &b).unwrap());
    }

    #[test]
    fn test_bool_cmp_text_text() {
        let a = "0".as_value();
        let b = "0".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = "0".as_value();
        let b = "1".as_value();
        assert!(!greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(less_than_value(&a, &b).unwrap());

        let a = "1".as_value();
        let b = "0".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(greater_than_value(&a, &b).unwrap());
        assert!(!less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());

        let a = "1".as_value();
        let b = "1".as_value();
        assert!(greater_eq_value(&a, &b).unwrap());
        assert!(!greater_than_value(&a, &b).unwrap());
        assert!(less_eq_value(&a, &b).unwrap());
        assert!(!less_than_value(&a, &b).unwrap());
    }
}
