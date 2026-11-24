# C / C++ interoperability (DRAFT)

## C

`[NativeImport("sensor.h", Library="sensor")]` binds functions with an explicit ABI and target. Header discovery requires the configured C preprocessor/Clang and target defines. `using native "sensor.h";` is optional proposed syntax.

## C++

Clang-based importer extracts supported declarations; generated C ABI shims handle overloads, methods, constructors/destructors and supported template instantiations. C++ ABI, exceptions, RTTI and compiler flags must match the target toolchain. Unsupported constructs are diagnosed.

## Safety

Untrusted native calls are unsafe even when their signatures have no pointers. Audited wrappers may expose safe interfaces with explicit ownership, pointer length, nullability, threading and lifetime contracts. C++ exceptions must be caught within a compatible native shim and translated to CSharpX exception transport.

## Build

Projects can include `.c`, `.cpp`, `.h`, `.hpp` sources and libraries, with explicit compile/link options per target. Native build failures retain source mapping.

## Implemented Windows x64 file bridge

In a scalar CSharpX source, using "name.h"; selects an included project header by filename. Only supported declarations from the headers selected in that source are imported under their original C names. An import without an alias makes its functions available as unqualified calls. using "name.h" AS Alias; or using "name.h" as Alias; requires Alias.function(...) for that import. Each alias is case-sensitive and must be unique within its source file. Unknown or ambiguous header filenames fail the project build. Pass the header via CSharpXCHeader in MSBuild or via build-project --c-header in the CLI. The CSharpX call takes an ASCII path without an embedded NUL and returns a UTF-8 string. The project must opt in with CSharpXTrustedCFileDirectory, or the CLI with build-project --trusted-c-file. This opts into running the native code supplied in csx_c_file_native.lib with the program's permissions. Only use a library built from reviewed sources.

The fixed C ABI is int name(const char *path, unsigned char *output, size_t capacity). Both pointers are borrowed for the duration of a single synchronous call. The caller supplies a NUL-terminated path and a writable 256-byte output buffer. The library retains neither pointer and writes at most capacity bytes. A nonnegative return is the number of bytes written, and a negative return reports failure. The wrapper checks the returned length and UTF-8 before producing a string. The one unsafe call is confined to that separately compiled wrapper. Generated application code continues to forbid unsafe code.

Calls to imported C and C# functions require a lexical unsafe block. A block grants permission to cross the external trust boundary and does not enable arbitrary unsafe backend code.
