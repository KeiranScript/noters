use noters::{config::Config, error::{Result, NoterError}, note::NotesManager};
use std::path::PathBuf;
use std::env;

fn main() -> Result<()> {
    env_logger::init();
    let config = Config::load()?;
    let notes_manager = NotesManager::new(config)?;

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str);

    match command {
        Some("new") => {
            let title = match args.get(2) {
                Some(t) => t.to_string(),
                None => noters::utils::get_input("Note title: "),
            };
            
            if let Some(extension) = std::path::Path::new(&title)
                .extension()
                .and_then(|ext| ext.to_str())
            {
                let title_without_ext = title.trim_end_matches(&format!(".{}", extension));
                notes_manager.create_note(title_without_ext)?;
            } else {
                notes_manager.create_note(&title)?;
            }
            println!("Note created successfully.");
        }
        Some("export") => {
            let id: i64 = args.get(2)
                .ok_or_else(|| {
                    println!("Usage: noters export <id> [output_path]");
                    NoterError::InvalidInput("No ID provided".to_string())
                })?
                .parse()
                .map_err(|_| {
                    println!("Invalid note ID. Usage: noters export <id> [output_path]");
                    NoterError::InvalidInput("Invalid ID format".to_string())
                })?;

            let output_path = args.get(3).map(PathBuf::from);
            match notes_manager.export_note(id, output_path.as_deref()) {
                Ok(path) => println!("Note exported successfully to: {}", path.display()),
                Err(NoterError::NoteNotFound(_)) => println!("Note not found."),
                Err(e) => println!("Error exporting note: {}", e),
            }
        }
        Some("list") => {
            let notes = notes_manager.list_notes()?;
            if notes.is_empty() {
                println!("No notes found.");
            } else {
                for note in notes {
                    println!("[{}] {} ({})", note.id, note.title, note.filename);
                }
            }
        }
        Some("delete") => {
            let id: i64 = args.get(2)
                .ok_or_else(|| {
                    println!("Usage: noters delete <id>");
                    NoterError::InvalidInput("No ID provided".to_string())
                })?
                .parse()
                .map_err(|_| {
                    println!("Invalid note ID. Usage: noters delete <id>");
                    NoterError::InvalidInput("Invalid ID format".to_string())
                })?;

            match notes_manager.delete_note(id)? {
                true => println!("Note deleted successfully."),
                false => println!("Note not found."),
            }
        }
        Some("edit") => {
            let id: i64 = args.get(2)
                .ok_or_else(|| {
                    println!("Usage: noters edit <id>");
                    NoterError::InvalidInput("No ID provided".to_string())
                })?
                .parse()
                .map_err(|_| {
                    println!("Invalid note ID. Usage: noters edit <id>");
                    NoterError::InvalidInput("Invalid ID format".to_string())
                })?;

            match notes_manager.edit_note(id) {
                Ok(_) => println!("Note edited successfully."),
                Err(NoterError::EditorNotFound) => {
                    println!("No editor configured. Set $EDITOR environment variable or specify 'editor' in config.toml");
                }
                Err(NoterError::NoteNotFound(_)) => println!("Note not found."),
                Err(e) => println!("Error editing note: {}", e),
            }
        }
        Some("search") => {
            if let Some(query) = args.get(2) {
                let results = notes_manager.search_notes(query)?;
                if results.is_empty() {
                    println!("No matching notes found.");
                } else {
                    for note in results {
                        println!("[{}] {} ({})", note.id, note.title, note.filename);
                    }
                }
            } else {
                println!("Usage: noters search <query>");
            }
        }
        Some(cmd) => {
            println!("Unknown command: {}", cmd);
            print_usage();
        }
        None => {
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage: noters <command> [args]");
    println!("Commands:");
    println!("  new [title]    Create a new note");
    println!("  list           List all notes");
    println!("  delete <id>    Delete a note by ID");
    println!("  edit <id>      Edit a note in your configured editor");
    println!("  export <id> [output_path]  Export decrypted note to file");
    println!("  search <query> Search notes");
}
