use std::io::{self, Write};
use std::fs;
use crate::api::{self, Message};
use crate::python_exec::CodeExecutor;
use crate::utils::extract_python_code;
use crate::logger::{Logger, SessionMetrics};
use colored::*;

// Fonction publique utilisable depuis main.rs affichant un bandeau de bienvenue 
pub fn print_banner() {
    println!("{}", "====================================".bright_cyan());
    println!("{}", "        PYTHON MAKER BOT v0.2       ".bright_cyan().bold());
    println!("{}", "====================================".bright_cyan());
    println!("{}", " AI-Powered Python Code Generator".bright_white());
    println!("{}\n", " Type /help for commands or /quit to exit".dimmed());
}

// Fonction utilitaire pour poser des question à l'utilisateur et récupérer la réponse
pub fn ask_user(question: &str) -> String {
    print!("{question}");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

// Fonction utilitaire qui pose une une question oui/non en utilisant ask_user
// Elle renvoi un booléen 
pub fn confirm(question: &str) -> bool {
    let ans = ask_user(&format!("{question} (o/n) : "));
    ans.to_lowercase().starts_with('o')
}

// Fonction d'affichage pour le code python généré 
pub fn display_code(code: &str) {
    println!("\n{}", "━━━━━━━━━━━ Generated Code ━━━━━━━━━━━".bright_green().bold());
    // Simple syntax highlighting for Python
    for line in code.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            println!("{}", line.bright_black());
        } else if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
            println!("{}", line.bright_yellow());
        } else if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
            println!("{}", line.bright_magenta());
        } else {
            println!("{}", line);
        }
    }
    println!("{}\n", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_green());
}

