# Build und Tests

Alle Befehle laufen aus dem CSharpX-Projektordner. Prüfe zuerst `cargo --version`, `rustc --version`, `python --version` und `dotnet --list-sdks`. Auf Windows braucht das Rust-MSVC-Ziel einen funktionierenden MSVC-Linker und ein Windows SDK. Für VSIX werden Visual Studio 2022 mit Erweiterungsentwicklungs-Workload, die .NET-Framework-4.7.2-Referenzassemblies und gegebenenfalls NuGet-Zugriff benötigt. Für das bestehende Restore-Beispiel müssen die net8.0-Referenzpakete auflösbar sein.

## Compiler und Editor-Smoke unter Windows

```powershell
cargo test --locked --workspace
cargo run --locked -p csxc -- check examples/expressions/Program.csx
cargo run --locked -p csxc -- emit-rust examples/expressions/Program.csx
cargo run --locked -p csxc -- build examples/expressions/Program.csx
.\csx-out\csx-app.exe
cargo run --locked -p csxc -- ownership-report examples/inheritance/Program.csx
cargo run --locked -p csxc -- build examples/inheritance/Program.csx
.\csx-out\csx-app.exe
python -m unittest discover -s tools/csx-language-server -v
```

Prüfe nach jedem Befehl den Exitcode. Führe die jeweilige EXE nur nach erfolgreichem Build aus, da beide Beispiele denselben Ausgabepfad benutzen. Erwartet werden für expressions die Zeilen `6`, `sensor failure`, `cleanup`, für inheritance zweimal `2`. Der Ownership-Bericht soll einen exklusiven und einen geteilten Fall zeigen. Auf Unix bietet `sh tests/run-smoke.sh` einen Teil dieser Prüfung. Die Sollausgaben sind aus den Beispielen abgeleitet.

## Windows-Paketierung

`Setup-SDK.bat` versucht fehlende Werkzeuge über winget zu installieren und kann die Workload einer vorhandenen VS-2022-Installation ergänzen. Prüfe danach in einem neuen Terminal die echten Versionen. `Build-Compiler.bat` baut Release, `Build-VSIX.bat` baut die VSIX, `Build-All.bat` startet beide Builds. Prüfe nach erfolgreichem VSIX-Build, ob die Datei im Release-Ordner liegt. `Install-VSIX.bat` ist der anschließende Installationsschritt. Details stehen in `BUILD-WINDOWS.md`.

## NuGet und DLL-Inventar

```powershell
.\Restore-NuGet.bat examples\nuget\NuGetDemo.csxproj
```

Der Live-Restore benötigt Zugriff auf die konfigurierten Paketquellen oder einen geeigneten Paketcache. Prüfe danach `examples/nuget/obj/csharpx-dotnet-references.json` auf Newtonsoft.Json Compile-/Runtime-Einträge. Für ein bereits vorhandenes Assets-File kann der Bridge-Aufruf mit `--offline-assets <Pfad-zu-project.assets.json>` ausgeführt werden. Das Manifest ist ein Inventar. Der Beispielaufruf von JsonConvert ist ein späterer Interop-Akzeptanztest.

## Native CSharpX-Projekte in Visual Studio 2022

Installiere die VSIX 0.2.1 und öffne danach über **Datei → Öffnen → Projekt/Projektmappe** die Datei examples\hello\Hello.csxproj. **Erstellen → Projektmappe erstellen** verwendet das MSBuild-Ziel Build. Der Compiler aus target\release muss bereits gebaut sein und rustc muss für Visual Studio im PATH liegen. Das Programm liegt unter examples\hello\bin\Debug\Hello.exe bzw. bin\Release\Hello.exe.

Die VSIX enthält die CPS-Registrierung für .csxproj und eine CSharpX-Console-Vorlage. Für ein Projekt aus der Vorlage wird der vollständige Compilerpfad über die Eigenschaft CSharpXCompiler (oder eine gleichnamige Umgebungsvariable vor dem Start von Visual Studio) gesetzt. Die Vorlage speichert die MSBuild-Ziele im Projektordner. Die mitgelieferten Beispiele verwenden die gemeinsamen Targets im Repository.

