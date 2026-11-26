# Typisierter Textaufruf über eine CSharpX-DLL

Baue unter Windows x64 zuerst TextRules.csxproj und danach TextClient.csxproj mit derselben Konfiguration. Das Beispiel nutzt die C++-Buildtools für die vorhandene C-Dateifunktion. Starte TextClient.exe. Es gibt [Grüße • 日本語|Grüße • 日本語|] aus und erzeugt native-report.txt im Arbeitsverzeichnis. Verwende für weitere Läufe ein frisches Arbeitsverzeichnis.

Dll.Describe(config, text) erhält eine unveränderliche TextConfig und einen string. Die Bibliothek gibt string zurück. Describe.csx verwendet Getter, if, while, ein lokales String-Array, Interpolation und try/catch/finally. Decorate.csx enthält eine Hilfsfunktion mit einem eigenen Headerimport. Die Hilfsfunktion ruft C in einem expliziten unsafe-Block auf. Für die DLL-Übergabe erzeugt der Compiler sämtliche Adapter.

Das Build-Manifest CSXABI2 beschreibt den String-Vertrag. Der ABI-Datensatz enthält Getterwerte, eine Byteanzahl und einen festen UTF-8-Puffer. Auch Rückgabewert und Fehlermeldung werden über einen geprüften Puffer übertragen. Ein- und Ausgabe sind jeweils auf 16.384 UTF-8-Bytes begrenzt. Der Aufruf kopiert Werte. Es werden keine Anwendungsreferenzen oder Eigentumsrechte übergeben. Die EXE und DLL werden gemeinsam ausgeliefert. Das Manifest wird nur beim Build benötigt.

Eine Hilfsquelldatei enthält genau eine statische Funktion mit int-, bool- oder string-Parametern und entsprechendem Rückgabetyp. return innerhalb von try, catch und finally sowie Überladungen sind im aktuellen Funktionsumfang offen. Der ältere CSXABI1-Vertrag bleibt für Konfigurationen mit int- oder bool-Rückgabe verfügbar.
