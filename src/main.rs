use noters::{config::Config, error::{Result, NoterError}, note::NotesManager};
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    New {
        #[arg(help = "Title of the new note")]
        title: Option<String>,
    },
    List,
    Delete {
        #[arg(help = "ID of the note to delete")]
        id: i64,
    },
    Edit {
        #[arg(help = "ID of the note to edit")]
        id: i64,
    },
    Export {
        #[arg(help = "Directory to export notes to")]
        dir: Option<PathBuf>,
    },
    Search {
        #[arg(help = "Search query")]
        query: String,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    let config = Config::load()?;
    let notes_manager = NotesManager::new(config)?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::New { title }) => {
            let title = title.unwrap_or_else(|| noters::utils::get_input("Note title: "));
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
        Some(Commands::List) => {
            let notes = notes_manager.list_notes()?;
            if notes.is_empty() {
                println!("No notes found.");
            } else {
                for note in notes {
                    println!("[{}] {} ({})", note.id, note.title, note.filename);
                }
            }
        }
        Some(Commands::Delete { id }) => {
            match notes_manager.delete_note(id)? {
                true => println!("Note deleted successfully."),
                false => println!("Note not found."),
            }
        }
        Some(Commands::Edit { id }) => {
            match notes_manager.edit_note(id) {
                Ok(_) => println!("Note edited successfully."),
                Err(NoterError::EditorNotFound) => {
                    println!("No editor configured. Set $EDITOR environment variable or specify 'editor' in config.toml");
                }
                Err(NoterError::NoteNotFound(_)) => println!("Note not found."),
                Err(e) => println!("Error editing note: {}", e),
            }
        }
        Some(Commands::Export { dir }) => {
            let export_dir = dir;

            if let Some(ref dir) = export_dir {
                if !dir.exists() {
                    if let Err(e) = std::fs::create_dir_all(dir) {
                        println!("Failed to create export directory: {}", e);
                        return Ok(());
                    }
                } else if !dir.is_dir() {
                    println!("Error: {} is not a directory", dir.display());
                    return Ok(());
                }
            }

            match notes_manager.export_notes(export_dir.as_deref()) {
                Ok((success_count, total_count)) => {
                    if success_count == 0 && total_count == 0 {
                        println!("No notes to export.");
                    } else if success_count == total_count {
                        println!("Successfully exported all {} notes.", total_count);
                    } else {
                        println!("Exported {}/{} notes successfully.", success_count, total_count);
                        println!("Check the application logs for details about failed exports.");
                    }
                }
                Err(e) => println!("Error during export: {}", e),
            }
        }
        Some(Commands::Search { query }) => {
            let results = notes_manager.search_notes(&query)?;
            if results.is_empty() {
                println!("No matching notes found.");
            } else {
                for note in results {
                    println!("[{}] {} ({})", note.id, note.title, note.filename);
                }
            }
        }
        None => {
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    const USAGE: &str = "Usage: noters <command> [args]";
    const COMMANDS: &[(&str, &str)] = &[
        ("new [title]", "Create a new note"),
        ("list", "List all notes"),
        ("delete <id>", "Delete a note by ID"),
        ("edit <id>", "Edit a note in your configured editor"),
        ("export [dir]", "Export all notes to directory (defaults to configured export dir)"),
        ("search <query>", "Search notes"),
    ];

    println!("\n{}", "╭─────────────────────────────────────╮".bright_blue());
    println!("{} {} {}", "│".bright_blue(), USAGE.bright_white().bold(), "│".bright_blue());
    println!("{}", "╰─────────────────────────────────────╯".bright_blue());
    
    println!("\n{}", "Commands:".bright_yellow().bold());
    
    for (cmd, description) in COMMANDS {
        println!("  {} {:<15} │ {}", "►".bright_green(), cmd.bright_cyan(), description);
    }
    println!("");
}
