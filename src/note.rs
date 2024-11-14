use crate::config::Config;
use crate::crypto::Crypto;
use crate::db::{Database, NoteRecord};
use crate::error::{NoterError, Result};
use chrono::Local;
use log::info;
use std::fs;
use std::path::PathBuf;

pub struct NotesManager {
    config: Config,
    db: Database,
    crypto: Crypto,
    notes_dir: PathBuf,
}

impl NotesManager {
    pub fn new(config: Config) -> Result<Self> {
        let notes_dir = config.notes_dir.clone();
        fs::create_dir_all(&notes_dir)?;
        let db = Database::new(config.db_path.clone())?;
        let crypto = Crypto::new(&config.encryption_key);
        Ok(Self {
            config,
            db,
            crypto,
            notes_dir,
        })
    }

    pub fn create_note(&self, title: &str) -> Result<()> {
        if title.trim().is_empty() {
            return Err(NoterError::InvalidTitle("Title cannot be empty".to_string()));
        }

        let filename = self.format_filename(title);
        let file_path = self.notes_dir.join(&filename);

        let content = format!(
            "---\ntitle: {}\ndate: {}\n---\n\n",
            title,
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        let encrypted = self
            .crypto
            .encrypt(content.as_bytes())
            .map_err(|e| NoterError::Encryption(e.to_string()))?;
        fs::write(&file_path, encrypted)?;

        self.db.insert_note(title, &filename)?;
        info!("Created encrypted note: {} at {:?}", title, file_path);

        Ok(())
    }

    pub fn read_note(&self, id: i64) -> Result<String> {
        let note = self.db.get_note(id)?.ok_or_else(|| NoterError::NoteNotFound(id))?;
        let file_path = self.notes_dir.join(&note.filename);
        let encrypted = fs::read_to_string(file_path)?;
        let decrypted = self
            .crypto
            .decrypt(&encrypted)
            .map_err(|e| NoterError::Encryption(e.to_string()))?;
        String::from_utf8(decrypted).map_err(|e| NoterError::Encryption(e.to_string()))
    }

    pub fn edit_note(&self, id: i64) -> Result<()> {
        let note = self.db.get_note(id)?.ok_or_else(|| NoterError::NoteNotFound(id))?;
        let file_path = self.notes_dir.join(&note.filename);

        let encrypted_content = fs::read_to_string(&file_path)?;
        let decrypted_content = self.crypto
            .decrypt(&encrypted_content)
            .map_err(|e| NoterError::Encryption(e.to_string()))?;

        let temp_path = file_path.with_extension("temp");
        fs::write(&temp_path, &decrypted_content)?;

        let editor = self.config.editor.clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .ok_or_else(|| NoterError::EditorNotFound)?;

        let status = std::process::Command::new(editor)
            .arg(&temp_path)
            .status()
            .map_err(|e| NoterError::EditorError(e.to_string()))?;

        if !status.success() {
            fs::remove_file(&temp_path)?;
            return Err(NoterError::EditorError("Editor exited with non-zero status".to_string()));
        }

        let modified_content = fs::read(&temp_path)?;

        let encrypted = self.crypto
            .encrypt(&modified_content)
            .map_err(|e| NoterError::Encryption(e.to_string()))?;

        fs::write(&file_path, encrypted)?;

        fs::remove_file(&temp_path)?;

        Ok(())
    }

    pub fn list_notes(&self) -> Result<Vec<NoteRecord>> {
        self.db.get_all_notes()
    }

    pub fn search_notes(&self, query: &str) -> Result<Vec<NoteRecord>> {
        self.db.search_notes(query)
    }

    pub fn delete_note(&self, id: i64) -> Result<bool> {
        if let Some(note) = self.db.get_note(id)? {
            let file_path = self.notes_dir.join(&note.filename);
            if file_path.exists() {
                fs::remove_file(file_path)?;
            }
            self.db.delete_note(id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn format_filename(&self, title: &str) -> String {
        let safe_title = title.replace(|c: char| !c.is_alphanumeric() && c != '-', "-");
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        format!("{}-{}.{}", timestamp, safe_title, self.config.default_extension)
    }
}
