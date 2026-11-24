use std::collections::{BTreeMap, BTreeSet};

use super::{display_expression, emit_expr, strings, Expr, Parser, Tok, Ty};

#[derive(Clone, PartialEq)]
pub(crate) struct Field {
    name: Option<String>,
    ty: Ty,
}

fn expression_element(parser: &mut Parser) -> Result<(Option<String>, Expr), String> {
    let name = if matches!(&parser.t[parser.i..], [Tok::Ident(_), Tok::Sym(':'), ..]) {
        let name = parser.name()?;
        parser.expect(':')?;
        Some(name)
    } else {
        None
    };
    Ok((name, parser.expr(0)?))
}

pub(super) fn expression(parser: &mut Parser) -> Result<Expr, String> {
    let first = expression_element(parser)?;
    if first.0.is_none() && parser.sym(')') {
        return Ok(first.1);
    }
    parser.expect(',')?;
    let mut elements = vec![first];
    loop {
        elements.push(expression_element(parser)?);
        if !parser.sym(',') {
            break;
        }
    }
    parser.expect(')')?;
    Ok(Expr::Tuple(elements))
}

pub(super) fn declared_type(parser: &mut Parser) -> Result<Ty, String> {
    let mut fields = Vec::new();
    loop {
        let ty = if parser.sym('(') {
            declared_type(parser)?
        } else {
            match parser.name()?.as_str() {
                "int" => Ty::Int,
                "string" | "String" => Ty::Str,
                "bool" => Ty::Bool,
                _ => {
                    return Err(
                        "CSX1950: tuple elements must be int, string, bool or a tuple".into(),
                    )
                }
            }
        };
        let name = if matches!(parser.peek(), Tok::Ident(_)) {
            Some(parser.name()?)
        } else {
            None
        };
        fields.push(Field { name, ty });
        if !parser.sym(',') {
            break;
        }
    }
    parser.expect(')')?;
    if fields.len() < 2 {
        return Err("CSX1950: a tuple requires at least two elements".into());
    }
    validate_names(&fields)?;
    Ok(Ty::Tuple(fields))
}

fn validate_names(fields: &[Field]) -> Result<(), String> {
    let mut names = BTreeSet::new();
    for (index, field) in fields.iter().enumerate() {
        if let Some(name) = &field.name {
            let reserved = item_index(name).is_some_and(|position| position != index);
            if reserved || !names.insert(name) {
                return Err(format!(
                    "CSX1951: invalid or duplicate tuple element name {name}"
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn emit(
    elements: &[(Option<String>, Expr)],
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, Ty), String> {
    let mut fields = Vec::new();
    let mut values = Vec::new();
    let mut used: BTreeSet<&str> = elements
        .iter()
        .filter_map(|(name, _)| name.as_deref())
        .collect();
    for (index, (name, expr)) in elements.iter().enumerate() {
        let (code, ty) = emit_expr(expr, vars)?;
        if !matches!(ty, Ty::Int | Ty::Str | Ty::Bool | Ty::Tuple(_)) {
            return Err("CSX1950: tuple elements must be int, string, bool or a tuple".into());
        }
        let inferred = match expr {
            Expr::Var(candidate, _)
                if name.is_none()
                    && item_index(candidate).is_none_or(|position| position == index)
                    && used.insert(candidate) =>
            {
                Some(candidate.clone())
            }
            _ => None,
        };
        fields.push(Field {
            name: name.clone().or(inferred),
            ty,
        });
        values.push(code);
    }
    validate_names(&fields)?;
    Ok((format!("({})", values.join(", ")), Ty::Tuple(fields)))
}

fn item_index(name: &str) -> Option<usize> {
    let suffix = name.strip_prefix("Item")?;
    if suffix.starts_with('0') {
        return None;
    }
    suffix.parse::<usize>().ok()?.checked_sub(1)
}

fn find(fields: &[Field], member: &str) -> Result<usize, String> {
    item_index(member)
        .filter(|index| *index < fields.len())
        .or_else(|| {
            fields
                .iter()
                .position(|field| field.name.as_deref() == Some(member))
        })
        .ok_or_else(|| format!("CSX1952: tuple has no element named {member}"))
}

pub(super) fn member(
    receiver: &Expr,
    member: &str,
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, Ty), String> {
    let (code, ty) = emit_expr(receiver, vars)?;
    if let Ty::Tuple(fields) = ty {
        let index = find(&fields, member)?;
        Ok((format!("({code}).{index}"), fields[index].ty.clone()))
    } else {
        strings::property(receiver, member, vars)
    }
}

pub(super) fn compatible(expected: &Ty, actual: &Ty) -> bool {
    match (expected, actual) {
        (Ty::Tuple(a), Ty::Tuple(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| compatible(&a.ty, &b.ty))
        }
        _ => expected == actual,
    }
}

pub(super) fn assign_field(
    name: &str,
    member: &str,
    value: &Expr,
    vars: &BTreeMap<String, Ty>,
    out: &mut String,
    pad: &str,
) -> Result<(), String> {
    let Some(Ty::Tuple(fields)) = vars.get(name) else {
        return Err(format!("CSX1952: {name} is not a tuple"));
    };
    let index = find(fields, member)?;
    let (code, ty) = emit_expr(value, vars)?;
    if !compatible(&fields[index].ty, &ty) {
        return Err("CSX1953: incompatible tuple element assignment".into());
    }
    out.push_str(&format!("{pad}v_{name}.{index} = {code};\n"));
    Ok(())
}

pub(super) fn deconstruct(
    names: &[String],
    expr: &Expr,
    vars: &mut BTreeMap<String, Ty>,
    out: &mut String,
    pad: &str,
) -> Result<(), String> {
    let (code, ty) = emit_expr(expr, vars)?;
    let Ty::Tuple(fields) = ty else {
        return Err("CSX1954: deconstruction requires a tuple".into());
    };
    if fields.len() != names.len() {
        return Err("CSX1954: deconstruction count does not match tuple arity".into());
    }
    let mut bindings = Vec::new();
    for (name, field) in names.iter().zip(fields) {
        if name == "_" {
            bindings.push("_".to_owned());
        } else {
            if vars.contains_key(name) {
                return Err(format!("CSX1203: duplicate variable {name}"));
            }
            vars.insert(name.clone(), field.ty);
            bindings.push(format!("mut v_{name}"));
        }
    }
    out.push_str(&format!("{pad}let ({}) = {code};\n", bindings.join(", ")));
    Ok(())
}

pub(super) fn display(code: String, fields: Vec<Field>) -> String {
    let template = vec!["{}"; fields.len()].join(", ");
    let args = fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| display_expression(format!("csx_tuple.{index}"), field.ty))
        .collect::<Vec<_>>();
    format!(
        "{{ let csx_tuple = {code}; format!(\"({template})\", {}) }}",
        args.join(", ")
    )
}
