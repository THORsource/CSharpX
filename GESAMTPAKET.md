# CSharpX – zusammengeführtes Gesamtprojekt

Dieses Paket führt alle bisherigen CSharpX-Archive in einer Projektstruktur zusammen. Bei Überschneidungen hat die jeweils neuere Entwicklungsstufe Vorrang; die zuletzt erstellte DLL-/NuGet-Integration ist maßgeblich. Die zusätzliche Dokumentation `docs/CSHARP-USINGS.md` bleibt erhalten.

## Einstieg unter Windows

1. `Setup-SDK.bat` – Voraussetzungen prüfen/installieren (vorher Skript und Installationsquellen prüfen).
2. `Build-All.bat` – Compiler und VSIX-Quellprojekt bauen.
3. `Install-VSIX.bat` – nur nach erfolgreichem VSIX-Build.
4. `Restore-NuGet.bat examples\nuget\NuGetDemo.csxproj` – NuGet-Restore und Assembly-Inventarisierung.

Siehe `README.md`, `BUILD-WINDOWS.md` und `ROADMAP.md`.

## Implementierungsgrenzen

Das ist ein experimentelles Entwicklungsprojekt, keine fertig validierte CSharpX-Toolchain. Die semantische IntelliSense-Analyse ist ein Teilmengen-Prototyp. Native .NET-DLL-Ausführung, vollständige C#-Semantik, Cycle Collector, vollständige C/C++-Bindings, Embedded-Backends und ein produktionsreifes VSIX sind noch nicht implementiert. Die Rust- und VSIX-Builds wurden in dieser Umgebung nicht bestätigt.
