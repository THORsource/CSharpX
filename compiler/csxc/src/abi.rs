use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::language::{self, Tok, Ty};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Scalar {
    Int,
    Bool,
    Text,
}

impl Scalar {
    pub(crate) fn parse(name: &str) -> Result<Self, String> {
        match name {
            "int" => Ok(Self::Int),
            "bool" => Ok(Self::Bool),
            "string" => Ok(Self::Text),
            _ => Err(format!(
                "CSX2501: unsupported ABI type {name}; expected int, bool or string"
            )),
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Bool => "bool",
            Self::Text => "string",
        }
    }

    pub(crate) fn ty(self) -> Ty {
        match self {
            Self::Int => Ty::Int,
            Self::Bool => Ty::Bool,
            Self::Text => Ty::Str,
        }
    }

    pub(crate) fn from_type(ty: &Ty) -> Option<Self> {
        match ty {
            Ty::Int => Some(Self::Int),
            Ty::Bool => Some(Self::Bool),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Contract {
    pub class: String,
    pub function: String,
    pub result: Scalar,
    pub fields: BTreeMap<String, Scalar>,
    pub borrowed: bool,
}

pub(crate) fn identifier(value: &str) -> bool {
    crate::c_header::identifier(value) && value.len() <= 64
}

impl Contract {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !identifier(&self.class)
            || !identifier(&self.function)
            || !(1..=16).contains(&self.fields.len())
            || self.fields.keys().any(|name| !identifier(name))
            || self.fields.values().any(|ty| *ty == Scalar::Text)
        {
            return Err(
                "CSX2501: an ABI contract needs valid names and 1–16 primitive getters".into(),
            );
        }
        Ok(())
    }

    fn manifest(&self) -> String {
        let mut record = format!(
            "{}\n{}\n{}\n{}\n",
            match (self.borrowed, self.result == Scalar::Text) {
                (false, false) => "CSXABI1",
                (false, true) => "CSXABI2",
                (true, false) => "CSXABI3",
                (true, true) => "CSXABI4",
            },
            self.class,
            self.function,
            self.result.name()
        );
        for (name, ty) in &self.fields {
            record.push_str(&format!("{name} {}\n", ty.name()));
        }
        record
    }

    fn parse(record: &str) -> Result<Self, String> {
        let mut lines = record.lines();
        let version = lines.next();
        if !matches!(version, Some("CSXABI1" | "CSXABI2" | "CSXABI3" | "CSXABI4")) {
            return Err("CSX2502: unsupported CSharpX DLL contract version".into());
        }
        let missing = "CSX2502: incomplete CSharpX DLL contract";
        let class = lines.next().ok_or(missing)?.to_owned();
        let function = lines.next().ok_or(missing)?.to_owned();
        let result = Scalar::parse(lines.next().ok_or(missing)?)?;
        if matches!(version, Some("CSXABI2" | "CSXABI4")) != (result == Scalar::Text) {
            return Err("CSX2502: contract result does not match its ABI version".into());
        }
        let mut fields = BTreeMap::new();
        for row in lines {
            let (name, ty) = row.split_once(' ').ok_or(missing)?;
            if fields.insert(name.to_owned(), Scalar::parse(ty)?).is_some() {
                return Err("CSX2502: duplicate ABI getter".into());
            }
        }
        let contract = Self {
            class,
            function,
            result,
            fields,
            borrowed: matches!(version, Some("CSXABI3" | "CSXABI4")),
        };
        contract.validate()?;
        Ok(contract)
    }

    pub(crate) fn symbol(&self) -> String {
        // The full ABI identity is encoded, with no hash collisions.
        let mut name = String::from(match (self.borrowed, self.result == Scalar::Text) {
            (false, false) => "csx_abi_v1",
            (false, true) => "csx_abi_v2",
            (true, false) => "csx_abi_v3",
            (true, true) => "csx_abi_v4",
        });
        for token in self.manifest().split_ascii_whitespace().skip(1) {
            name.push_str(&format!("_{}_{token}", token.len()));
        }
        name
    }

    fn layout(&self) -> String {
        let fields = (0..self.fields.len())
            .map(|index| format!("f{index}: i64,"))
            .collect::<Vec<_>>()
            .join(" ");
        format!("#[repr(C)] struct Payload {{ {fields} }}\n#[repr(C)] struct Reply {{ status: i64, value: i64 }}\n")
    }

    pub(crate) fn emit_call(
        &self,
        function: &str,
        args: &[language::Expr],
        vars: &BTreeMap<String, Ty>,
    ) -> Result<(String, Ty), String> {
        if function != self.function {
            return Err(format!("CSX2503: DLL does not export {function}"));
        }
        let (first, text) = match (self.result, args) {
            (Scalar::Text, [first, text]) => (first, Some(text)),
            (Scalar::Int | Scalar::Bool, [first]) => (first, None),
            _ => {
                return Err(format!(
                    "CSX2503: incorrect argument count for {}",
                    self.function
                ))
            }
        };
        let language::Expr::Var(name, _) = first else {
            return Err(format!(
                "CSX2503: {} requires one local {} argument",
                self.function, self.class
            ));
        };
        let Some(Ty::Object(class, getters)) = vars.get(name) else {
            return Err(format!("CSX2503: expected an instance of {}", self.class));
        };
        let fields = getters
            .iter()
            .map(|(method, (_, ty))| Scalar::from_type(ty).map(|scalar| (method.clone(), scalar)))
            .collect::<Option<BTreeMap<_, _>>>();
        if class != &self.class || fields.as_ref() != Some(&self.fields) {
            return Err(format!(
                "CSX2504: class {class} does not match the DLL contract for {}",
                self.class
            ));
        }
        let configuration = if self.borrowed {
            format!("&*v_{name}")
        } else {
            let arguments = self
                .fields
                .iter()
                .map(|(method, ty)| {
                    let dispatch = &getters[method].0;
                    let call = format!("v_{name}.{dispatch}()");
                    if *ty == Scalar::Bool {
                        format!("i64::from({call})")
                    } else {
                        call
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{arguments}]")
        };
        let call = if let Some(text) = text {
            let (code, ty) = language::emit_expr(text, vars)?;
            if ty != Ty::Str {
                return Err("CSX2503: second DLL argument must be a string".into());
            }
            format!("::csx_typed_abi::invoke({configuration}, &({code}))?")
        } else {
            format!("::csx_typed_abi::invoke({configuration})?")
        };
        Ok((
            if self.result == Scalar::Bool {
                format!("({call} != 0)")
            } else {
                call
            },
            self.result.ty(),
        ))
    }

    pub(crate) fn import_shim(&self, dll: &Path) -> Result<String, String> {
        let name = dll
            .file_stem()
            .and_then(|name| name.to_str())
            .filter(|name| {
                !name.is_empty()
                    && name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
            })
            .ok_or(
                "CSX2502: DLL filename must contain only letters, digits, underscores or hyphens",
            )?;
        let payload = (0..self.fields.len())
            .map(|index| format!("f{index}: values[{index}]"))
            .collect::<Vec<_>>()
            .join(", ");
        if self.result == Scalar::Text {
            return Ok(crate::abi_text::import_shim(self, name, &payload));
        }
        if self.borrowed {
            return Ok(crate::abi_borrow::primitive_import(self, name));
        }
        let code = format!(
            r#"
#[link(name = "{name}", kind = "raw-dylib")]
extern "C" {{
    #[link_name = "{symbol}"]
    fn invoke_dll(payload: Payload) -> Reply;
}}
pub fn invoke(values: [i64; {count}]) -> Result<i64, String> {{
    // Generated against the verified versioned contract and by-value layout.
    let reply = unsafe {{ invoke_dll(Payload {{ {payload} }}) }};
    match reply.status {{
        0 {validation} => Ok(reply.value),
        _ => Err("CSharpX DLL call failed".into()),
    }}
}}
"#,
            symbol = self.symbol(),
            count = self.fields.len(),
            validation = if self.result == Scalar::Bool {
                "if matches!(reply.value, 0 | 1)"
            } else {
                ""
            }
        );
        Ok(self.layout() + &code)
    }

    pub(crate) fn read(dll: &Path) -> Result<Self, String> {
        if !dll.is_file() || dll.extension().and_then(|ext| ext.to_str()) != Some("dll") {
            return Err(format!("CSX2502: DLL not found: {}", dll.display()));
        }
        let manifest = dll.with_extension("csxabi");
        let record = fs::read_to_string(&manifest)
            .map_err(|error| format!("CSX2502: cannot read {}: {error}", manifest.display()))?;
        let contract = Self::parse(&record)?;
        verify_export(dll, &contract.symbol())?;
        Ok(contract)
    }

    pub(crate) fn write(&self, dll: &Path) -> Result<(), String> {
        verify_export(dll, &self.symbol())?;
        fs::write(dll.with_extension("csxabi"), self.manifest())
            .map_err(|error| format!("CSX2502: cannot write DLL contract: {error}"))
    }
}

// Read only the PE export directory. No DLL code is loaded during a build.
fn verify_export(dll: &Path, expected: &str) -> Result<(), String> {
    fn checked(bytes: &[u8], expected: &str) -> Option<bool> {
        fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
            Some(u16::from_le_bytes(
                bytes.get(at..at.checked_add(2)?)?.try_into().ok()?,
            ))
        }
        fn u32_at(bytes: &[u8], at: usize) -> Option<usize> {
            Some(u32::from_le_bytes(bytes.get(at..at.checked_add(4)?)?.try_into().ok()?) as usize)
        }
        if bytes.get(..2)? != b"MZ" {
            return None;
        }
        let pe = u32_at(bytes, 0x3c)?;
        if bytes.get(pe..pe.checked_add(4)?)? != b"PE\0\0"
            || u16_at(bytes, pe + 4)? != 0x8664
            || u16_at(bytes, pe + 22)? & 0x2000 == 0
        {
            return None;
        }
        let sections = u16_at(bytes, pe + 6)? as usize;
        let optional_size = u16_at(bytes, pe + 20)? as usize;
        let optional = pe + 24;
        if optional_size < 120 || u16_at(bytes, optional)? != 0x20b {
            return None;
        }
        let table = optional + optional_size;
        let locate = |rva: usize| -> Option<(usize, usize)> {
            for index in 0..sections {
                let section = table + 40 * index;
                let address = u32_at(bytes, section + 12)?;
                let size = u32_at(bytes, section + 16)?;
                if let Some(offset) = rva.checked_sub(address).filter(|offset| *offset < size) {
                    return Some((u32_at(bytes, section + 20)?.checked_add(offset)?, section));
                }
            }
            None
        };
        let map = |rva| locate(rva).map(|(offset, _)| offset);
        let export_rva = u32_at(bytes, optional + 112)?;
        let export_end = export_rva.checked_add(u32_at(bytes, optional + 116)?)?;
        let exports = map(export_rva)?;
        let names = u32_at(bytes, exports + 24)?;
        let addresses = map(u32_at(bytes, exports + 32)?)?;
        if names > bytes.len() / 4 {
            return None;
        }
        for index in 0..names {
            let start = map(u32_at(bytes, addresses + 4 * index)?)?;
            let tail = bytes.get(start..)?;
            if tail.starts_with(expected.as_bytes()) && tail.get(expected.len()) == Some(&0) {
                let ordinals = map(u32_at(bytes, exports + 36)?)?;
                let ordinal = u16_at(bytes, ordinals + 2 * index)? as usize;
                if ordinal >= u32_at(bytes, exports + 20)? {
                    return None;
                }
                let functions = map(u32_at(bytes, exports + 28)?)?;
                let function = u32_at(bytes, functions + 4 * ordinal)?;
                // Forwarders and data exports are outside this ABI.
                if function == 0 || (export_rva..export_end).contains(&function) {
                    return Some(false);
                }
                let (offset, section) = locate(function)?;
                return Some(
                    offset < bytes.len() && u32_at(bytes, section + 36)? & 0x2000_0000 != 0,
                );
            }
        }
        Some(false)
    }
    let bytes = fs::read(dll).map_err(|error| format!("CSX2502: cannot read DLL: {error}"))?;
    if checked(&bytes, expected) == Some(true) {
        return Ok(());
    }
    Err(format!(
        "CSX2502: {} does not export the matching Windows x64 CSharpX contract",
        dll.display()
    ))
}

pub(crate) fn emit_export(
    paths: &[&Path],
    implementation: &Path,
    borrowed: bool,
) -> Result<(String, Contract), String> {
    let tokens = language::lex(&crate::read_source(implementation)?)?;
    let (offset, imports) = language::function_imports(&tokens)?;
    if matches!(&tokens[offset..], [Tok::Ident(public), Tok::Ident(static_), Tok::Ident(result), ..] if public == "public" && static_ == "static" && result == "string")
    {
        return crate::abi_text::emit_export(paths, &tokens[offset..], imports, borrowed);
    }
    let [definition] = paths else {
        return Err("CSX2501: a DLL export project requires one configuration class source".into());
    };
    let source = crate::read_source(definition)?;
    let (class, fields) = crate::objects::abi_class(&source)?;
    let [Tok::Ident(public), Tok::Ident(static_), Tok::Ident(result), Tok::Ident(function), Tok::Sym('('), Tok::Ident(parameter_type), Tok::Ident(parameter), Tok::Sym(')'), Tok::Sym('{'), body @ .., Tok::Sym('}'), Tok::Eof] =
        tokens.as_slice()
    else {
        return Err("CSX2501: expected public static int or bool Function(Config value) { return expression; }".into());
    };
    if public != "public" || static_ != "static" || parameter_type != &class {
        return Err("CSX2501: export must be a public static function with the configuration class as parameter".into());
    }
    let result = Scalar::parse(result)?;
    let contract = Contract {
        class,
        function: function.clone(),
        result,
        fields,
        borrowed,
    };
    contract.validate()?;
    let getters = contract
        .fields
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| (name.clone(), (format!("get_{index}"), ty.ty())))
        .collect();
    let vars = BTreeMap::from([(
        parameter.clone(),
        Ty::Object(contract.class.clone(), getters),
    )]);
    let expr = export_expression(body, result, &vars)?;
    let mut code = String::from("#![deny(unsafe_code)]\n");
    if borrowed {
        code.push_str(&crate::abi_borrow::configuration(&contract));
        code.push_str("#[repr(C)] struct Reply { status: i64, value: i64 }\n");
    } else {
        code.push_str(&contract.layout());
        code.push_str("impl Payload {\n");
    }
    for (index, ty) in contract.fields.values().enumerate().filter(|_| !borrowed) {
        let access = if *ty == Scalar::Bool {
            format!("self.f{index} != 0")
        } else {
            format!("self.f{index}")
        };
        let rust_type = if *ty == Scalar::Bool { "bool" } else { "i64" };
        code.push_str(&format!(
            "fn get_{index}(&self) -> {rust_type} {{ {access} }}\n"
        ));
    }
    let validation = contract.fields.values().enumerate().filter(|(_, ty)| **ty == Scalar::Bool)
        .map(|(index, _)| format!("if !matches!(v_{parameter}.f{index}, 0 | 1) {{ return Reply {{ status: 1, value: 0 }}; }}"))
        .collect::<Vec<_>>().join("\n");
    let symbol = contract.symbol();
    if !borrowed {
        code.push_str("}\n");
    }
    let signature = if borrowed {
        "address: *const Configuration, witness: u64".to_owned()
    } else {
        format!("v_{parameter}: Payload")
    };
    let borrow = if borrowed {
        crate::abi_borrow::borrow_reference(parameter, "return Reply { status: 1, value: 0 };")
    } else {
        String::new()
    };
    code.push_str(&format!(
        r#"
#[allow(unsafe_code)]
#[export_name = "{symbol}"]
extern "C" fn csx_export({signature}) -> Reply {{
    {borrow}
    {validation}
    let result = std::panic::catch_unwind(|| -> Result<i64, String> {{ Ok(i64::from({expr})) }});
    match result {{
        Ok(Ok(value)) => Reply {{ status: 0, value }},
        _ => Reply {{ status: 1, value: 0 }},
    }}
}}
"#
    ));
    Ok((code, contract))
}

fn export_expression(
    body: &[Tok],
    result: Scalar,
    vars: &BTreeMap<String, Ty>,
) -> Result<String, String> {
    fn returning(
        tokens: &[Tok],
        result: Scalar,
        vars: &BTreeMap<String, Ty>,
    ) -> Result<String, String> {
        match tokens {
            [Tok::Ident(keyword), expr @ .., Tok::Sym(';')] if keyword == "return" => {
                language::emit_typed_expression(expr.to_vec(), result.ty(), vars)
            }
            _ => Err("CSX2501: expected return expression in DLL export".into()),
        }
    }
    if let [Tok::Ident(keyword), Tok::Sym('('), tail @ ..] = body {
        if keyword == "if" {
            let start = tail
                .iter()
                .position(|token| *token == Tok::Sym('{'))
                .ok_or("CSX2501: expected block after DLL export condition")?;
            let [condition @ .., Tok::Sym(')')] = &tail[..start] else {
                return Err("CSX2501: unclosed DLL export condition".into());
            };
            let tail = &tail[start + 1..];
            let end = tail
                .iter()
                .position(|token| *token == Tok::Sym('}'))
                .ok_or("CSX2501: unclosed DLL export branch")?;
            let condition = language::emit_typed_expression(condition.to_vec(), Ty::Bool, vars)?;
            let yes = returning(&tail[..end], result, vars)?;
            let no = returning(&tail[end + 1..], result, vars)?;
            return Ok(format!("if {condition} {{ {yes} }} else {{ {no} }}"));
        }
    }
    returning(body, result, vars)
}

pub(crate) fn install(dll: &Path, binary: &Path, contract: &Contract) -> Result<(), String> {
    let target: PathBuf =
        binary.with_file_name(dll.file_name().ok_or("CSX2502: invalid DLL path")?);
    if fs::canonicalize(dll).ok() != fs::canonicalize(&target).ok() {
        fs::copy(dll, &target)
            .map_err(|error| format!("CSX2502: cannot copy DLL beside application: {error}"))?;
    }
    verify_export(&target, &contract.symbol())
}
