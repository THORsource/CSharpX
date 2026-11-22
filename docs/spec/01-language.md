# Lexical, type and object model (DRAFT)

## Syntax

Files end in `.csx`. Keywords include `namespace`, `using`, `class`, `interface`, `struct`, `enum`, `public`, `private`, `protected`, `internal`, `virtual`, `override`, `abstract`, `sealed`, `base`, `new`, `static`, `readonly`, `const`, `unsafe`, `try`, `catch`, `finally`, `throw`, `when`, `async`, `await`, `using`.

## Type system

Value types have value semantics; class instances have identity and reference semantics. Single class inheritance and multiple interface implementation are supported. A class reference assignment MUST preserve identity, not clone the object. `null` and nullable types require precise null-check rules. Generics must support constraint checking and inherited member resolution.

## Dispatch

Virtual dispatch uses runtime type. Upcasts preserve identity. Downcasts are checked and produce CSharpX exceptions when invalid. Object layout and handle representation are not observable by ordinary source code.

## Target restrictions

The same semantics hold across targets. Heapless targets may reject programs whose required lifetime/identity graph cannot be represented with their configured static/pool resources.
