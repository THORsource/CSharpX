use super::{emit_expr, Expr, Parser, Ty};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TemporalType {
    DateOnly,
    TimeOnly,
    DateTime,
    TimeSpan,
}

impl TemporalType {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "DateOnly" => Some(Self::DateOnly),
            "TimeOnly" => Some(Self::TimeOnly),
            "DateTime" => Some(Self::DateTime),
            "TimeSpan" => Some(Self::TimeSpan),
            _ => None,
        }
    }
}

impl Parser {
    pub(super) fn temporal_constructor(&mut self) -> Result<Expr, String> {
        let name = self.name()?;
        let ty = TemporalType::from_name(&name)
            .ok_or_else(|| format!("CSX1401: unknown temporal type {name}"))?;
        self.expect('(')?;
        let mut args = vec![self.expr(0)?];
        while self.sym(',') {
            args.push(self.expr(0)?);
        }
        self.expect(')')?;
        Ok(Expr::Temporal(ty, args))
    }
}

pub(super) fn constructor(
    ty: TemporalType,
    args: &[Expr],
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, Ty), String> {
    use TemporalType::*;
    let mut values = Vec::new();
    for arg in args {
        let (code, kind) = emit_expr(arg, vars)?;
        if kind != Ty::Int {
            return Err("CSX1403: date/time constructors require integer arguments".into());
        }
        values.push(code);
    }
    let constructor = match (ty, args.len()) {
        (DateOnly, 3) => "date",
        (TimeOnly, 2 | 3) => "time",
        (DateTime, 3 | 6) => "date_time",
        (TimeSpan, 3 | 4) => "span",
        _ => return Err(format!("CSX1404: unsupported argument count for {ty:?}")),
    };
    match (ty, args.len()) {
        (TimeOnly, 2) => values.push("0".into()),
        (DateTime, 3) => values.extend(["0".into(), "0".into(), "0".into()]),
        (TimeSpan, 3) => values.insert(0, "0".into()),
        _ => {}
    }
    Ok((
        format!("csx_temporal::{constructor}({})?", values.join(", ")),
        Ty::Temporal(ty),
    ))
}

pub(super) fn member(
    receiver: &Expr,
    member: &str,
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, Ty), String> {
    use TemporalType::*;
    let (receiver, kind) = emit_expr(receiver, vars)?;
    let access = match (kind, member) {
        (Ty::Temporal(DateOnly), "Year") => "year",
        (Ty::Temporal(DateOnly), "Month") => "month",
        (Ty::Temporal(DateOnly), "Day") => "day",
        (Ty::Temporal(TimeOnly), "Hour") => "hour",
        (Ty::Temporal(TimeOnly), "Minute") => "minute",
        (Ty::Temporal(TimeOnly), "Second") => "second",
        (Ty::Temporal(DateTime), "Year") => "date.year",
        (Ty::Temporal(DateTime), "Month") => "date.month",
        (Ty::Temporal(DateTime), "Day") => "date.day",
        (Ty::Temporal(DateTime), "Hour") => "time.hour",
        (Ty::Temporal(DateTime), "Minute") => "time.minute",
        (Ty::Temporal(DateTime), "Second") => "time.second",
        (Ty::Temporal(TimeSpan), "Days") => "days()",
        (Ty::Temporal(TimeSpan), "Hours") => "hours()",
        (Ty::Temporal(TimeSpan), "Minutes") => "minutes()",
        (Ty::Temporal(TimeSpan), "Seconds") => "seconds()",
        _ => return Err(format!("CSX1405: unsupported temporal property {member}")),
    };
    Ok((format!("({receiver}).{access}"), Ty::Int))
}

pub(super) fn format(
    receiver: &Expr,
    format: Option<&str>,
    vars: &BTreeMap<String, Ty>,
) -> Result<(String, Ty), String> {
    use TemporalType::*;
    let (receiver, kind) = emit_expr(receiver, vars)?;
    let template = match (kind, format) {
        (Ty::Temporal(_), None)
        | (Ty::Temporal(DateOnly), Some("yyyy-MM-dd"))
        | (Ty::Temporal(TimeOnly), Some("HH:mm:ss"))
        | (Ty::Temporal(TimeSpan), Some("c")) => {
            return Ok((format!("({receiver}).to_string()"), Ty::Str));
        }
        (Ty::Temporal(DateTime), Some("yyyy-MM-dd HH:mm:ss")) => " ",
        (Ty::Temporal(DateTime), Some("yyyy-MM-dd'T'HH:mm:ss")) => "T",
        _ => return Err("CSX1406: unsupported date/time format for this type".into()),
    };
    Ok((format!("{{ let csx_datetime = {receiver}; format!(\"{{}}{template}{{}}\", csx_datetime.date, csx_datetime.time) }}"), Ty::Str))
}

pub(crate) fn append_runtime(rust: &mut String) {
    if rust.contains("csx_temporal::") {
        rust.push_str(include_str!("temporal_runtime.rs"));
    }
}
#[cfg(test)]
mod tests {

    #[test]
    fn temporal_initializer_and_format_are_checked() {
        let valid = crate::language::compile(
            "void Main() { DateOnly date = new DateOnly(2028, 2, 29); Console.WriteLine(date.ToString(\"yyyy-MM-dd\")); }",
        ).unwrap();
        assert!(valid.contains("csx_temporal::date"));
        let wrong_type =
            crate::language::compile("void Main() { TimeOnly t = new DateOnly(2028, 2, 29); }");
        assert!(wrong_type.unwrap_err().contains("CSX1407"));
    }
}
