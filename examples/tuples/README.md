# Tupel in CSharpX

Öffne Tuples.csxproj in Visual Studio 2022. Das Beispiel erstellt ein Tupel mit benannten Elementen, ändert einen Wert, greift über Item1 und Item2 zu, kopiert die Werte in ein explizit typisiertes Tupel und zerlegt es mit var. Es verwendet außerdem ein verschachteltes Tupel.

~~~csharp
var reading = (Value: 41, Unit: "kPa");
reading.Value = reading.Item1 + 1;
var (value, unit) = reading;
~~~

Unterstützte Elementtypen sind int, string, bool und weitere Tupel. Die Dekonstruktion erlaubt auch einen Platzhalter mit Unterstrich. Tupelparameter, Tupelrückgaben und Tupelvergleiche gehören noch nicht zum Sprachumfang.
