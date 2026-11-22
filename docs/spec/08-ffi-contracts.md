# Native ABI contracts (design specification, not implemented)

Every import records: header, symbol, calling convention, target triple, native library, integer widths, packing/alignment, ownership of pointer parameters, nullability, length relationships, error translation, thread/interrupt safety, exception boundary and library version.

C++ imports must use a compatible ABI or generated C wrappers. No C++ exception may cross an unspecified ABI boundary. C++ template instantiations must be explicitly materialized for concrete arguments.

Untrusted native functions require explicit `unsafe` at the CSharpX callsite. Safe wrappers must be audited and registered as trusted package boundaries. Generated application code without explicit `unsafe` is compiled with `#![forbid(unsafe_code)]`; this does not establish that linked C/C++ libraries or the runtime are memory safe.

## Planned diagnostics
- CSX6101: unsupported native declaration
- CSX6102: missing ABI/calling convention
- CSX6103: unsafe native call outside unsafe context
- CSX6104: C++ exception crosses native boundary
- CSX6105: missing ownership/length contract
