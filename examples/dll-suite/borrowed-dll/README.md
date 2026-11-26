# Geliehene CSharpX-Konfiguration über eine DLL

Baue BorrowedRules.csxproj und danach BorrowedClient.csxproj mit derselben Windows-x64-Konfiguration. Beide Projekte setzen CSharpXAbiBorrow auf true. Die Entsprechung für build-project lautet --abi-borrow. Der CSharpX-Aufruf bleibt Dll.Read(config).

new SampleConfig() erzeugt die Konfiguration im stabilen Layout. Das Beispiel erzeugt zusätzlich einen Alias auf dieselbe Instanz. Beide Aufrufe liefern 22. Davor gibt das Programm den unveränderten Getterwert 21 aus.

Der Aufruf ist synchron. Die EXE hält die Instanz für den vollständigen Aufruf am Leben. Die DLL bekommt die ursprüngliche Adresse und darf die Konfiguration über ihre Getter lesen. CSharpX erlaubt dem Exportkörper weder Mutation noch Speicherung, Freigabe oder Rückgabe dieser Referenz.

CSXABI3 verwendet eine skalare int- oder bool-Rückgabe. CSXABI4 ergänzt den bestehenden UTF-8-Textparameter und die Textantwort aus CSXABI2. Die Textpuffer werden weiterhin kopiert. CSXABI1 und CSXABI2 bleiben der Standard für Wertkopien.

Der Build prüft Modus, Klassenname, Gettertypen und das passende DLL-Symbol. Die Schnittstelle setzt eine vertrauenswürdige, compilererzeugte DLL voraus. Zum Start gehören EXE und DLL in denselben Ordner. Das Manifest ist eine Build-Eingabe.
