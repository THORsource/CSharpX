#![forbid(unsafe_code)]
mod abi;
mod abi_text;
mod abi_borrow;
mod c_header;
mod diagnostics;
mod language;
mod managed;
mod managed_bundle;
mod objects;

use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

fn read_source(path: &Path) -> Result<String, String> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("csx") {
        return Err(format!("CSX9004: expected a .csx file: {}", path.display()));
    }
    fs::read_to_string(path).map_err(|error| format!("CSX9001: {}: {error}", path.display()))
}

#[derive(Default)]
struct BuildOptions<'a> {
    debug: bool,
    optimize: bool,
    library: bool,
    bundle_managed: bool,
    bundle_dotnet: bool,
    target: Option<&'a str>,
    linker: Option<&'a str>,
    trusted_c_file: Option<&'a str>,
    c_headers: Vec<&'a str>,
    cs_sources: Vec<&'a str>,
    rust_sources: Vec<&'a str>,
    startup: Option<&'a str>,
    abi_export: Option<&'a str>,
    abi_import: Option<&'a str>,
    abi_borrow: bool,
}

impl<'a> BuildOptions<'a> {
    fn parse(mut args: &'a [String]) -> Result<Self, String> {
        let mut options = Self::default();
        while let Some((flag, rest)) = args.split_first() {
            args = rest;
            match flag.as_str() {
                "--optimize" if !options.optimize => options.optimize = true,
                "--debug" if !options.debug => options.debug = true,
                "--bundle-dotnet" if !options.bundle_dotnet => options.bundle_dotnet = true,
                "--bundle-managed" if !options.bundle_managed => options.bundle_managed = true,
                "--abi-borrow" if !options.abi_borrow => options.abi_borrow = true,
                "--library" if !options.library => options.library = true,
                "--c-header" | "--cs-source" | "--rs-source" => {
                    let Some((value, remaining)) = args.split_first() else {
                        return Err(format!("CSX9006: missing value for {flag}"));
                    };
                    if value.is_empty() || value.starts_with("--") {
                        return Err(format!("CSX9006: invalid value for {flag}"));
                    }
                    match flag.as_str() {
                        "--c-header" => options.c_headers.push(value),
                        "--cs-source" => options.cs_sources.push(value),
                        _ => options.rust_sources.push(value),
                    }
                    args = remaining;
                }
                "--target" | "--linker" | "--trusted-c-file" | "--startup-object" | "--abi-export" | "--abi-import" => {
                    let slot = match flag.as_str() {
                        "--target" => &mut options.target,
                        "--linker" => &mut options.linker,
                        "--startup-object" => &mut options.startup,
                        "--abi-export" => &mut options.abi_export,
                        "--abi-import" => &mut options.abi_import,
                        _ => &mut options.trusted_c_file,
                    };
                    let Some((value, remaining)) = args.split_first() else {
                        return Err(format!("CSX9006: missing value for {flag}"));
                    };
                    if slot.is_some() || value.is_empty() || value.starts_with("--") {
                        return Err(format!("CSX9006: invalid or duplicate {flag}"));
                    }
                    *slot = Some(value);
                    args = remaining;
                }
                _ => {
                    return Err(format!(
                        "CSX9006: unsupported or duplicate build flag {flag}"
                    ))
                }
            }
        }
        Ok(options)
    }

    fn maps_pdb(&self) -> bool {
        cfg!(all(windows, target_env = "msvc"))
            && self.debug
            && self
                .target
                .is_none_or(|target| target.ends_with("-windows-msvc"))
    }
}

struct SourceDebug<'a> {
    path: &'a Path,
    lines: Vec<u32>,
}

