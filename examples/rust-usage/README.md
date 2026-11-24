# Rust aus CSharpX aufrufen

Öffne RustUsage.csxproj in Visual Studio 2022. Baue den CSharpX-Compiler im Repository-Stamm mit cargo build --release. Das Beispiel kompiliert Greeting.rs und TextTools.rs mit dem Rust-Compiler für das Projektziel.

Program.csx importiert Greeting.rs ohne Alias und TextTools.rs unter dem Alias Text. Innerhalb von unsafe ruft es greet direkt und upper über Text.upper auf. Beide Funktionen werden in die Ausgabe einkompiliert. Die Beispielausgabe lautet HALLO, CSHARPX!.

Der Import unterstützt öffentliche Rust-Funktionen auf Modulebene mit der Signatur pub fn name(value: &str) -> String. Pro CSX-Datei kann eine Rust- oder C#-Datei ohne Alias importiert werden. Gib weiteren Dateien eigene Aliase. Cargo-Abhängigkeiten und andere Funktionssignaturen werden über diesen Import noch nicht eingebunden.
