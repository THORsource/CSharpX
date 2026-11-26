use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use crate::{
    abi::{identifier, Contract, Scalar},
    language::{self, CHeaderUsage, Tok, Ty},
};

const CAPACITY: usize = 16_384;

fn layout(contract: &Contract) -> String {
    let fields = (0..if contract.borrowed {
        0
    } else {
        contract.fields.len()
    })
        .map(|index| format!("f{index}: i64,"))
        .collect::<Vec<_>>()
        .join(" ");
    let config = if contract.borrowed {
        crate::abi_borrow::configuration(contract)
    } else {
        String::new()
    };
    format!(
        "{config}const CAP: usize = {CAPACITY};\n\
         #[repr(C)] struct Payload {{ {fields} len: u64, text: [u8; CAP] }}\n\
         #[repr(C)] struct Reply {{ status: u64, len: u64, text: [u8; CAP] }}\n"
    )
}

pub(crate) fn import_shim(contract: &Contract, name: &str, getters: &str) -> String {
    let count = contract.fields.len();
    let symbol = contract.symbol();
    let wire_parameters = if contract.borrowed {
        "address: *const Configuration, witness: u64, payload: Payload"
    } else {
        "payload: Payload"
    };
    let argument = if contract.borrowed {
        "config: &Configuration".to_owned()
    } else {
        format!("values: [i64; {count}]")
    };
    let payload_fields = if contract.borrowed {
        String::new()
    } else {
        format!("{getters}, ")
    };
    let pre_call = if contract.borrowed {
        "let address = std::ptr::from_ref(config);"
    } else {
        ""
    };
    let invocation = if contract.borrowed {
        "address, address.addr() as u64, payload"
    } else {
        "payload"
    };
    let body = format!(
        r#"
#[link(name = "{name}", kind = "raw-dylib")]
extern "C" {{
    #[link_name = "{symbol}"]
    fn invoke_dll({wire_parameters}) -> Reply;
}}

pub fn invoke({argument}, text: &str) -> Result<String, String> {{
    if text.len() > CAP {{
        return Err("CSharpX DLL input exceeds 16384 UTF-8 bytes".into());
    }}
    let mut payload = Payload {{ {payload_fields}len: text.len() as u64, text: [0; CAP] }};
    payload.text[..text.len()].copy_from_slice(text.as_bytes());
    // The export was checked against this exact versioned by-value ABI.
    {pre_call}
    let reply = unsafe {{ invoke_dll({invocation}) }};
    let length = usize::try_from(reply.len).map_err(|_| "invalid DLL response length")?;
    let bytes = reply.text.get(..length).ok_or("invalid DLL response length")?;
    match reply.status {{
        0 => String::from_utf8(bytes.to_vec()).map_err(|_| "invalid DLL UTF-8 response".into()),
        1 => Err(String::from_utf8(bytes.to_vec()).map_err(|_| "invalid DLL error text")?),
        2 => Err("CSharpX DLL response exceeds 16384 UTF-8 bytes".into()),
        _ => Err("invalid DLL response status".into()),
    }}
}}
"#
    );
    layout(contract) + &body
}

struct Helper {
    name: String,
    result: Scalar,
    parameters: Vec<(String, Scalar)>,
    body: Vec<Tok>,
    imports: Vec<CHeaderUsage>,
}

