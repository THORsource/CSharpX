# Implemented experimental inheritance / ownership subset

## Supported
- `class Name { ... }`, `class Derived : Base { ... }` with a single base class.
- `public virtual int M() { return N; }` and `public override int M() { return N; }`.
- Constant-returning integer methods, virtual dispatch according to runtime concrete type.
- `void Main()`, `new Class()`, typed upcasts, `var`, simple alias declarations, and `Console.WriteLine(object.Method())`.
- Inheritance cycle detection, missing base type, invalid override, duplicate methods, incompatible upcast, undefined variables.
- Whole-method allocation/alias counts: exclusively held objects use `Box`; objects with aliases use `Rc`. Safe tagged enum dispatch; generated source has `#![forbid(unsafe_code)]`.
- `csxc ownership-report <file.csx>` prints allocation classifications.

## Explicit limitations
- This is **not full C# inheritance**. No fields, constructors, properties, interfaces, generics, abstract classes, overloads, `base` calls, casts, member hiding, access control, object mutation, or runtime type tests.
- The object model is a safe enum and not an ABI-compatible C# object layout.
- Ownership analysis is conservative and restricted to straight-line local declarations. No reassignments, control-flow joins, interprocedural escape analysis, lifetime inference, borrow inference, threads, cycles or cycle collection.
- Aliased objects are reference-counted; `Rc` cycles would leak, but the current subset cannot construct cycles.
- An `Rc` handle cannot be used as a `Box` alias; analysis selects the allocation strategy before generating any local variable.
- The old statement compiler is retained for programs without object syntax. Object-mode programs do not support its `try/catch` syntax yet.
- Codegen supports only simple ASCII identifiers and does not yet validate all Rust reserved words. No production-grade parser recovery.

## Run
```
cargo test --workspace
cargo run -p csxc -- check examples/inheritance/Program.csx
cargo run -p csxc -- ownership-report examples/inheritance/Program.csx
cargo run -p csxc -- build examples/inheritance/Program.csx
```
Expected application output: `2` then `2`.
