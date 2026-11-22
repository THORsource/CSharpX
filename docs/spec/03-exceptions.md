# Exception semantics (DRAFT)

CSharpX exposes C#-style `try`, ordered typed `catch`, `when` filters, `finally`, `throw expr`, `throw;`, exception inheritance, `InnerException`, stack traces and `using`/`IDisposable` cleanup.

- `finally` executes on normal return, `break`, `continue`, and exception propagation.
- `throw;` rethrows the current exception while preserving its original throw context.
- Catch selection and filter evaluation MUST follow defined C#-compatible order.
- Exceptions raised by cleanup can replace an in-flight exception according to specified precedence.
- `await` propagates stored exceptions at await points; native exceptions must not unwind across unsupported ABI boundaries.
- Runtime faults such as bounds violations, null dereferences and division by zero map to documented CSharpX exceptions where specified. Rust panic is NOT a CSharpX exception.
- Compiler MAY lower exception paths to an internal tagged result, but this representation is not visible in CSharpX source or ABI.
- Heapless targets must define pool exhaustion and fatal-fault policy. `[NoThrow]` rejects possible transitive throws.

## Open design questions before conformance

Exact filter failure semantics, asynchronous cancellation, stack trace capture, exception allocation failure, and interactions with fatal hardware faults require executable conformance tests.
