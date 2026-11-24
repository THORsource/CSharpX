// Compiled separately from programs, which forbid unsafe code.
use std::ffi::{c_char, c_int, CString};

const CAPACITY: usize = 256;

type CFileFunction = unsafe extern "C" fn(*const c_char, *mut u8, usize) -> c_int;

fn round_trip(path: &str, function: CFileFunction) -> Result<String, String> {
    if !path.is_ascii() {
        return Err("C file access currently requires an ASCII path".into());
    }
    let path = CString::new(path).map_err(|_| "C file path contains a null byte")?;
    let mut output = [0u8; CAPACITY];
    // The synchronous C function borrows the terminated path and writes at
    // most CAPACITY bytes. It retains neither pointer and returns bytes read,
    // or a negative status. Only an explicitly trusted library may supply it.
    let length = unsafe { function(path.as_ptr(), output.as_mut_ptr(), output.len()) };
    let length = usize::try_from(length).map_err(|_| "C file round trip failed")?;
    let bytes = output.get(..length).ok_or("C file returned an invalid length")?;
    String::from_utf8(bytes.to_vec()).map_err(|_| "C file returned invalid UTF-8".into())
}
