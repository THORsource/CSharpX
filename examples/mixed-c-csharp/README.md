# C und C# gemeinsam aus CSharpX aufrufen

Öffne MixedInterop.csxproj in Visual Studio 2022. Das Beispiel benötigt Windows x64, die Visual-Studio-C++-Buildtools und das .NET-9-SDK. Baue den CSharpX-Compiler im Repository-Stamm einmal mit cargo build --release. Baue und starte dann das Beispielprojekt.

Program.csx importiert c_file_access.h als Native und Summary.cs als Managed. Die C-Funktion schreibt zwei Zeilen in mixed-interop-demo.txt, liest die Datei wieder ein und liefert den Text an CSharpX. CSharpX gibt ihn aus und reicht ihn an C# weiter. C# zählt die Zeilen und liefert die Zusammenfassung C# hat 2 Zeilen aus C erhalten.

Die Datei entsteht im Arbeitsverzeichnis. Für einen weiteren Lauf wähle im Beispiel einen neuen Dateinamen oder benenne die erzeugte Datei um. Eine vorhandene Datei wird beim Start nicht überschrieben.

Die C-Quelle und ihr Header sind als verknüpfte Projektelemente eingebunden. Der Build verwendet dieselben C-Dateien und Buildregeln wie das eigenständige C-Dateibeispiel. C# wird beim Projektbuild mitgebaut. Unter CSharpX → Ausgabe lassen sich Einbettung der C#-Dateien und zusätzliche Einbettung der .NET-Laufzeit einstellen.

Aufrufe importierter C- und C#-Funktionen müssen in einem unsafe-Block stehen. Der Block hat einen eigenen Variablenbereich und markiert die Vertrauensgrenze. Eingebaute CSharpX-Aufrufe benötigen diesen Block nicht.
