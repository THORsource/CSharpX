# Gemeinsame Klassen und Startobjekt

Öffne SharedClasses.csxproj in Visual Studio 2022. Unter Projekteigenschaften → CSharpX → Ausgabe steht das Feld Startobjekt. Gib Program oder AlternateProgram ein. Program gibt 42 aus. AlternateProgram gibt zuerst Alternative und dann 42 aus.

Die Einstiegspunkte stehen in Programmklassen mit je einer statischen Main-Methode. Beide nutzen Reading und TemperatureReading aus eigenen CSX-Dateien. Die Einträge CSharpXCompile in der Projektdatei entscheiden, welche Dateien zusammen kompiliert werden. Die Basisklasse darf in der Liste hinter der abgeleiteten Klasse stehen.

Ist genau eine Main-Methode vorhanden, kann das Feld leer bleiben. Bei mehreren Einstiegspunkten verlangt der Compiler die Angabe eines Klassennamens. Der gleiche Wert lässt sich in der Projektdatei als StartupObject oder für einen einzelnen MSBuild-Aufruf über /p:StartupObject=AlternateProgram setzen.

Das Beispiel verwendet das vorhandene Subset für Klassen und Vererbung. Teildefinitionen derselben Klasse (partial class) sind weiterhin offen. Das Mehrdatei-Beispiel CompilerComposition zeigt Klassen zusammen mit skalarem Kontrollfluss und einem C-Dateiimport. Die Auswahl im Visual-Studio-Fenster erfolgt über ein Textfeld.
