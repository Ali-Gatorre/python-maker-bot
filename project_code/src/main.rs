use std::io::{self, Write};
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rand::seq::SliceRandom;

/// Structure de mémoire pour stocker les phrases apprises.
#[derive(Serialize, Deserialize)]
struct Memory {
    responses: HashMap<String, String>,
}

impl Memory {
    fn load() -> Self {
        fs::read_to_string("memory.json")
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or(Self { responses: HashMap::new() })
    }

    fn save(&self) {
        let _ = fs::write("memory.json", serde_json::to_string_pretty(self).unwrap());
    }

    fn clear(&mut self) {
        self.responses.clear();
        self.save();
    }
}

fn main() {
    println!("RustBot v1.0 – Chatbot complet avec apprentissage et commandes");
    println!("Commandes disponibles : /save, /clear, /help, quit\n");

    let mut memory = Memory::load();

    let greetings = [
        "Bonjour, comment allez-vous ?",
        "Content de vous revoir.",
        "Je suis prêt à discuter.",
    ];
    println!("RustBot : {}", greetings.choose(&mut rand::thread_rng()).unwrap());

    loop {
        print!("Vous : ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "quit" => {
                println!("RustBot : Au revoir !");
                memory.save();
                break;
            }
            "/save" => {
                memory.save();
                println!("RustBot : Mémoire sauvegardée.");
                continue;
            }
            "/clear" => {
                memory.clear();
                println!("RustBot : Mémoire effacée.");
                continue;
            }
            "/help" => {
                println!("Commandes : /save, /clear, /help, quit");
                continue;
            }
            _ => {}
        }

        if let Some(reply) = memory.responses.get(&input) {
            println!("RustBot : {}", reply);
        } else {
            println!("RustBot : Je ne connais pas cette phrase. Que devrais-je répondre ?");
            print!("Vous : ");
            io::stdout().flush().unwrap();

            let mut new_reply = String::new();
            io::stdin().read_line(&mut new_reply).unwrap();
            let new_reply = new_reply.trim().to_string();

            memory.responses.insert(input.clone(), new_reply.clone());
            memory.save();

            println!("RustBot : D'accord, je répondrai '{}' lorsque vous direz '{}'.", new_reply, input);
        }
    }
}
