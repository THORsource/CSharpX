# CSharpX unter Windows installieren

Doppelklicke im Repository auf **Install-CSharpX.bat**. Der Assistent führt dich in drei Schritten durch die Installation.

1. **Voraussetzungen prüfen.** Der Assistent zeigt, welche Werkzeuge verfügbar sind. Bestätige mit **J**, wenn er fehlende Werkzeuge installieren soll. Er prüft das .NET-9-SDK, Python 3, Rust und Cargo sowie Visual Studio 2022 mit C++-Buildtools und dem .NET-Framework-4.7.2-Targeting-Pack.
2. **CSharpX bauen.** Der Assistent baut den Compiler und die Visual-Studio-Erweiterung. Beim ersten Build kann der NuGet-Restore eine Internetverbindung benötigen. Der Compiler bleibt unter target\release\csxc.exe im Repository.
3. **Erweiterung installieren.** Bestätige erneut mit **J**. Schließe Visual Studio, falls es noch geöffnet ist, und folge dem VSIX-Installationsdialog.

Öffne anschließend in Visual Studio **Datei → Öffnen → Projekt/Projektmappe** und wähle examples\hello\Hello.csxproj. Baue das Projekt und starte es mit F5. Das Hello-Beispiel wartet auf Eingaben per Enter.

## Einzelne Schritte und Fehlerbehebung

- Install-CSharpX.bat -CheckOnly prüft die Voraussetzungen ohne Installation und Build.
- Install-CSharpX.bat -BuildOnly prüft und baut den Compiler sowie die VSIX ohne Installation.
- Setup-SDK.bat, Build-All.bat und Install-VSIX.bat führen die Schritte einzeln aus.
- Fehlende Pakete installiert der Assistent nach Bestätigung über winget. Falls winget fehlt, installiere den Microsoft App Installer oder die angezeigten Voraussetzungen manuell. Visual Studio Installer kann Administratorrechte verlangen. Nach einem angeforderten Neustart führe den Assistenten erneut aus.
- Die VSIX liegt unter ide\visual-studio-2022\VSIX\bin\Release\net472. Halte den Projektordner am selben Ort oder passe bei einem eigenen Projekt den Compilerpfad an.
