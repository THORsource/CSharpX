use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn one_catch_site_handles_different_errors() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = std::env::temp_dir().join(format!("csxc-catch-{}-{nonce}", std::process::id()));
    fs::create_dir(&work).unwrap();
    let source = work.join("Errors.csx");
    fs::write(
        &source,
        r#"
        void Main() {
            int attempt = 0;
            while (attempt < 2) {
                try {
                    if (attempt == 0) { throw "sensor failure"; }
                    int quotient = 1 / 0;
                } catch (Exception error) {
                    Console.WriteLine(error);
                }
                attempt = attempt + 1;
            }
        }
    "#,
    )
    .unwrap();

    let build = Command::new(env!("CARGO_BIN_EXE_csxc"))
        .current_dir(&work)
        .arg("build")
        .arg(&source)
        .output()
        .unwrap();
    let run = build.status.success().then(|| {
        let executable = format!("csx-app{}", std::env::consts::EXE_SUFFIX);
        Command::new(work.join("csx-out").join(executable))
            .output()
            .unwrap()
    });
    fs::remove_dir_all(&work).unwrap();

    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let run = run.unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8(run.stdout)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        ["sensor failure", "DivideByZeroException or overflow"],
    );
}
