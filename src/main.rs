use noters::{config::Config, error::{Result, NoterError}, note::NotesManager};
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(global = true, short, long)]
    verbose: bool,
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
    let cli = Cli::parse();
    
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    } else {
        env_logger::init();
    }

    let config = Config::load()?;
    let notes_manager = NotesManager::new(config)?;

    match cli.command {
        Some(Commands::New { title }) => {
            let title = title.unwrap_or_else(|| noters::utils::get_input("Note title: ").trim().to_string());
            let title = title.trim();
            if title.is_empty() {
                println!("{}", "Error: Title cannot be empty".red());
                return Ok(());
            }
            
            let title_without_ext = match std::path::Path::new(&title).extension() {
                Some(ext) => title.trim_end_matches(&format!(".{}", ext.to_str().unwrap_or(""))),
                None => title,
            };
            
            notes_manager.create_note(title_without_ext)?;
            println!("{}", "Note created successfully.".green());
        }
        Some(Commands::List) => {
            let notes = notes_manager.list_notes()?;
            if notes.is_empty() {
                println!("{}", "No notes found.".yellow());
            } else {
                for note in notes {
                    println!("{} {} {}", 
                        format!("[{}]", note.id).cyan(),
                        note.title.bright_white(),
                        format!("({})", note.filename).dimmed()
                    );
                }
            }
        }
        Some(Commands::Delete { id }) => {
            match notes_manager.delete_note(id)? {
                true => println!("{}", "Note deleted successfully.".green()),
                false => println!("{}", "Note not found.".red()),
            }
        }
        Some(Commands::Edit { id }) => {
            match notes_manager.edit_note(id) {
                Ok(_) => println!("{}", "Note edited successfully.".green()),
                Err(NoterError::EditorNotFound) => {
                    println!("{}", "No editor configured. Set $EDITOR environment variable or specify 'editor' in config.toml".red());
                }
                Err(NoterError::NoteNotFound(_)) => println!("{}", "Note not found.".red()),
                Err(e) => println!("{} {}", "Error editing note:".red(), e),
            }
        }
        Some(Commands::Export { dir }) => {
            if let Some(ref dir) = dir {
                if !dir.exists() {
                    std::fs::create_dir_all(dir).map_err(|e| {
                        println!("{} {}", "Failed to create export directory:".red(), e);
                        e
                    })?;
                } else if !dir.is_dir() {
                    println!("{} {}", "Error:".red(), format!("{} is not a directory", dir.display()));
                    return Ok(());
                }
            }

            match notes_manager.export_notes(dir.as_deref()) {
                Ok((success_count, total_count)) => {
                    if success_count == 0 && total_count == 0 {
                        println!("{}", "No notes to export.".yellow());
                    } else if success_count == total_count {
                        println!("{}", format!("Successfully exported all {} notes.", total_count).green());
                    } else {
                        println!("{}", format!("Exported {}/{} notes successfully.", success_count, total_count).yellow());
                        println!("{}", "Check the application logs for details about failed exports.".yellow());
                    }
                }
                Err(e) => println!("{} {}", "Error during export:".red(), e),
            }
        }
        Some(Commands::Search { query }) => {
            let results = notes_manager.search_notes(&query)?;
            if results.is_empty() {
                println!("{}", "No matching notes found.".yellow());
            } else {
                for note in results {
                    println!("{} {} {}", 
                        format!("[{}]", note.id).cyan(),
                        note.title.bright_white(),
                        format!("({})", note.filename).dimmed()
                    );
                }
            }
        }
        None => print_usage(),
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
