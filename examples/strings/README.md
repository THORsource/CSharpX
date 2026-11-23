# String-Beispiel

Öffne Strings.csxproj in Visual Studio 2022 und baue die Konfiguration Debug. Starte anschließend mit F5.

string, String und System.String bezeichnen denselben eingebauten Texttyp. Das Beispiel deckt Length, Substring, Trim, Replace, IndexOf, Contains, StartsWith, EndsWith, Equals, ToUpperInvariant, ToLowerInvariant, String.Empty, Concat, IsNullOrEmpty und IsNullOrWhiteSpace ab. + kann Strings mit Strings, Ganzzahlen oder booleschen Werten verbinden.

Length, IndexOf und Substring zählen UTF-16-Codeeinheiten. Werte werden intern als gültiges UTF-8 gespeichert. Teilt ein Substring ein Surrogatpaar, meldet CSharpX CSX1505. String-Suchen vergleichen ordinal. Die Unicode-Schreibweisen richten sich nach der Rust-Implementierung. Lokalisierte Vergleiche, alle .NET-Überladungen und nullable String-Variablen sind noch offen. Die statischen String-Methoden akzeptieren das null-Literal in ihren unterstützten Argumenten.

Die 22 Ausgabezeilen des Beispiels stimmten unter Windows bytegenau mit der C#-Referenz unter .NET 9.0.302 überein.

## Verbatim und Interpolation

- @"C:\Temp\Reports" behält Backslashes und Zeilenumbrüche im Text. Schreibe "" für ein Anführungszeichen.
- $"Label: {label}" setzt den Wert eines unterstützten Ausdrucks ein. {{ und }} schreiben sichtbare Klammern.
- $@"{folder}\{label}.txt" und @$"Name: ""{label}""" kombinieren beide Eigenschaften.

Alignment, Formatangaben direkt im Interpolationsplatzhalter und Raw-String-Literale bleiben offen. Explizite Formatmethoden wie date.ToString("yyyy-MM-dd") können als Ausdruck in einem Platzhalter verwendet werden.
