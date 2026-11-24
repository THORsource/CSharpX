use super::{emit_expr, temporal::TemporalType, Expr, Parser, Tok, Ty};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Element {
    Int,
    String,
    Bool,
    Temporal(TemporalType),
}

impl Element {
    pub(super) fn from_type(ty: &Ty) -> Result<Self, String> {
        match ty {
            Ty::Int => Ok(Self::Int),
            Ty::Str => Ok(Self::String),
            Ty::Bool => Ok(Self::Bool),
            Ty::Temporal(ty) => Ok(Self::Temporal(*ty)),
            Ty::Tuple(_) => Err("CSX1601: arrays of tuples are not supported".into()),
            Ty::Pointer(_) => Err("CSX1601: arrays of pointers are not supported".into()),
            Ty::Array(_) => Err("CSX1601: nested arrays are not supported".into()),
        }
    }

    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "int" => Some(Self::Int),
            "string" | "String" => Some(Self::String),
            "bool" => Some(Self::Bool),
            name => TemporalType::from_name(name).map(Self::Temporal),
        }
    }

    fn scalar(self) -> Ty {
        match self {
            Self::Int => Ty::Int,
            Self::String => Ty::Str,
            Self::Bool => Ty::Bool,
            Self::Temporal(ty) => Ty::Temporal(ty),
        }
    }

    fn rust_type(self) -> &'static str {
        match self {
            Self::Int => "i64",
            Self::String => "String",
            Self::Bool => "bool",
            Self::Temporal(TemporalType::DateOnly) => "csx_temporal::Date",
            Self::Temporal(TemporalType::TimeOnly) => "csx_temporal::Time",
            Self::Temporal(TemporalType::DateTime) => "csx_temporal::DateTime",
            Self::Temporal(TemporalType::TimeSpan) => "csx_temporal::Span",
        }
    }

    fn default(self) -> Result<&'static str, String> {
        match self {
            Self::Int => Ok("0i64"),
            Self::Bool => Ok("false"),
            Self::String => Err(
                "CSX1602: string arrays require an initializer; nullable strings are not supported"
                    .into(),
            ),
            Self::Temporal(TemporalType::DateOnly) => Ok("csx_temporal::date(1, 1, 1)?"),
            Self::Temporal(TemporalType::TimeOnly) => Ok("csx_temporal::time(0, 0, 0)?"),
            Self::Temporal(TemporalType::DateTime) => {
                Ok("csx_temporal::date_time(1, 1, 1, 0, 0, 0)?")
            }
            Self::Temporal(TemporalType::TimeSpan) => Ok("csx_temporal::span(0, 0, 0, 0)?"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Initializer {
    Items(Vec<Expr>),
    Length(Box<Expr>),
}

impl Parser {
    pub(super) fn array_or_temporal(&mut self) -> Result<Expr, String> {
        if self.sym('[') {
            self.expect(']')?;
            return self.array_items(None);
        }
        let Some([Tok::Ident(name), Tok::Sym('['), ..]) = self.t.get(self.i..) else {
            return self.temporal_constructor();
        };
        let ty = Element::from_name(name)
            .ok_or_else(|| format!("CSX1601: unsupported array element type {name}"))?;
        self.i += 2;
        if self.sym(']') {
            return self.array_items(Some(ty));
        }
        let length = self.expr(0)?;
        self.expect(']')?;
        Ok(Expr::Array(Some(ty), Initializer::Length(Box::new(length))))
    }

    pub(super) fn array_items(&mut self, element: Option<Element>) -> Result<Expr, String> {
        self.expect('{')?;
        let mut items = Vec::new();
        while !self.sym('}') {
            items.push(self.expr(0)?);
            if !self.sym(',') {
                self.expect('}')?;
                break;
            }
        }
        Ok(Expr::Array(element, Initializer::Items(items)))
    }
}

pub(super) fn emit_array(
    element: Option<Element>,
    init: &Initializer,
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, Ty), String> {
    match init {
        Initializer::Length(length) => {
            let element = element.ok_or("CSX1601: array length requires a type")?;
            let (length, ty) = emit_expr(length, vars)?;
            if ty != Ty::Int {
                return Err("CSX1603: array length must be an integer".into());
            }
            Ok((
                format!("csx_array::filled({length}, {})?", element.default()?),
                Ty::Array(element),
            ))
        }
        Initializer::Items(items) => {
            let mut element = element;
            let mut values = Vec::new();
            for expression in items {
                let (value, kind) = emit_expr(expression, vars)?;
                let expected = element.get_or_insert(Element::from_type(&kind)?);
                if kind != expected.scalar() {
                    return Err("CSX1604: incompatible array element type".into());
                }
                values.push(value);
            }
            let element = element.ok_or("CSX1601: cannot infer the type of an empty array")?;
            let ty = element.rust_type();
            Ok((
                format!(
                    "csx_array::from_vec(Vec::<{ty}>::from([{}]))",
                    values.join(", ")
                ),
                Ty::Array(element),
            ))
        }
    }
}

fn target(
    array: &Expr,
    index: &Expr,
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, String, Ty), String> {
    let (array, ty) = emit_expr(array, vars)?;
    let Ty::Array(element) = ty else {
        return Err("CSX1605: indexed value must be an array".into());
    };
    let (index, index_ty) = emit_expr(index, vars)?;
    if index_ty != Ty::Int {
        return Err("CSX1603: array index must be an integer".into());
    }
    Ok((array, index, element.scalar()))
}

pub(super) fn get(
    array: &Expr,
    index: &Expr,
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, Ty), String> {
    let (array, index, element) = target(array, index, vars)?;
    Ok((format!("csx_array::get(&({array}), {index})?"), element))
}

pub(super) fn set(
    array: &Expr,
    index: &Expr,
    value: &Expr,
    vars: &BTreeMap<String, Ty>,
) -> Result<String, String> {
    let (array, index, element) = target(array, index, vars)?;
    let (value, ty) = emit_expr(value, vars)?;
    if ty != element {
        return Err("CSX1604: incompatible array element type".into());
    }
    Ok(format!("csx_array::set(&({array}), {index}, {value})?;"))
}

pub(crate) fn append_runtime(rust: &mut String) {
    if rust.contains("csx_array::") {
        rust.push_str(include_str!("array_runtime.rs"));
    }
}

#[cfg(test)]
mod tests {
    use crate::language::compile;

    #[test]
    fn supports_inferred_arrays_and_checked_index_assignment() {
        let code = compile("void Main() { var a = new[] { 1, 2 }; int[] b = a; b[0] = b[1]; Console.WriteLine(a.Length); }").unwrap();
        assert!(code.contains("csx_array::get") && code.contains("csx_array::set"));
        assert!(code.contains(".borrow().len()"));
    }

    #[test]
    fn rejects_incompatible_elements_and_indices() {
        assert!(compile(r#"void Main() { int[] a = { 1, "x" }; }"#)
            .unwrap_err()
            .contains("CSX1604"));
        assert!(compile(r#"void Main() { int[] a = { 1 }; a["x"] = 2; }"#)
            .unwrap_err()
            .contains("CSX1603"));
    }
}
