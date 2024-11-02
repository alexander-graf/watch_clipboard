# Clipboard Monitor

Ein Rust-basiertes Tool zum Überwachen der Zwischenablage und automatischen Speichern von Bildern und Text in Obsidian-kompatiblem Markdown-Format.

## Features

- Automatische Überwachung der Zwischenablage
- Speichert Bilder als PNG mit zugehöriger Markdown-Datei
- Speichert Text als Markdown-Datei
- Unterstützt Tags für Screenshots
- Integration mit Obsidian über obsidian-cli
- Konfigurierbare Speicherpfade
- Ausführliche Logging-Funktion

## Voraussetzungen

- Rust (neueste stabile Version)
- Linux-System mit X11
- Folgende externe Programme:
  - `zenity` - für Ordnerauswahl und Dialoge
  - `yad` - für Tag-Eingabe und Dialoge
  - `obsidian-cli` - für Obsidian-Integration
  - `Nextcloud` (optional) - für Synchronisation

## Installation

1. Repository klonen:
```bash
git clone [repository-url]
cd clipboard-monitor
```

2. Kompilieren und installieren:
```bash
cargo build --release
sudo cp target/release/clipboard-monitor /usr/local/bin/
```

## Konfiguration

Beim ersten Start wird automatisch eine Konfigurationsdatei unter `~/.config/clipboard-monitor/config.json` erstellt. Sie werden aufgefordert:

1. Den Speicherort für Screenshots auszuwählen
2. Den Speicherort für Textnotizen auszuwählen

Die Standardpfade sind:
- Screenshots: `~/Nextcloud/Obsy/Screenshots`
- Notizen: `~/Nextcloud/Obsy/Notes`

## Verwendung

1. Programm starten:
```bash
clipboard-monitor
```

2. Bilder oder Text in die Zwischenablage kopieren:
   - Bei Bildern werden Sie nach Tags gefragt (kommagetrennt)
   - Bei Bildern werden Sie gefragt, ob Obsidian geöffnet werden soll
   - Alle Dateien werden automatisch im konfigurierten Verzeichnis gespeichert

## Dateiformate

### Screenshots
- Bilder werden als PNG gespeichert
- Markdown-Datei wird mit folgendem Format erstellt:
```markdown
## Screenshot vom [Datum] um [Uhrzeit]

#tag1 #tag2 #tag3

![Screenshot](bildname.png)
```

### Textnotizen
```markdown
Text aus der Zwischenablage vom [Datum] um [Uhrzeit]

[Kopierter Text]
```

## Logging

Debug-Informationen werden in `~/bin/watch_clipboard_debug.txt` protokolliert.

## Fehlerbehebung

- Stellen Sie sicher, dass alle erforderlichen Programme installiert sind
- Überprüfen Sie die Log-Datei für detaillierte Fehlerinformationen
- Stellen Sie sicher, dass die Zielverzeichnisse existieren und beschreibbar sind

## Lizenz

[Ihre gewählte Lizenz]

## Beitragen

Fehler, Feature-Requests und Pull-Requests sind willkommen!
