# CSharpX – Arbeitsanweisungen für Codex

Lies zuerst README.md, SPECIFICATION.md, ARCHITECTURE.md, IMPLEMENTATION_STATUS.md und ROADMAP.md. Arbeite vom Verzeichnis mit Cargo.toml aus. Als Einstieg gilt P0 in ROADMAP.md.

- Trenne Zielanforderungen, per Quellinspektion vorhandene Funktionen und reproduzierte Ergebnisse. Bei Widersprüchen prüfe Quellcode und Tests. Aktualisiere den Implementierungsstand mit Befehl, Umgebung, Ergebnis und Blocker.
- Erhalte die vorhandenen CLI-Einstiegspunkte `check`, `emit-rust`, `build` und `ownership-report`. Belege neue Sprachregeln mit positiven und negativen Fällen. Prüfe bei Codegen-Änderungen auch das erzeugte Rust und mindestens einen tatsächlich ausgeführten Smoke-Test.
- Aus sicherem CSharpX darf kein Rust-`unsafe` entstehen. Erhalte `#![forbid(unsafe_code)]` im vorhandenen Backend. Änderungen am Vertrauensmodell für FFI oder CLR benötigen einen dokumentierten Entwurf und gezielte Tests.
- Prüfe bei LSP-Änderungen sowohl `tools/csx-language-server/` als auch die VSIX-Kopie. Halte beide synchron, bis ein gemeinsamer Paketierungsweg getestet ist.
- Behandle .NET-Restore, Metadatenimport, Symbolbindung und Laufzeitaufruf als getrennte Prüfschritte. Embedded- und Desktop-Fähigkeiten brauchen jeweils eigene Nachweise.
- Prüfe Werkzeugversionen vor dem Build. Führe Systeminstallationen, Änderungen an Visual Studio und die VSIX-Installation nur mit Freigabe aus. Wenn ein Werkzeug fehlt, dokumentiere den Fehler und führe unabhängige Checks aus.
- Aktualisiere bei Sprachänderungen Spezifikation, Beispiele und gegebenenfalls den KI-Index. Verwende für Experimente kleine, gezielte Änderungen und erhalte vorhandene Beispiele.
