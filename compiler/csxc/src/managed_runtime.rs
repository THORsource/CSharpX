fn managed_call(index: usize, value: &str) -> Result<String, String> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };

    let extracted = if MANAGED_PAYLOAD.is_empty() {
        None
    } else {
        Some(
            ManagedExtraction::create()
                .map_err(|error| format!("could not extract C# files: {error}"))?,
        )
    };
    let folder = match &extracted {
        Some(files) => files.0.clone(),
        None => std::env::current_exe()
            .map_err(|error| error.to_string())?
            .with_extension("managed"),
    };
    let mut command = if BUNDLED_DOTNET {
        let dotnet = folder.join("dotnet");
        let mut command = Command::new(dotnet.join("dotnet.exe"));
        command.env("DOTNET_ROOT", dotnet);
        command
    } else {
        Command::new("dotnet")
    };
    let mut child = command
        .arg(folder.join("CSharpX.Managed.dll"))
        .arg(index.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start the .NET runtime: {error}"))?;
    let sent = child
        .stdin
        .take()
        .ok_or("managed input pipe missing")?
        .write_all(value.as_bytes());
    let result = child
        .wait_with_output()
        .map_err(|error| error.to_string())?;
    if !result.status.success() {
        return Err("C# function failed".into());
    }
    sent.map_err(|error| error.to_string())?;
    String::from_utf8(result.stdout).map_err(|_| "C# function returned invalid UTF-8".into())
}

struct ManagedExtraction(std::path::PathBuf);

impl ManagedExtraction {
    fn create() -> std::io::Result<Self> {
        use std::{
            fs, io,
            sync::atomic::{AtomicU64, Ordering},
            time::{SystemTime, UNIX_EPOCH},
        };
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let base = std::env::temp_dir();
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        loop {
            let index = SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let dir = base.join(format!("csharpx-{}-{time:x}-{index:x}", std::process::id()));
            match builder.create(&dir) {
                Ok(()) => {
                    let extraction = Self(dir);
                    for (name, bytes) in MANAGED_PAYLOAD {
                        let path = extraction.0.join(name);
                        if let Some(parent) = path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::write(path, bytes)?;
                    }
                    return Ok(extraction);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }
}

impl Drop for ManagedExtraction {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
