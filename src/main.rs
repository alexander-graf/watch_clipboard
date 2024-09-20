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
use native_dialog::{MessageDialog, MessageType};

fn find_obsidian_cli() -> Option<PathBuf> {
    which::which("obsidian-cli").ok()
}

fn monitor_clipboard() -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    clipboard.clear()?;
    println!("Zwischenablage wurde geleert.");

    let mut last_image_hash = 0;
    let mut last_text_hash = 0;
    println!("Überwache die Zwischenablage auf Bilder und Text...");

    let obsidian_cli_path = match find_obsidian_cli() {
        Some(path) => {
            println!("obsidian-cli Pfad gefunden: {:?}", path);
            path
        },
        None => {
            println!("obsidian-cli konnte nicht gefunden werden.");
            return Err("obsidian-cli nicht gefunden".into());
        }
    };

    loop {
        let mut clipboard_changed = false;

        // Überprüfe auf Bilder
        if let Ok(image) = clipboard.get_image() {
            let new_hash = calculate_hash(&image.bytes);
            
            if new_hash != last_image_hash {
                println!("Neues Bild in der Zwischenablage gefunden: {}x{}", image.width, image.height);
                
                let buffer: RgbaImage = ImageBuffer::from_raw(
                    image.width.try_into().unwrap(),
                    image.height.try_into().unwrap(),
                    image.bytes.into_owned(),
                ).unwrap();
                
                let md_path = save_image_and_markdown(&buffer)?;
                if ask_to_open_obsidian() {
                    open_obsidian_cli(&obsidian_cli_path, &md_path)?;
                }
                
                last_image_hash = new_hash;
                clipboard_changed = true;
            }
        }

        // Überprüfe auf Text
        if let Ok(text) = clipboard.get_text() {
            if !text.is_empty() {
                let new_hash = calculate_hash(text.as_bytes());
                
                if new_hash != last_text_hash {
                    println!("Neuer Text in der Zwischenablage gefunden: {} Zeichen", text.len());
                    
                    save_text(&text)?;
                    
                    last_text_hash = new_hash;
                    clipboard_changed = true;
                }
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
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let uuid = Uuid::new_v4();
    format!("{}_{}_{}{}",prefix, timestamp, uuid, extension)
}

fn save_image_and_markdown(buffer: &RgbaImage) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let timestamp = Local::now().format("%d.%m.%Y %H:%M");
    let png_filename = generate_unique_filename("clipboard_image", ".png");
    let md_filename = generate_unique_filename("Screenshot", ".md");
    
    let mut path = home_dir().unwrap();
    path.push("Nextcloud");
    path.push("Obsy");
    fs::create_dir_all(&path)?;
    
    let png_path = path.join(&png_filename);
    buffer.save(&png_path)?;
    
    let md_path = path.join(&md_filename);
    let timestamp_str = timestamp.to_string();
    create_markdown_file(&md_path, &png_filename, &timestamp_str)?;
    
    println!("Bild gespeichert als: {}", png_path.display());
    println!("Markdown-Datei erstellt: {}", md_path.display());
    
    Ok(md_path)
}

fn save_text(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = Local::now().format("%d.%m.%Y %H:%M");
    let txt_filename = generate_unique_filename("clipboard_text", ".txt");
    
    let mut path = home_dir().unwrap();
    path.push("Nextcloud");
    path.push("Obsy");
    fs::create_dir_all(&path)?;
    
    let txt_path = path.join(&txt_filename);
    let mut file = File::create(&txt_path)?;
    writeln!(file, "Text aus der Zwischenablage vom {}", timestamp)?;
    writeln!(file)?;
    write!(file, "{}", text)?;
    
    println!("Text gespeichert als: {}", txt_path.display());
    
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
    println!("Obsidian geöffnet mit: {}", file_name);
    Ok(())
}

fn ask_to_open_obsidian() -> bool {
    let output = Command::new("yad")
        .args(&[
            "--title=Obsidian öffnen",
            "--text=Möchten Sie Obsidian öffnen?",
            "--button=Ja:0",
            "--button=Nein:1",
            "--center",
            "--width=300",
            "--height=100"
        ])
        .output()
        .expect("Failed to execute yad");

    output.status.success()
}


fn main() {
    if let Err(e) = monitor_clipboard() {
        eprintln!("Fehler: {}", e);
    }
}
