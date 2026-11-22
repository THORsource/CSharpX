# C / C++ interoperability (DRAFT)

## C

`[NativeImport("sensor.h", Library="sensor")]` binds functions with an explicit ABI and target. Header discovery requires the configured C preprocessor/Clang and target defines. `using native "sensor.h";` is optional proposed syntax.

## C++

Clang-based importer extracts supported declarations; generated C ABI shims handle overloads, methods, constructors/destructors and supported template instantiations. C++ ABI, exceptions, RTTI and compiler flags must match the target toolchain. Unsupported constructs are diagnosed.

## Safety

Untrusted native calls are unsafe even when their signatures have no pointers. Audited wrappers may expose safe interfaces with explicit ownership, pointer length, nullability, threading and lifetime contracts. C++ exceptions must be caught within a compatible native shim and translated to CSharpX exception transport.

## Build

Projects can include `.c`, `.cpp`, `.h`, `.hpp` sources and libraries, with explicit compile/link options per target. Native build failures retain source mapping.
