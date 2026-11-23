use super::{emit_expr, temporal, Expr, Ty};
use std::collections::BTreeMap;

pub(super) fn property(
    receiver: &Expr,
    name: &str,
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, Ty), String> {
    if matches!(receiver, Expr::StringType) {
        return if name == "Empty" {
            Ok(("String::new()".into(), Ty::Str))
        } else {
            Err(format!("CSX1501: unsupported String property {name}"))
        };
    }
    let (code, ty) = emit_expr(receiver, vars)?;
    if matches!(ty, Ty::Array(_)) {
        return if name == "Length" {
            Ok((format!("({code}).borrow().len() as i64"), Ty::Int))
        } else {
            Err(format!("CSX1607: unsupported array property {name}"))
        };
    }
    if ty == Ty::Str {
        return if name == "Length" {
            Ok((format!("({code}).encode_utf16().count() as i64"), Ty::Int))
        } else {
            Err(format!("CSX1501: unsupported String property {name}"))
        };
    }
    temporal::member(receiver, name, vars)
}

pub(super) fn call(
    receiver: &Expr,
    name: &str,
    args: &[Expr],
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, Ty), String> {
    if matches!(receiver, Expr::StringType) {
        return static_call(name, args, vars);
    }
    let (value, ty) = emit_expr(receiver, vars)?;
    if matches!(ty, Ty::Temporal(_)) && name == "ToString" {
        let format = match args {
            [] => None,
            [Expr::Str(value)] => Some(value.as_str()),
            _ => return Err("CSX1402: expected a literal date/time format".into()),
        };
        return temporal::format(receiver, format, vars);
    }
    if ty != Ty::Str {
        return Err(format!("CSX1502: {name} requires a string receiver"));
    }
    let args = args
        .iter()
        .map(|arg| emit_expr(arg, vars))
        .collect::<Result<Vec<_>, _>>()?;
    let (expression, result) = match (name, args.as_slice()) {
        ("ToString", []) => (value, Ty::Str),
        ("Trim", []) => (format!("({value}).trim().to_owned()"), Ty::Str),
        ("ToUpperInvariant", []) => (format!("({value}).to_uppercase()"), Ty::Str),
        ("ToLowerInvariant", []) => (format!("({value}).to_lowercase()"), Ty::Str),
        ("Contains", [(needle, Ty::Str)]) => (format!("({value}).contains(&*({needle}))"), Ty::Bool),
        ("StartsWith", [(needle, Ty::Str)]) => (format!("({value}).starts_with(&*({needle}))"), Ty::Bool),
        ("EndsWith", [(needle, Ty::Str)]) => (format!("({value}).ends_with(&*({needle}))"), Ty::Bool),
        ("Equals", [(other, Ty::Str)]) => (format!("({value}) == ({other})"), Ty::Bool),
        ("IndexOf", [(needle, Ty::Str)]) => (
            format!("{{ let csx_text = {value}; csx_text.find(&*({needle})).map_or(-1i64, |at| csx_text[..at].encode_utf16().count() as i64) }}"),
            Ty::Int,
        ),
        ("Replace", [(old, Ty::Str), (new, Ty::Str)]) => (
            format!("{{ let csx_text = {value}; let csx_old = {old}; let csx_new = {new}; if csx_old.is_empty() {{ return Err(\"ArgumentException: empty replacement pattern\".into()); }} csx_text.replace(&csx_old, &csx_new) }}"),
            Ty::Str,
        ),
        ("Substring", [(start, Ty::Int)]) => (
            format!("csx_string::substring(&({value}), {start}, None)?"),
            Ty::Str,
        ),
        ("Substring", [(start, Ty::Int), (length, Ty::Int)]) => (
            format!("csx_string::substring(&({value}), {start}, Some({length}))?"),
            Ty::Str,
        ),
        _ => return Err(format!("CSX1503: unsupported String method or arguments: {name}")),
    };
    Ok((expression, result))
}

fn static_call(
    name: &str,
    args: &[Expr],
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, Ty), String> {
    let mut values = Vec::new();
    for arg in args {
        if matches!(arg, Expr::Null) {
            values.push("String::new()".into());
            continue;
        }
        let (value, kind) = emit_expr(arg, vars)?;
        if kind != Ty::Str {
            return Err(format!("CSX1503: String.{name} requires string arguments"));
        }
        values.push(value);
    }
    match (name, values.as_slice()) {
        ("IsNullOrEmpty", [value]) => Ok((format!("({value}).is_empty()"), Ty::Bool)),
        ("IsNullOrWhiteSpace", [value]) => Ok((format!("({value}).trim().is_empty()"), Ty::Bool)),
        ("Concat", [first, second, rest @ ..]) if rest.len() <= 2 => {
            let mut result = format!("({first}) + &({second})");
            for value in rest {
                result.push_str(&format!(" + &({value})"));
            }
            Ok((result, Ty::Str))
        }
        _ => Err(format!(
            "CSX1503: unsupported static String method or arguments: {name}"
        )),
    }
}

pub(crate) fn append_runtime(rust: &mut String) {
    if rust.contains("csx_string::substring(") {
        rust.push_str(include_str!("string_runtime.rs"));
    }
}

#[cfg(test)]
mod tests {
    use crate::language::compile;

    #[test]
    fn compiles_alias_length_and_substring() {
        let code = compile(r#"void Main() { System.String s = "a🙂b"; Console.WriteLine(s.Length); Console.WriteLine(s.Substring(1, 2)); }"#).unwrap();
        assert!(code.contains("encode_utf16().count()"));
        assert!(code.contains("csx_string::substring"));
    }

    #[test]
    fn rejects_invalid_string_types_and_overloads() {
        assert!(compile("void Main() { String s = 1; }")
            .unwrap_err()
            .contains("CSX1504"));
        assert!(
            compile(r#"void Main() { Console.WriteLine("test".Substring("x")); }"#)
                .unwrap_err()
                .contains("CSX1503")
        );
    }
}
