# C#-Quellen aus CSharpX aufrufen

CsUsage.csxproj enthält Program.csx, Greeting.cs und TextTools.cs. Baue den CSharpX-Compiler im Repository-Stamm mit cargo build --release und öffne dann CsUsage.csxproj in Visual Studio 2022. Zum Bauen ist das .NET-9-SDK nötig. Ohne eingebettete Laufzeit muss auf dem Zielrechner die .NET-9-Laufzeit über dotnet erreichbar sein.

~~~csharp
using System;

using "Greeting.cs";
using "TextTools.cs" AS Text;
~~~

Greeting.cs stellt Greet global in dieser CSX-Datei bereit. TextTools.cs stellt Upper unter Text.Upper bereit. Binde weitere C#-Dateien als CSharpXCompile-Elemente ein und verwende pro zusätzlicher Datei einen eigenen Alias. Ein globaler C#-Import pro CSX-Datei wird unterstützt.

Aufrufbar sind öffentliche statische Methoden mit der Signatur string Methode(string wert). Die öffentliche Klasse steht im globalen Namespace und trägt denselben Namen wie die Datei ohne .cs. Weitere C#-Klassen und Methoden können innerhalb des importierten C#-Codes verwendet werden.

System enthält die bekannte Console-Oberfläche. CSharpX kennt Console.WriteLine bereits in seiner eingebauten Bibliothek und akzeptiert using System; optional. using mit einem Dateinamen wählt externe Projektdateien aus.

## Ausgabe in Visual Studio

Öffne die Projekteigenschaften und wähle CSharpX → Ausgabe. Beide Optionen sind standardmäßig aus.

| Einzeldatei für C#-Importe | .NET-Laufzeit mitliefern | Auslieferung |
| --- | --- | --- |
| Aus | Aus | EXE mit dem Ordner CsUsage.managed und lokal installierter .NET-9-Laufzeit |
| An | Aus | EXE oder DLL mit eingebetteten C#-Dateien und lokal installierter .NET-9-Laufzeit |
| An | An | EXE oder DLL mit eingebetteten C#-Dateien und .NET-9-Laufzeit für Windows x64 |

Eine gebündelte DLL exportiert csx_run mit der C-Signatur int csx_run(void). Die Funktion führt Main aus und liefert 0 bei Erfolg. Jeder Aufruf einer importierten C#-Methode startet einen separaten .NET-Prozess. Bei Einzeldateiausgabe werden die eingebetteten Dateien für diesen Aufruf vorübergehend im Temp-Ordner abgelegt und danach entfernt.

Aufrufe importierter C- und C#-Funktionen müssen in einem unsafe-Block stehen. Der Block hat einen eigenen Variablenbereich und markiert die Vertrauensgrenze. Eingebaute CSharpX-Aufrufe benötigen diesen Block nicht.