// Boucle interactive : affiche le bandeau de lancement
pub async fn start_repl() {
    print_banner();

    let executor = CodeExecutor::new("generated").expect("Impossible de créer le dossier");
    let logger = Logger::new("logs").expect("Failed to create logger");
    let mut metrics = SessionMetrics::new();
    
    // Conversation history for multi-turn refinement
    let mut conversation_history: Vec<Message> = Vec::new();
    let mut last_generated_code = String::new();

    loop {
        let prompt = ask_user("> ");

        if prompt == "/quit" || prompt == "/exit" {
            println!("Goodbye!");
            break;
        }

        if prompt == "/help" {
            println!("\n{}", "Available Commands:".bright_cyan().bold());
            println!("  {}  - Exit the program", "/quit, /exit".green());
            println!("  {}         - Show this help", "/help".green());
            println!("  {}        - Clear conversation history", "/clear".green());
            println!("  {}       - Refine the last generated code", "/refine".green());
            println!("  {} <file> - Save last code to a file", "/save".green());
            println!("  {}      - Show conversation history", "/history".green());
            println!("  {}        - Show session statistics", "/stats".green());
            println!();
            continue;
        }

        if prompt == "/stats" {
            metrics.display();
            continue;
        }

        if prompt == "/clear" {
            conversation_history.clear();
            last_generated_code.clear();
            println!("{}", "✓ Conversation history cleared.".green());
            continue;
        }

        if prompt == "/history" {
            if conversation_history.is_empty() {
                println!("{}", "No conversation history yet.".yellow());
            } else {
                println!("\n{}", "Conversation History:".bright_cyan().bold());
                for (i, msg) in conversation_history.iter().enumerate() {
                    let role_color = if msg.role == "user" {
                        msg.role.bright_blue()
                    } else {
                        msg.role.bright_green()
                    };
                    println!("\n{}. [{}]", i + 1, role_color);
                    let preview = if msg.content.len() > 100 {
                        format!("{}...", &msg.content[..100])
                    } else {
                        msg.content.clone()
                    };
                    println!("{}", preview.dimmed());
                }
                println!();
            }
            continue;
        }

        if prompt.starts_with("/save") {
            if last_generated_code.is_empty() {
                println!("{}", "No code to save. Generate some code first!".yellow());
                continue;
            }
            
            let parts: Vec<&str> = prompt.split_whitespace().collect();
            let filename = if parts.len() > 1 {
                parts[1].to_string()
            } else {
                ask_user("Enter filename (e.g., script.py): ")
            };
            
            if filename.is_empty() {
                println!("{}", "Save cancelled.".yellow());
                continue;
            }
            
            match fs::write(&filename, &last_generated_code) {
                Ok(_) => println!("{} {}", "✓ Code saved to:".green(), filename.bright_white()),
                Err(e) => println!("{} {}", "✗ Failed to save file:".red(), e),
            }
            continue;
        }

        if prompt == "/refine" {
            if last_generated_code.is_empty() {
                println!("{}", "No code to refine. Generate some code first!".yellow());
                continue;
            }
            print!("{}", "What would you like to change or add? ".cyan());
            io::stdout().flush().unwrap();
            let mut refinement = String::new();
            io::stdin().read_line(&mut refinement).unwrap();
            let refinement = refinement.trim();
            
            if refinement.is_empty() {
                continue;
            }
            
            // Add refinement request to history
            conversation_history.push(Message {
                role: "user".to_string(),
                content: format!("Please refine the previous code: {}", refinement),
            });
        } else {
            // Regular prompt - add to history
            conversation_history.push(Message {
                role: "user".to_string(),
                content: prompt.clone(),
            });
        }

        // Log the request
        let _ = logger.log_api_request(&conversation_history.last().unwrap().content);
        metrics.total_requests += 1;

        // Call Hugging Face with conversation history
        match api::generate_code_with_history(conversation_history.clone()).await {
            Ok(raw_response) => {
                // Log the response
                let _ = logger.log_api_response(&raw_response);
                
                // Extract clean Python code from the response
                let code = extract_python_code(&raw_response);
                last_generated_code = code.clone();
                
                // Add assistant response to history
                conversation_history.push(Message {
                    role: "assistant".to_string(),
                    content: code.clone(),
                });
                
                display_code(&code);

                if confirm("Execute this script?") {
                    // Check for dependencies
                    let deps = executor.detect_dependencies(&code);
                    if !deps.is_empty() {
                        println!("\n{} {}", 
                            "⚠️  Detected non-standard dependencies:".yellow(),
                            deps.join(", ").bright_yellow());
                        if confirm("Install these dependencies?") {
                            if let Err(e) = executor.install_packages(&deps) {
                                println!("{} {}", "⚠️  Failed to install dependencies:".yellow(), e);
                                println!("{}", "Proceeding anyway...".dimmed());
                            }
                        }
                    }

                    match executor.write_and_run(&code) {
                        Ok(result) => {
                            let success = result.stderr.is_empty() || !result.stderr.contains("Error");
                            if success {
                                metrics.successful_executions += 1;
                            } else {
                                metrics.failed_executions += 1;
                            }
                            
                            let _ = logger.log_execution(success, &result.stdout);
                            
                            println!("\n{}", "━━━━━━━━━━━ Execution Result ━━━━━━━━━━━".bright_blue().bold());
                            println!("{} {:?}", "Script saved at:".dimmed(), result.script_path);
                            if !result.stdout.is_empty() {
                                println!("\n{}:", "STDOUT".green().bold());
                                println!("{}", result.stdout);
                            }
                            if !result.stderr.is_empty() {
                                println!("\n{}:", "STDERR".red().bold());
                                println!("{}", result.stderr);
                            }
                            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
                        }
                        Err(e) => {
                            metrics.failed_executions += 1;
                            let _ = logger.log_error(&format!("Execution error: {}", e));
                            println!("{} {}", "✗ Execution error:".red(), e);
                        }
                    }
                }
            }
            Err(e) => {
                metrics.api_errors += 1;
                let _ = logger.log_error(&format!("API error: {}", e));
                println!("{} {}", "✗ API error:".red(), e);
                // Remove the last user message if API call failed
                conversation_history.pop();
            }
        }
    }
    
    // Display session statistics on exit
    println!("\n{}", "Session ended.".bright_cyan());
    metrics.display();
}
