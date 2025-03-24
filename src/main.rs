mod agent;
mod tools;

use agent::Agent;
use std::io::{BufRead, Read};
use std::path::Path;

fn read_input() -> String {
    eprint!("\nYou: ");
    let mut input = String::new();
    loop {
        let _ = std::io::stdin().lock().read_line(&mut input);
        if input.trim_start().starts_with(":") || input.ends_with("\n\n") {
            break;
        }
    }
    input.trim().to_string()
}

fn set_args(chat: &mut Agent, key: &str, value: &str) {
    match key.trim() {
        "model" => {
            chat.set_model(value);
        }
        "temperature" | "temp" => {
            if let Ok(temperature) = value.parse::<f64>() {
                chat.set_temperature(temperature);
            } else {
                println!("Invalid temperature value: {}", value);
            }
        }
        "top_p" => {
            if let Ok(top_p) = value.parse::<f64>() {
                chat.set_top_p(top_p);
            } else {
                println!("Invalid top_p value: {}", value);
            }
        }
        "top_k" => {
            if let Ok(top_k) = value.parse::<u64>() {
                chat.set_top_k(top_k);
            } else {
                println!("Invalid top_k value: {}", value);
            }
        }
        "frequency_penalty" => {
            if let Ok(frequency_penalty) = value.parse::<f64>() {
                chat.set_frequency_penalty(frequency_penalty);
            } else {
                println!("Invalid frequency_penalty value: {}", value);
            }
        }
        "max_tokens" => {
            if let Ok(max_tokens) = value.parse::<u64>() {
                chat.set_max_tokens(max_tokens);
            } else {
                println!("Invalid max_tokens value: {}", value);
            }
        }
        _ => {
            println!("Unknown key: {}", key);
        }
    }
}

fn run(chat: &mut Agent) {
    chat.clear_messages();
    loop {
        let input = read_input();
        if input == ":exit" || input == ":quit" || input == ":q" {
            break;
        }
        if input == ":list" || input == ":l" {
            let models = chat.list_models();
            for model in models {
                println!("{}", model);
            }
            continue;
        }
        if input.starts_with(":set ") || input.starts_with(":s ") {
            let args = input.split_whitespace().skip(1).collect::<Vec<&str>>();
            if args.len() < 2 {
                println!("Usage: :set <key> <value>");
            } else {
                set_args(chat, args[0], args[1]);
            }
            continue;
        }
        if input == ":clear" || input == ":c" {
            chat.clear_messages();
            continue;
        }
        if input == ":show" {
            println!("{:?}", chat.get_config());
            continue;
        }
        if input.is_empty() {
            continue;
        }
        chat.add_message("user", &input);
        if let Ok(response) = chat.post() {
            chat.add_message("assistant", &response);
        }
    }
}

fn main() {
    let tools = tools::all_tools();
    for tool in tools {
        println!("{}", serde_json::to_string_pretty(&tool.schema()).unwrap());
    }

    let config_file = std::env::args().nth(1).unwrap_or("config.json".to_string());
    let mut chat_agent = Agent::new_with_config_file(Path::new(&config_file)).unwrap();
    run(&mut chat_agent);
}
