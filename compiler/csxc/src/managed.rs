use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

fn xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

pub(crate) fn build(
    generated: &Path,
    binary: &Path,
    sources: &BTreeSet<PathBuf>,
    calls: &[(String, String)],
    bundle: bool,
    runtime: bool,
) -> Result<String, String> {
    write_and_build(generated, binary, sources, calls, bundle, runtime)
        .map_err(|error| format!("CSX1802: managed C# build failed: {error}"))?
}

fn write_and_build(
    generated: &Path,
    binary: &Path,
    sources: &BTreeSet<PathBuf>,
    calls: &[(String, String)],
    bundle: bool,
    runtime: bool,
) -> io::Result<Result<String, String>> {
    let folder = std::path::absolute(generated.with_file_name("managed-usage"))?;
    let output = if bundle {
        folder.join("payload")
    } else {
        std::path::absolute(binary.with_extension("managed"))?
    };
    let imported = folder.join("imported");
    fs::create_dir_all(&imported)?;
    fs::write(
        folder.join("global.json"),
        r#"{"sdk":{"version":"9.0.100","rollForward":"latestFeature"}}"#,
    )?;
    let mut project = String::from(
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net9.0</TargetFramework>
    <AssemblyName>CSharpX.Imported</AssemblyName>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
  </PropertyGroup>
  <ItemGroup>
"#,
    );
    for source in sources {
        project.push_str(&format!(
            "    <Compile Include=\"{}\" />\n",
            xml(&std::path::absolute(source)?.to_string_lossy())
        ));
    }
    project.push_str("  </ItemGroup>\n</Project>\n");
    fs::write(imported.join("Imported.csproj"), project)?;
    fs::write(
        folder.join("Bridge.csproj"),
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net9.0</TargetFramework>
    <AssemblyName>CSharpX.Managed</AssemblyName>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
    <UseAppHost>false</UseAppHost>
  </PropertyGroup>
  <ItemGroup>
    <Compile Include="Bridge.cs" />
    <ProjectReference Include="imported/Imported.csproj" />
  </ItemGroup>
</Project>
"#,
    )?;
    fs::write(
        folder.join("NuGet.Config"),
        "<configuration><packageSources><clear /></packageSources></configuration>",
    )?;
    let mut runner = String::from(
        r#"using System;
using System.Text;

internal static class CSharpXManagedEntry
{
    static int Main(string[] args)
    {
        if (args.Length != 1 || !int.TryParse(args[0], out int index))
            return 2;
        Console.InputEncoding = Encoding.UTF8;
        Console.OutputEncoding = new UTF8Encoding(false);
        var response = Console.Out;
        Console.SetOut(Console.Error);
        Func<string, string> call = index switch
        {
"#,
    );
    for (index, (class, method)) in calls.iter().enumerate() {
        runner.push_str(&format!(
            "            {index} => global::@{class}.@{method},\n"
        ));
    }
    runner.push_str(
        r#"            _ => throw new ArgumentOutOfRangeException(nameof(index)),
        };
        string result = call(Console.In.ReadToEnd());
        if (result is null)
            return 3;
        response.Write(result);
        return 0;
    }
}
"#,
    );
    fs::write(folder.join("Bridge.cs"), runner)?;
    let result = Command::new("dotnet")
        .current_dir(&folder)
        .args(["build", "Bridge.csproj", "--nologo", "-c", "Release", "-o"])
        .arg(&output)
        .output()?;
    if result.status.success() {
        let declaration = if bundle {
            match crate::managed_bundle::declaration(&output, runtime) {
                Ok(code) => code,
                Err(error) => return Ok(Err(format!("CSX1803: cannot bundle .NET: {error}"))),
            }
        } else {
            "const MANAGED_PAYLOAD: &[(&str, &[u8])] = &[];\nconst BUNDLED_DOTNET: bool = false;"
                .into()
        };
        return Ok(Ok(declaration));
    }
    let diagnostics = String::from_utf8_lossy(&result.stdout);
    let details = diagnostics
        .lines()
        .find(|line| line.contains(": error "))
        .map(|line| line.split(" [").next().unwrap_or(line).trim())
        .unwrap_or("check that the .NET 9 SDK and reference packs are installed");
    Ok(Err(format!("CSX1802: C# import failed: {details}")))
}
