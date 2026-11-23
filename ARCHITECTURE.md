# Architektur

## Vorhandener Quellaufbau

| Bereich | Pfad und Datenfluss |
|---|---|
| CLI | `compiler/csxc/src/main.rs` liest eine .csx-Datei, wählt anhand von Textmerkmalen zwischen zwei Compilierpfaden und gibt Rust oder einen Prüfbericht aus. Build schreibt `csx-out/main.rs` und ruft `rustc` für das Hostsystem auf. |
| Sprachkern | `language.rs` implementiert Lexer, Parser und Rust-Ausgabe für Ausdrücke und Anweisungen einschließlich einfacher String-Exceptions. |
| Objekte | `objects.rs` verwendet einen eigenen Parser, ein Rust-Enum für Objekte, virtuelle Dispatch-Logik und eine einfache Box/Rc-Klassifikation. |
| Editor | `tools/csx-language-server/` enthält Python-LSP und Symbolindex. VSIX enthält Kopien von server.py und semantic.py. VSIX, Projektvorlagen und zwei MSBuild-Pfade liegen unter `ide/visual-studio-2022/`. |
| .NET-Referenzen | `tools/csx-dotnet/bridge.py` liest XML-Referenzen, erzeugt bei Bedarf ein Restore-Projekt und schreibt eine JSON-Dateiliste aus NuGet-Assets und HintPaths. |
| Plattformen | `targets/` enthält JSON-Schema und Beispielbeschreibungen, `interop/` C/C++-Entwürfe und Beispiele. `runtime/` und `stdlib/` enthalten README-Dateien. |

## Zielarchitektur

Dateien und Projektauflösung → einheitlicher Lexer und Parser mit Quellspannen → gemeinsame Symbol- und Typauflösung → typisierte Zwischenrepräsentation → Ownership-, Escape- und Effektanalyse → Rust-Codegen und Ziel-Runtime. Der Editor verwendet langfristig dieselbe Semantik. Die Projektauflösung liefert Quelldateien, Zielparameter und Referenzen. Dabei müssen native ABI-Bindings und managed Referenzen über explizite Schnittstellen eingebunden werden.

Für .NET folgen auf die vorhandene Inventarisierung ein Metadatenleser, Compiler- und LSP-Symbolimport und ein eigener Ausführungspfad für unterstützte Desktop-Ziele. CLR-Hosting bzw. eine kompatible Managed-Strategie bleibt eine Architekturentscheidung. Erforderlich sind Regeln für Typabbildung, Ressourcenbesitz, Fehler, Assemblyauflösung und Laufzeitabhängigkeiten. Für Bare Metal sind Ziel-Toolchain, Startcode, Linker und gegebenenfalls Allocator und HAL gesondert auszulegen. Die vorhandenen JSON-Dateien dienen als Anforderungen für diese Implementierung.
