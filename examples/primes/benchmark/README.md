# Die ersten 1.000 Primzahlen

Die vier Programme verwenden dieselbe Probedivision mit vorzeichenbehafteten 64-Bit-Ganzzahlen. Kandidaten beginnen bei 2, Teiler beginnen bei 2. Die Teilersuche endet bei einem Treffer oder sobald das Quadrat des Teilers größer als der Kandidat ist. Jede Suche füllt einen wiederverwendeten Speicher mit genau 1.000 Primzahlen und bildet deren Summe.

Pro Sprache wurden neun Prozesse in wechselnder Reihenfolge gemessen. Jeder Prozess wärmt mit 20 Suchen auf und misst dann 1.000 Suchen am Stück. Die Zeit pro Suche ergibt sich aus der Gesamtzeit geteilt durch 1.000. Prozessstart, Kompilierung, Speicherinitialisierung und Konsolenausgabe liegen außerhalb des Messfensters. Die Prüfsumme und das Schreiben der Treffer sind Teil der Messung. Ein Python-Sieb prüfte die vollständige Liste und die Prüfsumme aller Prozesse. Die letzte Primzahl ist 7919, die Summe der Liste ist 3682913.

| Version | Median (µs) | Minimum (µs) | Maximum (µs) |
| --- | ---: | ---: | ---: |
| C | 216,76 | 210,91 | 229,36 |
| C# | 218,67 | 211,39 | 221,99 |
| Rust | 128,90 | 124,57 | 139,59 |
| CSharpX | 129,27 | 125,42 | 138,57 |

Gemessen auf Windows x64. C lief mit MSVC 14.44 und /O2, C# mit .NET 9 Release und deaktivierter Tiered Compilation, Rust mit rustc 1.100.0 und Optimierungsstufe 3, CSharpX mit dem für dieses Beispiel erweiterten Compiler und --optimize (Optimierungsstufe 3 im Backend). Die Werte gelten für diese Implementierungen und diesen Rechner.

## Wiederholen

Voraussetzungen sind die Visual Studio C++-Buildtools, ein .NET-9-SDK mit Feature-Band 9.0.3xx oder neuer innerhalb .NET 9, Rust mit MSVC-Ziel, Python 3 und der aktualisierte CSharpX-Compiler. Zuerst im Repository-Stamm den CSharpX-Compiler mit cargo build --release bauen. Danach im Ordner examples/primes/benchmark ausführen.

```powershell
.\benchmark.ps1 -Csxc '..\..\..\target\release\csxc.exe'
```

Das Skript baut alle vier Versionen und schreibt neue Einzelmessungen nach `build/messwerte.csv`. Jedes Programm gibt zuerst Ticks, Frequenz in Hertz und die Prüfsumme aus. Danach folgen die 1.000 Primzahlen.
