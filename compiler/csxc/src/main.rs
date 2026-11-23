#![forbid(unsafe_code)]
mod diagnostics;
mod language;
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
    target: Option<&'a str>,
    linker: Option<&'a str>,
}

impl<'a> BuildOptions<'a> {
    fn parse(mut args: &'a [String]) -> Result<Self, String> {
        let mut options = Self::default();
        while let Some((flag, rest)) = args.split_first() {
            args = rest;
            match flag.as_str() {
                "--optimize" if !options.optimize => options.optimize = true,
                "--debug" if !options.debug => options.debug = true,
                "--library" if !options.library => options.library = true,
                "--target" | "--linker" => {
                    let slot = if flag == "--target" {
                        &mut options.target
                    } else {
                        &mut options.linker
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
    if options.library {
        rustc.args([
            "--crate-type",
            "dylib",
            "--cfg",
            "csharpx_library",
            "-C",
            "prefer-dynamic",
        ]);
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
    let output = rustc
        .arg(source)
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
    let mut original = None;
    let rust = match paths.as_slice() {
        [] => return Err("CSX9005: project has no CSharpX sources".into()),
        [path] => {
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
                let mut file_tokens = language::lex(&source)
                    .map_err(|error| diagnostics::visual_studio(path, &error))?;
                file_tokens.pop(); // Each lexer invocation ends in an EOF token.
                tokens.extend(file_tokens);
            }
            tokens.push(language::Tok::Eof);
            objects::compile_tokens(tokens)?.0
        }
    };
    build(&rust, generated, binary, options, original)
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
                    build(&rust, &out.join("main.rs"), &out.join(executable), &BuildOptions::default(), None)?;
                }
                _ => unreachable!(),
            }
            Ok(())
        }
        _ => Err("Usage: csxc <check|emit-rust|build|ownership-report> <file.csx>\n       csxc build-project <inputs.txt> <generated.rs> <executable> [--debug] [--optimize] [--target <triple>] [--linker <path>] [--library]".into()),
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
