#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use image::{RgbaImage, ImageBuffer};

    // Helper-Funktion für temporäres Testverzeichnis
    fn setup_test_dir() -> TempDir {
        tempfile::TempDir::new().expect("Failed to create temp directory")
    }

    // Helper-Funktion für Test-Image
    fn create_test_image() -> RgbaImage {
        ImageBuffer::from_fn(100, 100, |_, _| {
            image::Rgba([255, 0, 0, 255]) // Rotes Test-Bild
        })
    }

    #[test]
    fn test_generate_unique_filename() {
        let filename1 = generate_unique_filename("test", ".txt");
        let filename2 = generate_unique_filename("test", ".txt");
        
        assert!(filename1.starts_with("test_"));
        assert!(filename1.ends_with(".txt"));
        assert_ne!(filename1, filename2); // Prüfe Einzigartigkeit
    }

    #[test]
    fn test_calculate_hash() {
        let data1 = b"test data";
        let data2 = b"test data";
        let data3 = b"different data";
        
        let hash1 = calculate_hash(data1);
        let hash2 = calculate_hash(data2);
        let hash3 = calculate_hash(data3);
        
        assert_eq!(hash1, hash2); // Gleiche Daten = gleicher Hash
        assert_ne!(hash1, hash3); // Verschiedene Daten = verschiedene Hashes
    }

    #[test]
    fn test_create_markdown_file() {
        let temp_dir = setup_test_dir();
        let md_path = temp_dir.path().join("test.md");
        
        create_markdown_file(
            &md_path,
            "test_image.png",
            "2024-01-01 12:00"
        ).expect("Failed to create markdown file");
        
        let content = fs::read_to_string(&md_path)
            .expect("Failed to read markdown file");
        
        assert!(content.contains("## Screenshot vom 2024-01-01 12:00"));
        assert!(content.contains("![Screenshot](test_image.png)"));
    }

    #[test]
    fn test_save_image_and_markdown() {
        let temp_dir = setup_test_dir();
        std::env::set_var("HOME", temp_dir.path()); // Mock home directory
        
        let test_image = create_test_image();
        let result = save_image_and_markdown(&test_image);
        
        assert!(result.is_ok());
        if let Ok(md_path) = result {
            assert!(md_path.exists());
            // Prüfe ob PNG auch existiert
            let parent = md_path.parent().unwrap();
            let files = fs::read_dir(parent).unwrap();
            let has_png = files.filter_map(Result::ok)
                              .any(|entry| entry.path().extension().unwrap_or_default() == "png");
            assert!(has_png);
        }
    }

    #[test]
    fn test_save_text() {
        let temp_dir = setup_test_dir();
        std::env::set_var("HOME", temp_dir.path());
        
        let test_text = "This is a test text";
        let result = save_text(test_text);
        
        assert!(result.is_ok());
        
        // Prüfe ob Datei existiert und Inhalt korrekt ist
        let files = fs::read_dir(temp_dir.path().join("Nextcloud/Obsy"))
            .unwrap()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
            
        assert!(!files.is_empty());
        
        let md_file = &files[0];
        let content = fs::read_to_string(md_file.path()).unwrap();
        assert!(content.contains(test_text));
    }
}
