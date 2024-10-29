use arboard::Clipboard;
use std::time::Duration;
use std::thread::sleep;
use image::{ImageBuffer, RgbaImage};
use std::convert::TryInto;
use std::fs::{self, File};
use std::io::Write;
use chrono::Local;
use dirs::home_dir;
use std::path::PathBuf;
use uuid::Uuid;
use std::process::Command;
use std::fs::OpenOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    screenshots_path: PathBuf,
    notes_path: PathBuf,
    last_updated: String,
}

impl Config {
    fn new() -> Self {
        Config {
            screenshots_path: default_screenshots_path(),
            notes_path: default_notes_path(),
            last_updated: Local::now().format("%d.%m.%Y %H:%M:%S").to_string(),
        }
    }
}

fn default_screenshots_path() -> PathBuf {
    let mut path = home_dir().unwrap_or_default();
    path.push("Nextcloud/Obsy/Screenshots");
    path
}

fn default_notes_path() -> PathBuf {
    let mut path = home_dir().unwrap_or_default();
    path.push("Nextcloud/Obsy/Notes");
    path
}

fn get_config_path() -> PathBuf {
    let mut config_path = home_dir().unwrap_or_default();
    config_path.push(".config/clipboard-monitor/config.json");
    config_path
}

fn load_or_create_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if config_path.exists() {
        let config_str = fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&config_str)?;
        return Ok(config);
    }

    let mut config = Config::new();

    if let Ok(output) = Command::new("zenity")
        .args(&[
            "--file-selection",
            "--directory",
            "--title=Wähle Screenshots Ordner"
        ])
        .output()
    {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                config.screenshots_path = PathBuf::from(path.trim_end());
            }
        }
    }

    if let Ok(output) = Command::new("zenity")
        .args(&[
            "--file-selection",
            "--directory",
            "--title=Wähle Notizen Ordner"
        ])
        .output()
    {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                config.notes_path = PathBuf::from(path.trim_end());
            }
        }
    }

    if let Some(config_dir) = config_path.parent() {
        fs::create_dir_all(config_dir)?;
    }

    let config_str = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, config_str)?;

    Ok(config)
}

fn find_obsidian_cli() -> Option<PathBuf> {
    which::which("obsidian-cli").ok()
}

fn log_to_file(message: &str) -> std::io::Result<()> {
    let mut home = home_dir().ok_or_else(|| std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Home-Verzeichnis nicht gefunden"
    ))?;
    home.push("bin/watch_clipboard_debug.txt");
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&home)?;
        
    let timestamp = Local::now().format("%d.%m.%Y %H:%M:%S");
    writeln!(file, "[{}] {}", timestamp, message)?;
    Ok(())
}

fn monitor_clipboard(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("DISPLAY").is_err() {
        std::env::set_var("DISPLAY", ":0");
    }
    
    let mut retry_count = 0;
    let max_retries = 3;
    let mut clipboard = loop {
        match Clipboard::new() {
            Ok(cb) => break cb,
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    return Err(format!("Konnte Clipboard nach {} Versuchen nicht initialisieren: {}", max_retries, e).into());
                }
                log_to_file(&format!("Clipboard-Initialisierung fehlgeschlagen (Versuch {}), versuche erneut...", retry_count))?;
                sleep(Duration::from_secs(1));
            }
        }
    };

    clipboard.clear()?;
    log_to_file("Zwischenablage wurde geleert.")?;

    let mut last_image_hash = 0;
    let mut last_text_hash = 0;
    log_to_file("Überwache die Zwischenablage auf Bilder und Text...")?;

    let obsidian_cli_path = match find_obsidian_cli() {
        Some(path) => {
            log_to_file(&format!("obsidian-cli Pfad gefunden: {:?}", path))?;
            path
        },
        None => {
            log_to_file("obsidian-cli konnte nicht gefunden werden.")?;
            return Err("obsidian-cli nicht gefunden".into());
        }
    };

    loop {
        let mut clipboard_changed = false;

        if let Ok(image) = clipboard.get_image() {
            let new_hash = calculate_hash(&image.bytes);
            
            if new_hash != last_image_hash {
                log_to_file(&format!("Neues Bild in der Zwischenablage gefunden: {}x{}", image.width, image.height))?;
                
                let buffer: RgbaImage = ImageBuffer::from_raw(
                    image.width.try_into().unwrap(),
                    image.height.try_into().unwrap(),
                    image.bytes.into_owned(),
                ).unwrap();
                
                let md_path = save_image_and_markdown(&buffer, config)?;
                if ask_to_open_obsidian() {
                    open_obsidian_cli(&obsidian_cli_path, &md_path)?;
                }
                
                last_image_hash = new_hash;
                clipboard_changed = true;
            }
        }

        if let Ok(text) = clipboard.get_text() {
            if !text.is_empty() {
                let new_hash = calculate_hash(text.as_bytes());
                
                if new_hash != last_text_hash {
                    log_to_file(&format!("Neuer Text in der Zwischenablage gefunden: {} Zeichen", text.len()))?;
                    
                    match save_text(&text, config) {
                        Ok(_) => log_to_file("Text erfolgreich gespeichert")?,
                        Err(e) => log_to_file(&format!("Fehler beim Speichern des Texts: {}", e))?,
                    }
                    
                    last_text_hash = new_hash;
                    clipboard_changed = true;
                }
            } else {
                log_to_file("Leerer Text in der Zwischenablage")?;
            }
        }
        
        if !clipboard_changed {
            sleep(Duration::from_secs(1));
        }
    }
}

