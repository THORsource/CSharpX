# C / C++ interoperability (DRAFT)

## C

`[NativeImport("sensor.h", Library="sensor")]` binds functions with an explicit ABI and target. Header discovery requires the configured C preprocessor/Clang and target defines. `using native "sensor.h";` is optional proposed syntax.

## C++

Clang-based importer extracts supported declarations; generated C ABI shims handle overloads, methods, constructors/destructors and supported template instantiations. C++ ABI, exceptions, RTTI and compiler flags must match the target toolchain. Unsupported constructs are diagnosed.

## Safety

Untrusted native calls are unsafe even when their signatures have no pointers. Audited wrappers may expose safe interfaces with explicit ownership, pointer length, nullability, threading and lifetime contracts. C++ exceptions must be caught within a compatible native shim and translated to CSharpX exception transport.

## Build

Projects can include `.c`, `.cpp`, `.h`, `.hpp` sources and libraries, with explicit compile/link options per target. Native build failures retain source mapping.

## Implemented Windows x64 file bridge

In a scalar CSharpX source, using "name.h"; selects an included project header by filename. Only supported declarations from the headers selected in that source are imported under their original C names. An import without an alias makes its functions available as unqualified calls. using "name.h" AS Alias; or using "name.h" as Alias; requires Alias.function(...) for that import. Each alias is case-sensitive and must be unique within its source file. Unknown or ambiguous header filenames fail the project build. Pass the header via CSharpXCHeader in MSBuild or via build-project --c-header in the CLI. The CSharpX call takes an ASCII path without an embedded NUL and returns a UTF-8 string. The project must opt in with CSharpXTrustedCFileDirectory, or the CLI with build-project --trusted-c-file. This opts into running the native code supplied in csx_c_file_native.lib with the program's permissions. Only use a library built from reviewed sources.

The fixed C ABI is int name(const char *path, unsigned char *output, size_t capacity). Both pointers are borrowed for the duration of a single synchronous call. The caller supplies a NUL-terminated path and a writable 256-byte output buffer. The library retains neither pointer and writes at most capacity bytes. A nonnegative return is the number of bytes written, and a negative return reports failure. The wrapper checks the returned length and UTF-8 before producing a string. The one unsafe call is confined to that separately compiled wrapper. Generated application code continues to forbid unsafe code.

Calls to imported C and C# functions require a lexical unsafe block. A block grants permission to cross the external trust boundary and does not enable arbitrary unsafe backend code.

## Typisierte CSharpX-DLL-Verträge unter Windows x64

Ein DLL-Projekt enthält eine unveränderliche globale Konfigurationsklasse als CSharpXCompile und eine öffentliche statische Exportfunktion als CSharpXAbiExport. Die Funktion erhält die Klasse als einzigen Parameter. Für den ersten ABI-Umfang sind parameterlose, nichtvirtuelle int- und bool-Getter sowie ein skalares int- oder bool-Ergebnis definiert. Das aufrufende EXE-Projekt baut die Klasse mit ein und referenziert die DLL als CSharpXAbiReference. Der Aufruf lautet Dll.Funktion(config).

CSXABI1 sortiert Getter lexikografisch nach Name. Jeder Parameter-Slot belegt acht Bytes, wobei int als vorzeichenbehafteter 64-Bit-Wert und bool als 0 oder 1 kodiert wird. Windows x64 MSVC übergibt den repr(C)-Datensatz nach der C-Aufrufkonvention. Der Rückgabedatensatz enthält zuerst den 64-Bit-Status und danach das skalare Ergebnis in einem 64-Bit-Slot. Status 0 bedeutet Erfolg, Status 1 meldet einen Laufzeit- oder Layoutwertfehler. Die exportierte Funktion wird anhand der vollständigen, versionierten Signatur benannt.

Die Bibliothek und ihr nebenliegendes .csxabi-Manifest sind Build-Eingaben des Aufrufers. Der Compiler prüft die nominelle Klasse, Getter-Namen und Typen am Aufruf sowie den passenden Export in der Windows-x64-DLL. Die EXE importiert das geprüfte Symbol und der Build kopiert die DLL in das Ausgabeverzeichnis. Der build-project-Einstieg unterstützt die Optionen --abi-export und --abi-import.

Die Adapter sind compilererzeugt. Es werden Werte kopiert. Anwendungscode braucht an dieser geprüften Grenze keinen eigenen unsafe-Block. Die interne FFI-Operation liegt in einem separat kompilierten Adapter. Rust-Referenzen, Objekte und Zeiger sind kein Teil des ABI. Die Prüfung setzt eine vertrauenswürdige DLL voraus und ersetzt keine Prüfung der Herkunft. Frei importierte C-, C#- und Rust-Funktionen behalten ihre unsafe-Pflicht.

### UTF-8-Textvertrag CSXABI2

Die Form public static string Function(Config config, string text) verwendet ein getrenntes versioniertes Export-Symbol. Hinter den unveränderten Konfigurationsslots stehen ein u64 für die Byteanzahl und 16384 UTF-8-Bytes. Die Antwort enthält u64-Status, u64-Länge und einen gleich großen Puffer. Status 0 liefert den Text, Status 1 die Fehlermeldung und Status 2 meldet einen zu großen Ergebnispuffer. Die Adapter prüfen Länge und UTF-8 und geben keinen String-Speicher über die DLL-Grenze weiter.

Der Exportkörper und statische Hilfsfunktionen aus weiteren CSX-Dateien verwenden den skalaren Anweisungsparser. Dateilokale externe Importe bleiben an unsafe gebunden. Der überprüfte typisierte DLL-Aufruf benötigt im aufrufenden CSharpX-Code keinen unsafe-Block. Die ursprüngliche CSXABI1-Signatur bleibt verfügbar.

### Synchron geliehene Konfigurationen

Mit CSharpXAbiBorrow=true in beiden Projekten oder --abi-borrow in beiden Build-Aufrufen wählen Bibliothek und Aufrufer CSXABI3 für primitive beziehungsweise CSXABI4 für Textsignaturen. Bei new erzeugt der Aufrufer ein Objekt mit repr(C)-Layout. Die alphabetisch geordneten Getter entsprechen vorzeichenbehafteten 64-Bit-Slots mit Acht-Byte-Ausrichtung. bool verwendet 0 oder 1. Größe, Ausrichtung und jeder Offset werden in beiden generierten Adaptern statisch geprüft.

Der Adapter übergibt einen schreibgeschützten Zeiger auf diese konkrete Instanz und deren numerische Adresse zur Identitätsprüfung. Null, falsche Ausrichtung, abweichende Adresse und nichtkanonische bool-Slots werden abgewiesen. Der Aufrufer hält seine Besitzreferenz bis zur synchronen Rückkehr. Der generierte Export bindet die Konfiguration als geliehene, gemeinsame Referenz. Im CSharpX-Körper sind nur Getterzugriffe möglich. Die Referenz wird weder gespeichert, freigegeben, mutiert noch zurückgegeben. Der Fehlerkanal folgt dem jeweiligen Kopiervertrag.

CSXABI4 überträgt UTF-8 separat in den begrenzten Puffern aus CSXABI2. Das Export-Symbol enthält die Vertragsversion und die vollständige Signatur. Der Build lehnt gemischte Modi, inkompatible Konfigurationen und ein unpassendes Symbol ab. Geliehene Konfigurationsklassen unterstützen im aktuellen Umfang keine Vererbung oder virtuellen Getter. Die Laufzeitprüfung setzt voraus, dass beide Seiten aus vertrauenswürdigen CSharpX-Quellen gebaut wurden.