fn parse_helper(tokens: &[Tok], imports: Vec<CHeaderUsage>) -> Result<Helper, String> {
    let tokens = match tokens {
        [Tok::Ident(public), tail @ ..] if public == "public" => tail,
        _ => tokens,
    };
    let [Tok::Ident(static_), Tok::Ident(result), Tok::Ident(name), Tok::Sym('('), rest @ ..] =
        tokens
    else {
        return Err("CSX2511: expected a static typed helper function".into());
    };
    if static_ != "static" || !identifier(name) {
        return Err("CSX2511: invalid helper declaration".into());
    }
    let mut parameters = Vec::new();
    let mut names = BTreeSet::new();
    let mut tail = rest;
    let body = loop {
        match tail {
            [Tok::Sym(')'), remaining @ ..] if parameters.is_empty() => break remaining,
            [Tok::Ident(ty), Tok::Ident(name), Tok::Sym(separator @ (',' | ')')), remaining @ ..]
                if identifier(name) && names.insert(name) =>
            {
                parameters.push((name.clone(), Scalar::parse(ty)?));
                if *separator == ')' {
                    break remaining;
                }
                tail = remaining;
            }
            _ => return Err("CSX2511: invalid or duplicate helper parameter".into()),
        }
    };
    let [Tok::Sym('{'), .., Tok::Sym('}'), Tok::Eof] = body else {
        return Err("CSX2511: expected exactly one helper with a function body".into());
    };
    Ok(Helper {
        name: name.clone(),
        result: Scalar::parse(result)?,
        parameters,
        body: body[..body.len() - 1].to_vec(),
        imports,
    })
}

fn rust_type(ty: Scalar) -> &'static str {
    match ty {
        Scalar::Int => "i64",
        Scalar::Bool => "bool",
        Scalar::Text => "String",
    }
}