fn build_diagnostic(output: &[u8], target: Option<&str>) -> String {
    let output = String::from_utf8_lossy(output);
    let target = match target {
        Some("x86_64-pc-windows-msvc") => "Windows (x64)",
        Some("x86_64-unknown-linux-gnu") => "Linux (x64)",
        Some("x86_64-apple-darwin") => "macOS (x64)",
        _ => "the selected platform",
    };
    if output.contains("E0463") || output.contains("target may not be installed") {
        format!("CSX9020: Build support for {target} is missing. Install the target toolchain or choose another target in project settings.")
    } else if output
        .lines()
        .any(|line| line.contains("linker") && line.contains("not found"))
    {
        format!("CSX9021: No linker is available for {target}. Install the target build tools or set Linker in project settings.")
    } else if output.contains("LNK1168") || output.contains("os error 32") {
        "CSX9022: The output file is in use. Stop the running application and build again.".into()
    } else if output.contains("linking with") || output.contains("linker command failed") {
        format!("CSX9023: Could not link the output for {target}. Check the target SDK, linker and required libraries.")
    } else if output.contains("failed to write")
        || output.contains("couldn't create")
        || output.contains("Permission denied")
    {
        "CSX9024: Could not write the build output. Check the output folder and its permissions."
            .into()
    } else {
        "CSX9003: Could not create the CSharpX output. Check the project settings or report the issue with a small CSharpX example.".into()
    }
}

fn build(
    rust: &str,
    source: &Path,
    binary: &Path,
    options: &BuildOptions<'_>,
    original: Option<SourceDebug<'_>>,
    typed_import: Option<(&Path, &abi::Contract)>,
) -> Result<(), String> {
    for path in [source, binary] {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
    }
    fs::write(source, rust).map_err(|error| error.to_string())?;
    let mut rustc = Command::new("rustc");
    rustc.args(["--edition=2021", "--crate-name", "csharpx_app"]);
    if let Some(target) = options.target {
        rustc.args(["--target", target]);
    }
    if let Some(linker) = options.linker {
        rustc.arg("-C").arg(format!("linker={linker}"));
    }
    if options.debug {
        rustc.args(["-C", "debuginfo=2", "-C", "opt-level=0"]);
    } else if options.optimize {
        rustc.args(["-C", "opt-level=3"]);
    }
    if let Some(original) = &original {
        rustc
            .arg("--remap-path-scope=debuginfo")
            .arg("--remap-path-prefix");
        rustc.arg(format!("{}={}", source.display(), original.path.display()));
    }
    let mut input = source.to_path_buf();
    let mut managed_library = false;
    let calls_c = rust.contains(language::C_FILE_CALL_PREFIX);
    if calls_c || rust.contains(language::C_HEADER_USAGE_PREFIX) {
        let mut bridge = c_header::make_bridge(rust, &options.c_headers, &options.cs_sources, &options.rust_sources)?;
        if calls_c {
            if !bridge.managed_calls.is_empty() {
                if options.library && !options.bundle_managed {
                    return Err("CSX1801: C# imports in a library require the single-file output option".into());
                }
                if options.bundle_dotnet && !options.bundle_managed {
                    return Err("CSX1803: bundling .NET requires the single-file output option".into());
                }
                if options.bundle_dotnet
                    && (!cfg!(all(windows, target_arch = "x86_64", target_env = "msvc"))
                        || options.target.is_some_and(|target| target != "x86_64-pc-windows-msvc"))
                {
                    return Err("CSX1803: bundled .NET output requires a Windows x64 host and target".into());
                }
                let payload = managed::build(source, binary, &bridge.managed_sources, &bridge.managed_calls, options.bundle_managed, options.bundle_dotnet)?;
                bridge.code.push_str(&payload);
                managed_library = options.library;
            }
            let library = build_usage_bridge(source, options, &bridge)?;
            rustc.arg("--extern").arg(format!("csx_c_file={}", library.display()));
        }
    }
    if let Some((dll, contract)) = typed_import {
        let shim = source.with_file_name("csx_typed_abi.rs");
        let library = source.with_file_name("libcsx_typed_abi.rlib");
        fs::write(&shim, contract.import_shim(dll)?).map_err(|error| error.to_string())?;
        let result = Command::new("rustc")
            .args(["--edition=2021", "--crate-name", "csx_typed_abi", "--crate-type", "rlib", "--target", "x86_64-pc-windows-msvc"])
            .arg(shim).arg("-o").arg(&library).output()
            .map_err(|_| "CSX2502: could not start the DLL adapter compiler")?;
        if !result.status.success() {
            return Err("CSX2502: could not compile the typed DLL adapter".into());
        }
        rustc.arg("--extern").arg(format!("csx_typed_abi={}", library.display()));
    }
    if options.library {
        rustc.args(["--cfg", "csharpx_library"]);
        if options.abi_export.is_some() {
            rustc.args(["--crate-type", "cdylib"]);
        } else if managed_library {
            rustc.args(["--crate-type", "cdylib"]);
            input = source.with_file_name("csx_managed_library.rs");
            let path = std::path::absolute(source).map_err(|error| error.to_string())?;
            let wrapper = format!(
                "#[path = {:?}] mod app;\n{}",
                path.to_string_lossy(),
                include_str!("managed_library.rs"),
            );
            fs::write(&input, wrapper).map_err(|error| error.to_string())?;
        } else {
            rustc.args(["--crate-type", "dylib", "-C", "prefer-dynamic"]);
        }
    }
    let output = rustc
        .arg(&input)
        .arg("-o")
        .arg(binary)
        .output()
        .map_err(|_| "CSX9002: CSharpX build tools could not be started. Check that the target toolchain is installed.")?;
    if !output.status.success() {
        return Err(build_diagnostic(&output.stderr, options.target));
    }
    if let Some(original) = original {
        map_debug_symbols(binary, source, original)?;
    }
    println!("Built: {}", binary.display());
    Ok(())
}

