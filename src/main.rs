mod agent;
mod tools;

use agent::Agent;
use anyhow::Result;
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

fn invoke_tool(toolset: &tools::ToolSet, request: &str) -> Result<String> {
    let (tool, args) = parse_tool_invoke_json(request)?;
    let result = toolset.invoke(&tool, &args)?;
    Ok(result)
}

fn run_pipeline(agent: &mut Agent, toolset: &tools::ToolSet) {
    loop {
        println!("\n============= LLM RESPONSE =============");
        if let Ok(response) = agent.post() {
            agent.add_message("assistant", &response);
            let invoke_result = format!(
                "\n============= TOOL OUTPUT =============\n{}\n",
                invoke_tool(toolset, &response).unwrap_or_else(|e| format!("Error: {}", e))
            );
            println!("{}", invoke_result);
            agent.add_message("user", &invoke_result);
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

fn parse_tool_invoke_json(input: &str) -> Result<(String, serde_json::Value)> {
    let input = input
        .rsplit_once("[[[[INVOKE]]]]")
        .ok_or(anyhow::anyhow!(
            "Invalid input. Cannot find invoke heading of [[[[INVOKE]]]]"
        ))?
        .1
        .split_once("```json")
        .ok_or(anyhow::anyhow!(
            r#"Invalid input. Cannot find invoke begin of "```json"."#
        ))?
        .1
        .rsplit_once("```")
        .ok_or(anyhow::anyhow!(
            r#"Invalid input. Cannot find invoke end of "```"."#
        ))?
        .0;
    let invoke: serde_json::Value = serde_json::from_str(input)?;
    let invoke = invoke
        .as_object()
        .ok_or(anyhow::anyhow!("Input is not a valid JSON object"))?;
    if invoke.len() != 1 {
        Err(anyhow::anyhow!(
            "Invalid input. You can only call one tool at a time."
        ))
    } else {
        let (tool, args) = invoke.iter().next().ok_or(anyhow::anyhow!(
            "Invalid input. JSON object must have exactly one key"
        ))?;
        Ok((tool.clone(), args.clone()))
    }
}

fn main() {
    let mut toolset = tools::ToolSet::new();
    toolset.register_tool(tools::Tree::create());
    toolset.register_tool(tools::Read::create());

    let help = serde_json::to_string_pretty(&toolset.get_help()).unwrap();
    let config_file = std::env::args().nth(1).unwrap_or("config.json".to_string());
    let mut chat_agent = Agent::new_with_config_file(Path::new(&config_file)).unwrap();
    let prompt = format!(
        r#"
I would like you to help read a codebase. There are some tools you can use.
You can call them by providing the tool name and the arguments in JSON.
Here are the tools:
{}

Now tell me what you want to do and I will return you the output of the tool.
Please output the special heading and than follow the format (must be wrapped in triple backticks):

[[[[INVOKE]]]]
```json
{{
  "tool_name": {{
    "arg1": value1,
    "arg2": value2
  }}
}}
```

Please note that you can only call one tool at a time.
"#,
        help
    );
    println!(
        "\n============= PROMPT =============\n{}\n==================================\n",
        prompt
    );
    chat_agent.set_system_prompt(&prompt);

    run_pipeline(&mut chat_agent, &toolset);
}
