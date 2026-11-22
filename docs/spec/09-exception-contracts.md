# Exception semantics — design contract

Target semantics: C#-style `throw`, typed `catch`, filters, rethrow, `finally`, `using`, exception inheritance, stack traces and async propagation. Explicit error edges in IR are the preferred lowering; Rust panics are not user exceptions.

Required invariants: `finally` runs on normal return and exceptional exit, including break/continue; exception raised in `finally` replaces pending exception; bare `throw;` retains the active exception identity; filters evaluate before selecting a handler; native exceptions are caught at FFI boundary; embedded profiles define exhaustion and fatal error behavior.

**Current prototype**: string-valued exceptions and simple `try/catch/finally` only. This is not full C# exception compatibility.