fn build_usage_bridge(source: &Path, options: &BuildOptions<'_>, plan: &c_header::Bridge) -> Result<PathBuf, String> {
    const TARGET: &str = "x86_64-pc-windows-msvc";
    let mut compiler = Command::new("rustc");
    if plan.native {
        let directory = options.trusted_c_file.ok_or(
            "CSX1702: C function calls require an explicitly trusted C file library"
        )?;
        if !cfg!(all(windows, target_arch = "x86_64", target_env = "msvc"))
            || options.target.is_some_and(|target| target != TARGET)
        {
            return Err("CSX1703: the C file bridge supports Windows x64 MSVC".into());
        }
        let native = Path::new(directory);
        if !native.join("csx_c_file_native.lib").is_file() {
            return Err("CSX1704: trusted C file library was not found".into());
        }
        compiler.arg("-L").arg(format!("native={}", native.display()));
    }
    let bridge = source.with_file_name("csx_usage_bridge.rs");
    let library = source.with_file_name("libcsx_c_file.rlib");
    fs::write(&bridge, &plan.code)
        .map_err(|error| format!("CSX1705: could not write usage bridge: {error}"))?;
    if let Some(target) = options.target {
        compiler.args(["--target", target]);
    }
    let output = compiler
        .args(["--edition=2021", "--crate-name", "csx_c_file", "--crate-type", "rlib"])
        .arg(&bridge).arg("-o").arg(&library)
        .output()
        .map_err(|_| "CSX1705: could not start the usage bridge build")?;
    if !output.status.success() {
        return Err("CSX1705: could not compile the usage bridge.".into());
    }
    Ok(library)
}

fn map_debug_symbols(
    binary: &Path,
    generated: &Path,
    original: SourceDebug<'_>,
) -> Result<(), String> {
    let mut mapper = Command::new("python")
        .args(["-c", include_str!("map_pdb.py")])
        .arg(binary.with_extension("pdb"))
        .arg(generated)
        .arg(original.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "CSX9013: Source-level debug symbols could not be created. Check that the CSharpX debug tools are installed.")?;
    let mut input = mapper.stdin.take().expect("mapper stdin is piped");
    let result = original
        .lines
        .iter()
        .try_for_each(|line| writeln!(input, "{line}"));
    drop(input);
    let status = mapper
        .wait()
        .map_err(|_| "CSX9013: Could not complete source-level debug symbols.")?;
    if !status.success() {
        return Err("CSX9013: Could not map debug symbols to the CSharpX source. Rebuild the project or try Release.".into());
    }
    result.map_err(|_| "CSX9013: Could not complete source-level debug symbols.".into())
}

