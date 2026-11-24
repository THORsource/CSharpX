# Anforderungen und Sprachentwurf

- **Sprache:** C#-ähnliche Syntax mit Namespaces und `using`, Klassen, Vererbung, Interfaces, virtueller Dispatch und Generics. Spezifikationsdetails und vollständige C#-Kompatibilität sind funktionsweise zu definieren und mit Konformitätsfällen zu belegen.
- **Compiler:** Rust als Backend. Automatische Ownership- und Escape-Analyse für Stack- und Heap-Platzierung. Referenzzählung für geteilte Objekte und Cycle Collector für zyklische Objektgraphen. Lebensdauer, Aliasregeln, Mutation und Ressourcengrenzen explizit spezifizieren.
- **Sicherheit:** Rust-`unsafe` im generierten Code ist nur für ausdrücklich autorisierte CSharpX-`unsafe`-Konstrukte vorgesehen. Für FFI sind Vorbedingungen, Datenlayout, Lebensdauer und Vertrauensgrenzen zu dokumentieren.
- **Exceptions:** C#-kompatible Fehlerbehandlung als Ziel, einschließlich Typbindung, Auswahl des Catch-Zweigs und Finally-Verhalten. Zulässige Ausnahme- und Ressourcenmodelle auf Embedded-Zielen sind zu spezifizieren.
- **Interoperabilität:** Native C- und C++-Bibliotheken sowie bestehende .NET-DLLs und NuGet-Pakete. Ziel für .NET sind reale Aufrufe importierter Typen und Methoden über C#-artige Namespaces und `using`, einschließlich direkter DLL-Referenzen und `PackageReference`.
- **Embedded:** Bare Metal für ARM und RISC-V sowie AURIX/TriCore. Pro Ziel sind Toolchain, ABI, Linker, Startup, Speicherprofil und ausführbarer Smoke-Test festzulegen.
- **Werkzeuge:** Visual Studio 2022 mit VSIX, IntelliSense, Navigation, Debugging, Projekttyp und Build-Integration. Windows-BATCH-Dateien für reproduzierbaren Build und Installation fehlender SDK-Komponenten.
- **Dokumentation:** Vollständige Sprachreferenz und KI-lesbare Regeln mit Beispielen, Diagnostik, Versions- und Konformitätsangaben.

Die Detailentwürfe stehen in `docs/spec/00-status-and-conformance.md` bis `docs/spec/11-managed-dotnet-nuget.md`. `docs/ai/llms.txt` ist der bestehende KI-Einstieg. Beispiele und Entwürfe belegen die Zielsyntax. Der unterstützte Ist-Umfang folgt aus Einsicht in den Code und reproduzierten Tests.

## Skalare Kontrollfl?sse und Zeitmessung

Im skalaren Compilerpfad unterst?tzen `if`, optionales `else` und `while` jeweils Blockk?rper. Bedingungen sind boolesche Ausdr?cke, Ganzzahlvergleiche oder boolesche Gleichheitsvergleiche. `break` verl?sst die innerste Schleife im selben Ausnahmeblock. `Stopwatch.GetTimestamp()` liest eine monotone Uhr. `Stopwatch.Frequency` gibt deren Ticks pro Sekunde an. Zeitdauern ergeben sich aus der Differenz zweier Zeitstempel geteilt durch die Frequenz.

Aufrufe importierter C- und C#-Funktionen müssen in einem unsafe-Block stehen. Der Block hat einen eigenen Variablenbereich und markiert die Vertrauensgrenze. Eingebaute CSharpX-Aufrufe benötigen diesen Block nicht.

Im skalaren Sprachpfad sind int* und bool* in unsafe-Blöcken verfügbar. &variable nimmt die Adresse eines lokalen Werts. *pointer liest den Wert oder weist ihn über *pointer = wert zu. Die Gültigkeitsdauer des adressierten Werts liegt in der Verantwortung des aufrufenden Codes.

Rust-Quelldateien lassen sich in skalaren Projekten mit using und optionalem Alias importieren. Aufrufbar sind öffentliche Funktionen auf Modulebene mit der Signatur fn(&str) -> String. Die Aufrufe erfordern einen unsafe-Block.

Ein Klassenprojekt kann Instanztypen und ihre Basisklassen in getrennten CSX-Dateien deklarieren. Jede Datei muss ihre eigenen Blöcke schließen. Bei mehreren Klassen mit statischem Main wählt StartupObject den Einstiegspunkt anhand des Klassennamens.

Der skalare Compiler unterstützt Tupelliterale mit mindestens zwei Elementen, explizite Tupeltypen, benannte Elemente, ItemN-Zugriffe, Elementzuweisungen und Dekonstruktion mit var. Elementtypen sind int, string, bool und verschachtelte Tupel.
