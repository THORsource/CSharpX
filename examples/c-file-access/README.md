# Dateizugriff aus CSharpX über C

Öffne CFileAccess.csxproj in Visual Studio 2022. Baue zuerst den CSharpX-Compiler mit cargo build --release im Repository-Stamm. Visual Studio benötigt die C++-Desktop-Entwicklung mit den x64-Buildtools.

Das Projekt enthält main.c und extra_file.c mit ihren Headern c_file_access.h und extra_file.h. Der Build kompiliert die C-Dateien einzeln, bündelt die Objektdateien in einer statischen Bibliothek und importiert unterstützte Funktionsnamen aus allen eingebundenen Headern.

Program.csx importiert beide Header vor Main und ruft danach csx_c_file_round_trip und Extra.csx_extra_file auf.

~~~csharp
using "c_file_access.h";
using "extra_file.h" AS Extra;
~~~

Ein Header muss sowohl im Projekt eingebunden als auch in der aufrufenden CSX-Datei mit using ausgewählt sein. Ohne Alias ist der C-Name direkt verfügbar. Mit AS oder as ist die Funktion über Alias.Funktionsname aufrufbar. Aliase unterscheiden Groß- und Kleinschreibung und dürfen pro Datei nur einmal vergeben werden. Die Dateien c-file-access-demo.txt und c-file-access-extra.txt entstehen im Arbeitsverzeichnis. Für einen weiteren Lauf die erzeugten Dateien umbenennen oder im Beispiel andere Namen wählen. Vorhandene Dateien bleiben unverändert.

## Weitere C-Dateien hinzufügen

Füge weitere C-Dateien und Header als Projektelemente hinzu. In der Projektdatei sehen die Einträge so aus.

~~~xml
<CSharpXCompile Include="sensor.c" />
<CSharpXCompile Include="sensor.h" />
~~~

Binde in sensor.c den Header per #include "sensor.h" ein. Ergänze am Anfang der aufrufenden CSX-Datei using "sensor.h";. Danach kann CSharpX die unterstützte Funktion unter ihrem C-Namen aufrufen, wenn sie beim Projektbuild kompiliert und verlinkt wird.

Die Anbindung unterstützt Windows x64 und ASCII-Dateipfade. Importierbar sind Deklarationen der Form int name(const char *path, unsigned char *output, size_t capacity);. CSharpX übergibt den Dateipfad als string und erhält den UTF-8-Text als string zurück. Weitere C-Signaturen und ein allgemeiner C-Präprozessor sind noch offen.

## Eigenständiges C-Programm

Im x64 Native Tools Command Prompt for VS 2022 vom Repository-Stamm aus:

~~~bat
cd examples\c-file-access
mkdir build
cl /nologo /W4 /std:c11 /Fo:build\main.obj /Fe:build\c-file-access.exe main.c
build\c-file-access.exe build\demo.txt
~~~

Aufrufe importierter C- und C#-Funktionen müssen in einem unsafe-Block stehen. Der Block hat einen eigenen Variablenbereich und markiert die Vertrauensgrenze. Eingebaute CSharpX-Aufrufe benötigen diesen Block nicht.