fn uses_object_parser(source: &str) -> bool {
    use language::temporal::TemporalType;
    use language::Tok;

    if source.contains("namespace") && language::lex(source).is_ok_and(|tokens| {
        tokens.iter().any(|token| matches!(token, Tok::Ident(word) if word == "namespace"))
    }) {
        return true;
    }
    if !source.contains("class ") {
        return false;
    }
    if [" : ", "virtual ", "override ", "abstract "]
        .iter()
        .any(|marker| source.contains(marker))
    {
        return true;
    }
    language::lex(source).is_ok_and(|tokens| {
        tokens.windows(3).any(|pair| {
            matches!(pair, [Tok::Ident(keyword), Tok::Ident(name), next]
                if keyword == "new" && next != &Tok::Sym('[') && TemporalType::from_name(name).is_none())
        })
    })
}

fn compile(source: &str) -> Result<(String, Option<String>), String> {
    if uses_object_parser(source) {
        let (rust, report) = objects::compile(source)?;
        Ok((rust, Some(report)))
    } else {
        Ok((language::compile(source)?, None))
    }
}

fn build_project(
    inputs: &Path,
    generated: &Path,
    binary: &Path,
    options: &BuildOptions<'_>,
) -> Result<(), String> {
    let list = fs::read_to_string(inputs)
        .map_err(|error| format!("CSX9001: {}: {error}", inputs.display()))?;
    let paths: Vec<_> = list
        .trim_start_matches('\u{feff}')
        .lines()
        .map(Path::new)
        .collect();
    if (options.abi_export.is_some() || options.abi_import.is_some())
        && (!cfg!(all(windows, target_arch = "x86_64", target_env = "msvc"))
            || options.target.is_some_and(|target| target != "x86_64-pc-windows-msvc"))
    {
        return Err("CSX2500: typed DLL contracts require Windows x64 MSVC".into());
    }
    if let Some(implementation) = options.abi_export {
        if !options.library || options.abi_import.is_some() {
            return Err("CSX2500: ABI exports require a library project without an ABI import".into());
        }
        let implementation = Path::new(implementation);
        let (rust, contract) = abi::emit_export(&paths, implementation, options.abi_borrow)
            .map_err(|error| diagnostics::visual_studio(implementation, &error))?;
        build(&rust, generated, binary, options, None, None)?;
        return contract.write(binary);
    }
    if options.abi_import.is_some() && options.library {
        return Err("CSX2500: ABI imports currently require an executable project".into());
    }
    let contract = options.abi_import.map(|dll| abi::Contract::read(Path::new(dll))).transpose()?;
    if options.abi_borrow != contract.as_ref().is_some_and(|contract| contract.borrowed) {
        return Err("CSX2512: both projects must opt in to the same DLL borrowing mode".into());
    }
    let mut original = None;
    let rust = match paths.as_slice() {
        [] => return Err("CSX9005: project has no CSharpX sources".into()),
        [path] if options.startup.is_none() && contract.is_none() => {
            let source = read_source(path)?;
            if options.maps_pdb() && !uses_object_parser(&source) {
                let (rust, lines) = language::compile_with_lines(&source)
                    .map_err(|error| diagnostics::visual_studio(path, &error))?;
                original = Some(SourceDebug { path, lines });
                rust
            } else {
                compile(&source).map_err(|error| diagnostics::visual_studio(path, &error))?.0
            }
        }
        paths => {
            let mut tokens = Vec::new();
            for path in paths {
                let source = read_source(path)?;
                let (file_tokens, positions) = language::lex_with_lines(&source)
                    .map_err(|error| diagnostics::visual_studio(path, &error))?;
                let mut blocks = Vec::new();
                for (token, position) in file_tokens.into_iter().zip(positions) {
                    match token {
                        language::Tok::Sym('{') => blocks.push(position),
                        language::Tok::Sym('}') if blocks.pop().is_none() => {
                            return Err(diagnostics::visual_studio(path, &position.annotate(
                                "CSX2110: closing brace has no opening brace in this file",
                            )));
                        }
                        language::Tok::Eof => break,
                        _ => {}
                    }
                    tokens.push(token);
                }
                if let Some(position) = blocks.pop() {
                    return Err(diagnostics::visual_studio(path, &position.annotate(
                        "CSX2110: block must be closed in the same file",
                    )));
                }
                tokens.push(objects::FILE_BOUNDARY);
            }
            tokens.push(language::Tok::Eof);
            objects::compile_tokens_with_abi(tokens, options.startup, contract.as_ref())
                .map_err(|error| diagnostics::split_position(&error).1.to_owned())?.0
        }
    };
    let typed_import = options.abi_import.map(Path::new).zip(contract.as_ref())
        .filter(|(_, contract)| contract.borrowed || rust.contains("::csx_typed_abi::invoke("));
    build(&rust, generated, binary, options, original, typed_import)?;
    if let Some((dll, contract)) = typed_import {
        abi::install(dll, binary, contract)?;
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [command, inputs, generated, binary, options @ ..] if command == "build-project" => {
            let options = BuildOptions::parse(options)?;
            build_project(Path::new(inputs), Path::new(generated), Path::new(binary), &options)
        }
        [command, path] if matches!(command.as_str(), "check" | "emit-rust" | "build" | "ownership-report") => {
            let source_path = std::path::absolute(path).map_err(|error| error.to_string())?;
            let (rust, ownership) = compile(&read_source(&source_path)?)
                .map_err(|error| diagnostics::visual_studio(&source_path, &error))?;
            match command.as_str() {
                "check" => println!("CSharpX syntax and subset type check passed"),
                "emit-rust" => print!("{rust}"),
                "ownership-report" => match ownership {
                    Some(report) => print!("{report}"),
                    None => println!("No object allocations in the supported analysis subset."),
                },
                "build" => {
                    let out: PathBuf = env::current_dir().map_err(|error| error.to_string())?.join("csx-out");
                    let executable = if cfg!(windows) { "csx-app.exe" } else { "csx-app" };
                    build(&rust, &out.join("main.rs"), &out.join(executable), &BuildOptions::default(), None, None)?;
                }
                _ => unreachable!(),
            }
            Ok(())
        }
        _ => Err("Usage: csxc <check|emit-rust|build|ownership-report> <file.csx>\n       csxc build-project <inputs.txt> <generated.rs> <executable> [--debug] [--optimize] [--target <triple>] [--linker <path>] [--library] [--bundle-managed] [--bundle-dotnet] [--trusted-c-file <directory>] [--c-header <file.h>]... [--cs-source <file.cs>]... [--rs-source <file.rs>]... [--startup-object <class>] [--abi-borrow] [--abi-export <implementation.csx>] [--abi-import <library.dll>]".into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if error.starts_with("CSX") {
                eprintln!("error {error}");
            } else {
                eprintln!("{error}");
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod build_diagnostics_tests {
    use super::build_diagnostic;

    #[test]
    fn reports_missing_target_and_linker_with_csharpx_diagnostics() {
        let missing_target = build_diagnostic(
            b"error[E0463]: can't find crate for std",
            Some("x86_64-unknown-linux-gnu"),
        );
        assert!(missing_target.starts_with("CSX9020:") && missing_target.contains("Linux (x64)"));
        let missing_linker = build_diagnostic(
            b"error: linker link.exe not found",
            Some("x86_64-pc-windows-msvc"),
        );
        assert!(missing_linker.starts_with("CSX9021:") && missing_linker.contains("Windows (x64)"));
    }

    #[test]
    fn excludes_unrecognized_internal_error_details() {
        let diagnostic = build_diagnostic(b"rustc error in obj/Debug/main.rs", None);
        assert!(diagnostic.starts_with("CSX9003:"));
        assert!(!diagnostic.contains("rustc") && !diagnostic.contains("main.rs"));
    }
}