pub(crate) fn emit_export(
    paths: &[&Path],
    tokens: &[Tok],
    imports: Vec<CHeaderUsage>,
    borrowed: bool,
) -> Result<(String, Contract), String> {
    let [Tok::Ident(_), Tok::Ident(_), Tok::Ident(_), Tok::Ident(function), Tok::Sym('('), Tok::Ident(parameter_type), Tok::Ident(config), Tok::Sym(','), Tok::Ident(text_type), Tok::Ident(text), Tok::Sym(')'), body @ ..] =
        tokens
    else {
        return Err(
            "CSX2510: expected public static string Function(Config value, string text)".into(),
        );
    };
    if text_type != "string" || config == text {
        return Err("CSX2510: export needs distinct configuration and string parameters".into());
    }
    let [Tok::Sym('{'), .., Tok::Sym('}'), Tok::Eof] = body else {
        return Err("CSX2510: expected a single export function body".into());
    };
    let mut definition = None;
    let mut helpers = Vec::new();
    for path in paths {
        let source = crate::read_source(path)?;
        let tokens = language::lex(&source)?;
        let (start, imports) = language::function_imports(&tokens)?;
        if tokens[start..]
            .iter()
            .take(4)
            .any(|token| matches!(token, Tok::Ident(word) if word == "class"))
        {
            if definition
                .replace(crate::objects::abi_class(&source)?)
                .is_some()
            {
                return Err("CSX2510: expected one configuration class source".into());
            }
        } else {
            helpers.push(
                parse_helper(&tokens[start..], imports)
                    .map_err(|error| crate::diagnostics::visual_studio(path, &error))?,
            );
        }
    }
    let (class, fields) = definition.ok_or("CSX2510: missing configuration class")?;
    if &class != parameter_type {
        return Err("CSX2510: export parameter must use the configuration class".into());
    }
    let contract = Contract {
        class,
        function: function.clone(),
        result: Scalar::Text,
        fields,
        borrowed,
    };
    contract.validate()?;

    let mut signatures = BTreeMap::new();
    for helper in &helpers {
        let key = format!("\0fn:{}", helper.name);
        let params = helper.parameters.iter().map(|(_, ty)| ty.ty()).collect();
        if signatures
            .insert(key, Ty::LocalFunction(params, Box::new(helper.result.ty())))
            .is_some()
        {
            return Err(format!("CSX2511: duplicate helper {}", helper.name));
        }
    }

    let mut code = layout(&contract);
    if !borrowed {
        code.push_str("impl Payload {\n");
    }
    for (index, ty) in contract.fields.values().enumerate().filter(|_| !borrowed) {
        let read = if *ty == Scalar::Bool {
            format!("self.f{index} == 1")
        } else {
            format!("self.f{index}")
        };
        code.push_str(&format!(
            "fn get_{index}(&self) -> {} {{ {read} }}\n",
            rust_type(*ty)
        ));
    }
    if !borrowed {
        code.push_str("}\n");
    }

    for (index, helper) in helpers.into_iter().enumerate() {
        let mut vars = signatures.clone();
        let parameters = helper
            .parameters
            .iter()
            .map(|(name, ty)| {
                vars.insert(name.clone(), ty.ty());
                format!("mut v_{name}: {}", rust_type(*ty))
            })
            .collect::<Vec<_>>()
            .join(", ");
        code.push_str(&format!(
            "fn csx_helper_{}({parameters}) -> Result<{}, String> {{\n",
            helper.name,
            rust_type(helper.result)
        ));
        language::emit_function_body(
            helper.body,
            vars,
            helper.result.ty(),
            &helper.imports,
            &format!("csxAbi{index}_"),
            &mut code,
        )?;
        code.push_str(
            "Err(\"CSharpX helper reached the end without returning a value\".into())\n}\n",
        );
    }
    let getters = contract
        .fields
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| (name.clone(), (format!("get_{index}"), ty.ty())))
        .collect();
    signatures.insert(config.clone(), Ty::Object(contract.class.clone(), getters));
    signatures.insert(text.clone(), Ty::Str);
    let config_type = if borrowed {
        "&Configuration"
    } else {
        "Payload"
    };
    code.push_str(&format!("fn csx_export_body(v_{config}: {config_type}, mut v_{text}: String) -> Result<String, String> {{\n"));
    language::emit_function_body(
        body[..body.len() - 1].to_vec(),
        signatures,
        Ty::Str,
        &imports,
        "csxAbiExport_",
        &mut code,
    )?;
    code.push_str("Err(\"CSharpX export reached the end without returning a value\".into())\n}\n");

    let symbol = contract.symbol();
    let wire_parameters = if borrowed {
        "address: *const Configuration, witness: u64, payload: Payload"
    } else {
        "payload: Payload"
    };
    let borrow = if borrowed {
        crate::abi_borrow::borrow_reference(
            config,
            "return response(1, \"CSharpX DLL address verification failed\");",
        )
    } else {
        String::new()
    };
    let configuration = if borrowed {
        format!("v_{config}")
    } else {
        "payload".to_owned()
    };
    let invalid_bool = contract
        .fields
        .values()
        .enumerate()
        .filter(|(_, ty)| **ty == Scalar::Bool)
        .map(|(index, _)| format!(" || !matches!({configuration}.f{index}, 0 | 1)"))
        .collect::<Vec<_>>()
        .join("");
    code.push_str(&format!(
        r#"
fn response(status: u64, text: &str) -> Reply {{
    let mut reply = Reply {{ status, len: 0, text: [0; CAP] }};
    if text.len() > CAP {{
        reply.status = 2;
    }} else {{
        reply.len = text.len() as u64;
        reply.text[..text.len()].copy_from_slice(text.as_bytes());
    }}
    reply
}}

#[allow(unsafe_code)]
#[export_name = "{symbol}"]
extern "C" fn csx_export({wire_parameters}) -> Reply {{
    {borrow}
    let result = std::panic::catch_unwind(|| -> Result<String, String> {{
        if payload.len > CAP as u64 {invalid_bool} {{
            return Err("invalid CSharpX DLL request".into());
        }}
        let text = std::str::from_utf8(&payload.text[..payload.len as usize])
            .map_err(|_| "invalid DLL UTF-8 input")?.to_owned();
        csx_export_body({configuration}, text)
    }});
    match result {{
        Ok(Ok(text)) => response(0, &text),
        Ok(Err(error)) => response(1, &error),
        Err(_) => response(1, "CSharpX DLL panicked"),
    }}
}}
"#
    ));
    language::append_function_runtimes(&mut code);
    Ok(("#![deny(unsafe_code)]\n".to_owned() + &code, contract))
}