fn calculate_hash(data: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

fn generate_unique_filename(prefix: &str, extension: &str) -> String {
    let timestamp = Local::now().format("%d%m%Y_%H%M%S");
    let uuid = Uuid::new_v4();
    format!("{}_{}_{}{}",prefix, timestamp, uuid, extension)
}

fn save_image_and_markdown(buffer: &RgbaImage, config: &Config) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let timestamp = Local::now().format("%d.%m.%Y um %H:%M");
    let png_filename = generate_unique_filename("clipboard_image", ".png");
    let md_filename = generate_unique_filename("Screenshot", ".md");
    
    fs::create_dir_all(&config.screenshots_path)?;
    
    let png_path = config.screenshots_path.join(&png_filename);
    buffer.save(&png_path)?;
    
    let md_path = config.screenshots_path.join(&md_filename);
    let timestamp_str = timestamp.to_string();
    create_markdown_file(&md_path, &png_filename, &timestamp_str)?;
    
    log_to_file(&format!("Bild gespeichert als: {}", png_path.display()))?;
    log_to_file(&format!("Markdown-Datei erstellt: {}", md_path.display()))?;
    
    Ok(md_path)
}

fn save_text(text: &str, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let preview = text.chars().take(100).collect::<String>();
    let preview = if text.len() > 100 { format!("{}...", preview) } else { preview };
    
    Command::new("yad")
        .args(&[
            "--title=Text gespeichert",
            &format!("--text={}", preview),
            "--timeout=1",
            "--no-buttons",
            "--center",
            "--width=400"
        ])
        .spawn()?;

    let timestamp = Local::now().format("%d.%m.%Y um %H:%M");
    let txt_filename = generate_unique_filename("clipboard_text", ".md");
    
    fs::create_dir_all(&config.notes_path)?;
    
    let txt_path = config.notes_path.join(&txt_filename);
    let mut file = File::create(&txt_path)?;
    writeln!(file, "Text aus der Zwischenablage vom {}", timestamp)?;
    writeln!(file)?;
    write!(file, "{}", text)?;
    
    log_to_file(&format!("Text gespeichert als: {}", txt_path.display()))?;
    
    Ok(())
}

fn create_markdown_file(path: &PathBuf, image_filename: &str, timestamp: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    writeln!(file, "## Screenshot vom {}", timestamp)?;
    writeln!(file)?;
    writeln!(file)?;
    writeln!(file, "![Screenshot]({})", image_filename)?;
    Ok(())
}

fn open_obsidian_cli(obsidian_cli_path: &PathBuf, md_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = md_path.file_name()
        .ok_or("Konnte den Dateinamen nicht extrahieren")?
        .to_str()
        .ok_or("Konnte den Dateinamen nicht in einen String umwandeln")?;

    Command::new(obsidian_cli_path)
        .arg("open")
        .arg(file_name)
        .spawn()?;
    log_to_file(&format!("Obsidian geöffnet mit: {}", file_name))?;
    Ok(())
}

fn ask_to_open_obsidian() -> bool {
    match Command::new("yad")
        .args(&[
            "--title=Obsidian öffnen",
            "--text=Möchten Sie Obsidian öffnen?",
            "--button=Ja:0",
            "--button=Nein:1",
            "--center",
            "--width=300",
            "--height=100"
        ])
        .output() {
            Ok(output) => output.status.success(),
            Err(e) => {
                if let Err(log_err) = log_to_file(&format!("Fehler beim Ausführen von yad: {}", e)) {
                    eprintln!("Logging-Fehler: {}", log_err);
                }
                false
            }
        }
}

fn main() {
    match load_or_create_config() {
        Ok(config) => {
            if let Err(e) = monitor_clipboard(&config) {
                let error_msg = format!("Fehler: {}", e);
                if let Err(log_err) = log_to_file(&error_msg) {
                    eprintln!("Fehler beim Logging: {}", log_err);
                    eprintln!("{}", error_msg);
                }
            }
        }
        Err(e) => {
            eprintln!("Fehler beim Laden der Konfiguration: {}", e);
        }
    }
}
