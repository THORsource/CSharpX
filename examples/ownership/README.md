# Generics und Ownership anhand von Messwerten

Öffne Ownership.csxproj in Visual Studio 2022 und starte es im Debug-Modus. Die Ausgabe zeigt die konkreten Typen und Werte.

Reading<T, TLabel = string> hat einen erforderlichen Typparameter und einen optionalen Standardtyp. Reading<int> verwendet string, Reading<int, bool> überschreibt den Standardtyp. Das Beispiel weist eine Referenz zwischen Reading<string> und Reading<string, string> zu. Beide Schreibweisen ergeben dieselbe Spezialisierung.

Snapshot<TDate, TTime, TStamp, TDuration = TimeSpan> zeigt ein Zeitprofil. Insgesamt sind int, string, bool, DateOnly, TimeOnly, DateTime und TimeSpan im Beispiel vertreten. Der Compiler unterstützt die Typparameter als Rückgabetypen parameterloser Methoden des Objektpfads. Optionale Typparameter müssen am Ende stehen. Sie sind eine CSharpX-Spracherweiterung.

Der Ownership-Bericht zeigt Box für die Kalibrierung und Rc mit drei Referenzen für die Live-Messung. Er listet sämtliche aufgelösten Typparameter auf und markiert optionale Parameter mit ihrem Standardtyp. Führe den Befehl im CSharpX-Projektordner aus.

    .\target\release\csxc.exe ownership-report .\examples\ownership\Program.csx

Die Ownership-Analyse erfasst geradlinige lokale Aliaszuweisungen. Generische Methoden, Constraints und verschachtelte oder frei definierte Typargumente bleiben offen.
