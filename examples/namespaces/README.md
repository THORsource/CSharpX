# Namespaces in CSharpX

Öffne Namespaces.csxproj in Visual Studio 2022. Das Beispiel enthält zwei Klassen mit dem Namen Reading. Demo.Sensors.Reading wird über using Demo.Sensors; eingebunden. Demo.Simulation.Reading wird mit ihrem vollständigen Namen verwendet. Das Programm gibt 42 und 99 aus. SensorReading ist ein Alias für einen Typ, Sim ein Alias für einen Namespace. Aliase gelten in ihrem using-Geltungsbereich.

Der Klassenpfad unterstützt namespace Name { ... } und namespace Name;. Namespaces dürfen verschachtelt sein. using-Direktiven gelten für die Datei oder den umschließenden Namespace. Treffen mehrere importierte Klassen mit demselben Kurznamen zu, verlangt der Compiler einen eindeutigen Namen.

Bei einem Einstiegspunkt im Namespace kann das Startobjekt mit vollständigem Namen gesetzt werden, hier Demo.App.Program. Generische Klassen in Namespace-Projekten und externe Dateiimporte im Klassenpfad sind noch offen.
