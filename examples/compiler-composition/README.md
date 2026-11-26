# Klassen, Kontrollfluss und C-Import gemeinsam verwenden

Öffne CompilerComposition.csxproj in Visual Studio 2022. Baue vorher den CSharpX-Compiler mit cargo build --release. Das Beispiel benötigt Windows x64 und die C++-Buildtools für die bereits vorhandene C-Dateibrücke.

SampleReading.csx definiert eine Klasse mit parameterlosen Gettern für int, string und bool. Main instanziiert diese Klasse, weist Getter-Ergebnisse lokalen Variablen zu und nutzt Getter in einer Bedingung und in String-Interpolation. Die Schleife gibt Sample 0 und Sample 1 aus. Danach schreibt und liest ein Aufruf aus C die Datei compiler-demo.txt. Die Datei wird nur neu angelegt. Verwende für weitere Läufe einen freien Dateinamen.

Die Dateiimporte gelten pro Quelldatei. Aufrufe importierter Funktionen bleiben auf unsafe-Blöcke beschränkt. Parameterisierte Konstruktoren und mutable Klassenfelder gehören nicht zum Beispielumfang.
