# Pointer in CSharpX

Baue Pointers.csxproj in Visual Studio 2022. Das Beispiel gibt 10, 42 und True aus. Es nimmt die Adresse einer lokalen Ganzzahl und eines lokalen Bool-Werts, liest durch einen Pointer und ändert beide Werte durch Pointer-Zuweisungen.

Die Operationen int*, bool*, &variable, *pointer und *pointer = wert sind innerhalb von unsafe-Blöcken verfügbar. Pointer verweisen auf lokale Werte. Der angesprochene Wert muss bis zum letzten Zugriff gültig bleiben. Der skalare Compiler prüft die Gültigkeitsdauer eines Pointers nicht.

CSharpX-int entspricht hier wie auch in den übrigen skalaren Beispielen einer vorzeichenbehafteten 64-Bit-Ganzzahl. Pointerarithmetik, Casts, Nullpointer und die Übergabe von Pointern an C- oder C#-Importe gehören noch nicht zu diesem Sprachumfang.
