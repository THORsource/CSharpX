# AUTOSAR-orientierter TcpIp-Stack

Der ausführbare Stack steht in src/lib.rs. Er verwaltet pro Instanz einen Ethernet-Controller, die statische IPv4-Adresse, einen begrenzten Socketbestand und den Kommunikationszustand. smoltcp 0.12.0 verarbeitet Ethernet-, ARP-, IPv4-, TCP- und UDP-Pakete. Der Treiberanschluss akzeptiert eine smoltcp Device-Implementierung. src/frame_device.rs liefert dafür einen virtuellen Ethernet-Port mit begrenzten RX- und TX-Queues. Die Paketdaten laufen direkt zwischen zwei Stack-Instanzen.

## Bauen und ausführen

Im Ordner dieses Beispiels:

```powershell
cargo test --locked
cargo run --locked
```

Der erste Build benötigt die im Lockfile festgelegten Pakete von crates.io. Danach können beide Befehle mit --offline ausgeführt werden. Die Ausgabe des Programms zeigt die im Lauf beobachteten Frame-Typen. Das Programm prüft die Nutzdaten und den Absender des UDP-Datagramms als Teil des Ablaufs.

## Dienste und Schnittstellen

| CSharpX-Beispiel (Rust-API) | Bezug zur AUTOSAR-TcpIp-Spezifikation |
|---|---|
| init, request_com_mode, main_function | Initialisierung und zyklische Verarbeitung mit OFFLINE, STARTUP, ONLINE, ONHOLD und SHUTDOWN |
| Indication::Mode und Indication::Address | Statusmeldungen zur Anbindung an EthSM und obere Kommunikationsschichten |
| get_socket, tcp_listen, tcp_connect, tcp_is_connected | Socket-Handles und TCP-Verbindungsaufbau |
| tcp_transmit, tcp_receive | TCP-Streams mit Annahme teilweise übertragener Daten und Freigabe des Empfangspuffers beim Lesen |
| udp_bind, udp_transmit, udp_receive | UDP-Datagramme mit lokalem Port sowie Absenderadresse und Absenderport |
| FrameDevice::receive_frame und take_frame | Rahmenübergabe am virtuellen Ethernet-Port |

Im Modus ONHOLD bleiben Adresse und Socketbestand erhalten. Die öffentliche Socket-API und das Protokoll-Polling werden pausiert. Beim Wechsel zu OFFLINE von ONLINE aus schließt der Stack UDP-Sockets sofort und stößt den TCP-Verbindungsabbau an. Nach Schließung der TCP-Sockets oder nach Ablauf von 15 Sekunden gibt er die Adress- und Socketressourcen frei. OFFLINE aus ONHOLD gibt die Sockets ohne weitere Übertragung frei. Das Beispiel unterstützt maximal acht Sockets je Controller.

## Reichweite

Der Stack ist eine Implementierung eines abgegrenzten IPv4/Ethernet-Profils mit Rust-API. Die Spezifikation R25-11 dient als Referenz für die ausgewählten Zustands- und Socketdienste. Für ein vollständiges AUTOSAR-Classic-BSW-Modul fehlen die standardisierten C-API-Verträge, ECUC/ARXML, Hardware- und BSW-Integration, Rückrufverträge (unter anderem Socket-Owner und Empfangsbestätigung), schaltbare Adresszuweisungsverfahren, IPv6, zusätzliche Protokollprofile sowie ECU- und Konformitätstests. Insbesondere modelliert ein TCP-Listener im Beispiel eine einzelne Verbindung, und der Shutdown-Timeout ist eine eigene Festlegung des Beispiels.

CSharpX besitzt derzeit keinen Aufrufpfad für diese Rust-API. Die früheren Dateien AutosarTcpIp.csxproj, Program.csx und tcpip_demo.py sind als separates Desktop-Szenario erhalten. Sie werden für die Stack-Implementierung und ihre Tests nicht benötigt.

Referenz: [AUTOSAR Classic Platform R25-11 – Specification of TCP/IP Stack](https://www.autosar.org/fileadmin/standards/R25-11/CP/AUTOSAR_CP_SWS_TcpIp.pdf), Kapitel 7.8 und 8.
