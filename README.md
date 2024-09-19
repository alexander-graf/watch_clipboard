# Clipboard Watcher für Obsidian

Ein Rust-basiertes Programm, das die Zwischenablage überwacht und automatisch Bilder und Text speichert. Bei neuen Bildern wird eine Markdown-Datei erstellt und Obsidian öffnet sich mit dieser Datei.

## Funktionen

- **Zwischenablage überwachen:** Das Programm überwacht kontinuierlich die Zwischenablage auf neue Inhalte (Bilder und Text).
- **Bilder speichern:** Wenn ein neues Bild in der Zwischenablage gefunden wird, wird es als PNG-Datei im Verzeichnis `~/Nextcloud/Obsy` gespeichert.
- **Markdown-Datei erstellen:** Eine zugehörige Markdown-Datei wird erstellt, die einen Verweis auf das gespeicherte Bild enthält.
- **Obsidian öffnen:** Nach dem Speichern eines Bildes öffnet das Programm Obsidian mit der neuen Markdown-Datei.
- **Text speichern:** Neuer Text in der Zwischenablage wird in einer separaten Textdatei gespeichert.

## Voraussetzungen

Um dieses Programm auszuführen, benötigen Sie:

- Rust (einschließlich Cargo)
- `obsidian-cli` muss installiert sein und im System-PATH verfügbar sein.
- Die folgenden Rust-Abhängigkeiten:

```toml
[dependencies]
arboard = "3.2.0"
image = "0.24.6"
chrono = "0.4.26"
dirs = "5.0.1"
uuid = { version = "1.3.0", features = ["v4"] }
which = "4.4.0"