Der nachgewiesene Umfang umfasst native Desktop-Konsolenprojekte, Debug-/Release-Ausgabepfade, Build, Clean und Rebuild sowie mehrere CSharpXCompile-Dateien im bestehenden Objekt-/Vererbungs-Subset. Die Konfiguration wählt derzeit den Ausgabepfad. Projektreferenzen, CLR-/NuGet-Ausführung, weitere Zielplattformen und Visual-Studio-Debugging bleiben offen. Die VSIX-Registrierung und die Vorlage wurden im Paket geprüft. Visual Studio 2022 hat die Projektdatei mit devenv.com /Rebuild Debug geladen und erfolgreich gebaut. Der Start per F5 bleibt offen.


## Natives Debugging in Visual Studio

Wähle Debug und erstelle das Projekt neu. Bei einem einfachen Projekt mit einer .csx-Datei unter Windows/MSVC setzt du den Haltepunkt mit F9 direkt in Program.csx, startest mit F5 und gehst mit F10 zur nächsten CSharpX-Anweisung. Der Build benötigt Python 3 als python im PATH und schreibt die Quellzuordnung in die PDB neben der EXE. Im Objekt-/Vererbungspfad und bei Mehrdatei-Projekten gelten weiterhin die nativen Symbole des generierten Codes.
Am Haltepunkt öffnest du in Visual Studio unter Debuggen → Fenster → Lokal die lokalen Variablen. Doppelklicke bei einer Ganzzahl in die Spalte Wert, gib den neuen Wert ein und bestätige mit Enter. Die Änderung gilt in der laufenden Debugsitzung. Die Wertbearbeitung von Strings und Objekten ist nicht verifiziert.

## Ziel-Betriebssystem in Visual Studio

Wähle im Projektmappen-Explorer das CSharpX-Projekt und öffne mit F4 das Fenster Eigenschaften. Unter Ziel-Betriebssystem stehen Windows (x64), Linux (x64) und macOS (x64) zur Verfügung. Speichere das Projekt. Visual Studio schreibt die Wahl als CSharpXTargetOs in die csxproj. Der Programmname und ein optionaler Linkerpfad lassen sich im selben Fenster bearbeiten.

Das Ziel steuert die an rustc übergebene Zielkennung und den Ausgabeordner. Bei einem Cross-Build werden die Dateien in einem Unterordner für das Ziel abgelegt, beispielsweise bin\Debug\Linux. Linux und macOS benötigen jeweils das passende Rust-Ziel, einen Linker und gegebenenfalls ein Sysroot oder SDK. Der integrierte F5-Start auf Windows gilt für Windows-Programme. Projekte im Repository beziehen die Regel aus den gemeinsamen Targets. Bei Projekten aus älteren Vorlagen müssen die lokalen Targets und die zugehörige XAML-Regel aktualisiert werden.


## Buildmeldungen in Visual Studio

CSharpX-Projekte zeigen CSharpX-Diagnosen. Fehlende Zielwerkzeuge oder Linker, gesperrte Ausgaben und Linkfehler haben eigene Hinweise. Interne Compilerbefehle, generierte Dateien und Warnungen des nativen Compilers erscheinen im normalen Buildprotokoll nicht. Der Fehlerstatus wird weiterhin an Visual Studio gemeldet.

Die eigene Eigenschaftenseite CSharpX ist per Rechtsklick auf das Projekt und Eigenschaften oder mit Alt+Enter erreichbar. Der Ausgabetyp Anwendung erzeugt unter Windows eine EXE und unter Linux oder macOS eine ausführbare Datei ohne Endung. Bibliothek erzeugt eine DLL, SO oder DYLIB. Bibliotheken verwenden die native Rust-ABI, brauchen zur Nutzung eine kompatible Toolchain sowie deren Laufzeit und sind keine .NET-Assemblies. Windows-EXE und Windows-DLL wurden durch Ausführung bzw. einen separaten Aufrufer geprüft. Linux- und macOS-Ausgabepfade wurden ausgewertet.
