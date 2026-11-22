# CSharpX grammar (normative draft for current compiler subset)

```ebnf
program = { using_directive }, [ "public" ], [ "static" ], [ "class", identifier, "{" ],
          [ "public" ], [ "static" ], "void", "Main", "(", ")", block, [ "}" ] ;
using_directive = "using", { token - ";" }, ";" ;
block = "{", { statement }, "}" ;
statement = ( ( "int" | "string" | "var" ), identifier, "=", expression, ";" )
          | ( identifier, "=", expression, ";" )
          | ( "Console.WriteLine", "(", expression, ")", ";" )
          | ( "throw", expression, ";" )
          | ( "try", block, [ "catch", [ "(", identifier, [ identifier ], ")" ], block ], [ "finally", block ] ) ;
expression = primary, { ( "+" | "-" | "*" | "/" ), primary } ;
primary = integer | string | identifier | "(", expression, ")" ;
```

The long-term C#-compatible grammar is **not** implemented by this subset grammar. Parser precedence: multiplication/division before addition/subtraction.
