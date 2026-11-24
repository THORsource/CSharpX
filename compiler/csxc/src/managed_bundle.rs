use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) fn declaration(output: &Path, runtime: bool) -> io::Result<String> {
    let mut files = Vec::new();
    for name in [
        "CSharpX.Managed.dll",
        "CSharpX.Managed.deps.json",
        "CSharpX.Managed.runtimeconfig.json",
        "CSharpX.Imported.dll",
    ] {
        files.push((name.to_owned(), output.join(name)));
    }
    if runtime {
        runtime_files(&mut files)?;
    }
    let mut code = format!(
        "\nconst BUNDLED_DOTNET: bool = {runtime};\nconst MANAGED_PAYLOAD: &[(&str, &[u8])] = &[\n"
    );
    for (name, path) in files {
        if !path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                path.display().to_string(),
            ));
        }
        code.push_str(&format!(
            "    ({name:?}, include_bytes!({:?})),\n",
            path.to_string_lossy()
        ));
    }
    code.push_str("];\n");
    Ok(code)
}

fn runtime_files(files: &mut Vec<(String, PathBuf)>) -> io::Result<()> {
    let found = Command::new("dotnet").arg("--list-runtimes").output()?;
    let versions = String::from_utf8_lossy(&found.stdout);
    let (version, framework) = versions
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("Microsoft.NETCore.App ")?;
            let (version, folder) = rest.split_once(" [")?;
            let patch = version.strip_prefix("9.0.")?.parse::<u32>().ok()?;
            Some((patch, version, Path::new(folder.strip_suffix(']')?)))
        })
        .max_by_key(|(patch, _, _)| *patch)
        .map(|(_, version, framework)| (version, framework))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "a .NET 9 runtime installation is required on the build machine",
            )
        })?;
    let installation = framework.parent().and_then(Path::parent).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "invalid .NET runtime directory")
    })?;
    for name in ["dotnet.exe", "LICENSE.txt", "ThirdPartyNotices.txt"] {
        files.push((format!("dotnet/{name}"), installation.join(name)));
    }
    let host = format!("host/fxr/{version}/hostfxr.dll");
    files.push((format!("dotnet/{host}"), installation.join(host)));
    let framework_dir = framework.join(version);
    let mut entries = fs::read_dir(framework_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry.file_type()?.is_file() {
            let name = entry.file_name();
            files.push((
                format!(
                    "dotnet/shared/Microsoft.NETCore.App/{version}/{}",
                    name.to_string_lossy()
                ),
                entry.path(),
            ));
        }
    }
    Ok(())
}
