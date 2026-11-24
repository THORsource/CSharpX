// The generated application module still forbids unsafe code.
// This entry point exposes Main under a stable C ABI without passing Rust values.
#[no_mangle]
pub extern "C" fn csx_run() -> i32 {
    match std::panic::catch_unwind(app::csx_main) {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            eprintln!("Unhandled CSharpX exception: {error}");
            1
        }
        Err(_) => 1,
    }
}
