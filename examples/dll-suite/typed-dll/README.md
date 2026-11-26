# Typisierter CSharpX-Aufruf über eine DLL

Das Beispiel besteht aus MeterRules.csxproj und MeterClient.csxproj. Baue zuerst MeterRules als Windows-x64-Bibliothek und danach MeterClient in derselben Konfiguration. Öffne bei Bedarf beide Projekte in Visual Studio 2022 oder führe die folgenden Befehle in einer Visual-Studio-Entwicklerkonsole vom Repository-Stamm aus.

~~~bat
cargo build --release
msbuild examples\dll-suite\typed-dll\Library\MeterRules.csxproj /p:Configuration=Debug
msbuild examples\dll-suite\typed-dll\Client\MeterClient.csxproj /p:Configuration=Debug
examples\dll-suite\typed-dll\Client\bin\Debug\MeterClient.exe
~~~

Beide Projekte binden Configuration.csx ein. Die Bibliothek legt die CSharpX-Implementierung über CSharpXAbiExport fest. Der Aufrufer referenziert die zuvor erzeugte DLL über CSharpXAbiReference. Er erzeugt mit new eine MeterConfig und ruft Dll.Evaluate(config) auf. Die Bibliothek liest Enabled() und Units() am typisierten Parameter. Das Programm gibt 51 aus.

Der Bibliotheksbuild erzeugt MeterRules.dll und das passende Build-Manifest MeterRules.csxabi. Der Aufruferbuild benötigt beide Dateien. Er prüft den Vertrag und kopiert die DLL neben die EXE. Zur Auslieferung dieses Beispiels reichen MeterClient.exe und MeterRules.dll.

Derzeit wird pro Projekt ein Export oder eine DLL-Referenz unterstützt. Der Aufruf erwartet eine lokale Instanz der benannten, unveränderlichen Konfigurationsklasse und liefert einen einzelnen int- oder bool-Wert. Die Klasse bietet 1 bis 16 parameterlose int- oder bool-Getter. Die Exportfunktion unterstützt einen return-Ausdruck sowie ein if mit einer Rückgabe im Block und einer abschließenden Rückgabe. Namespaces, Überladungen, Strings, Arrays, Zeiger, weitere Parameter und Rückgabeobjekte gehören nicht zu diesem Vertrag.

Die Schnittstelle ist für vertrauenswürdige CSharpX-DLLs vorgesehen. Der Vertrag prüft Form und Aufrufkonvention. Die Bibliothek läuft im Prozess des Aufrufers und kann wie jede native DLL Code ausführen.
