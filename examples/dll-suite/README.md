# CSharpX-DLL-Beispiele

Öffne CSharpX.DllExamples.sln in Visual Studio 2022. Baue vorher den CSharpX-Compiler mit cargo build --release im Repository-Stamm und stelle sicher, dass die CSharpX-Erweiterung installiert ist. Die Beispiele verwenden Windows x64. Die beiden Textprojekte nutzen eine gemeinsam verlinkte C-Dateifunktion aus dem Repository und benötigen dafür die C++-Buildtools.

Die Projektmappe enthält acht Projekte und vier DLL-Verträge.

| Aufrufer | Vertrag | Konfigurationsübergabe | Ergebnis |
| --- | --- | --- | --- |
| MeterClient | CSXABI1 | Wertkopie | 51 |
| TextClient | CSXABI2 | Wertkopie mit UTF-8-Text | [Grüße • 日本語\|Grüße • 日本語\|] |
| BorrowedClient | CSXABI3 | Geliehene Instanz | 21, 22, 22 |
| BorrowedTextClient | CSXABI4 | Geliehene Instanz mit UTF-8-Text | [Grüße • 日本語\|Grüße • 日本語\|] |

Jeder Aufrufer hat seine DLL als Build-Abhängigkeit. Wähle nach dem Build den gewünschten Aufrufer als Startprojekt. Beide Textbeispiele legen eine neue Datei native-report.txt im Arbeitsverzeichnis an. Führe sie für wiederholte Läufe in getrennten Arbeitsverzeichnissen aus.
