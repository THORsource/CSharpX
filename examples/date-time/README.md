# Datums- und Zeittypen in CSharpX

Öffne DateTimeDemo.csxproj in Visual Studio. Das Beispiel zeigt den 29. Februar eines Schaltjahres, eine Uhrzeit, einen Zeitstempel und eine Dauer.

| Typ | Konstruktoren | Eigenschaften und Formate |
|---|---|---|
| DateOnly | Jahr, Monat, Tag | Year, Month, Day sowie yyyy-MM-dd |
| TimeOnly | Stunde, Minute, optional Sekunde | Hour, Minute, Second sowie HH:mm:ss |
| DateTime | Jahr, Monat, Tag, optional Stunde, Minute und Sekunde zusammen | Datums- und Uhreigenschaften sowie yyyy-MM-dd HH:mm:ss und yyyy-MM-dd'T'HH:mm:ss |
| TimeSpan | Stunden, Minuten, Sekunden oder Tage, Stunden, Minuten, Sekunden | Days, Hours, Minutes, Seconds sowie c |

Das Standardformat ist in CSharpX sprachunabhängig. Für C#-Vergleiche verwende dieselben expliziten Formate. Der aktuelle Sprachumfang arbeitet mit Gregorianischem Kalender und ganzen Sekunden. Zeitzonen, DateTimeOffset, Systemuhr, Parsing, Datumsarithmetik und weitere Formate sind offen.