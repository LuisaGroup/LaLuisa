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
    if let Ok((tool, args)) = parse_tool_invoke_json(request) {
        let result = toolset.invoke(&tool, &args)?;
        Ok(result)
    } else if request.contains("[[[[DOCUMENT]]]]") {
        Ok(r#"File has been documented. Please keep up the good work!

Or, if you think you have finished your task and want to stop,
please just output a special token [[[[DONE]]]]."#
            .to_string())
    } else {
        Err(anyhow::anyhow!(
            r#"
I cannot find correct tool name or arguments in the request.
Please check the format of the request and try again.

If you think you have finished your task and want to stop,
please just output a special token [[[[DONE]]]].

Otherwise, please either continue to use the tool with [[[[INVOKE]]]] or
write the documentation with [[[[DOCUMENT]]]] without outputting any [[[[DONE]]]].
"#
        ))
    }
}

fn run_pipeline(agent: &mut Agent, toolset: &tools::ToolSet) {
    loop {
        println!("\n============= LLM RESPONSE =============");
        if let Ok(response) = agent.post() {
            if response.trim().ends_with("[[[[DONE]]]]") {
                println!("\n\nDone.");
                break;
            }
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
    let codebase = Path::new(&std::env::args().nth(2).unwrap_or(".".to_string()))
        .canonicalize()
        .unwrap();
    let prompt = format!(
        r#"
I would like you to help write documentation for importance interface, headers, and source files in a codebase:
{:?}

There are some tools you can use. You can call them by providing the tool name and the arguments in JSON.
Here are the tools:
{}

Now tell me what you want to do and I will return you the output of the tool.
Please output a special heading and than JSON requests following the format (must be wrapped in triple backticks):

[[[[INVOKE]]]]
```json
{{
  "<tool-nane>": {{
    "arg1": value1,
    "arg2": value2
  }}
}}
```

For example, if you want to read the contents of a file, you can use the `read` tool like this:
[[[[INVOKE]]]]
```json
{{
  "read": {{
    "path": "/path/to/file"
  }}
}}

Please note that you can only call **one** tool **once** at a time. Otherwise errors will be returned.

If you would like to document a file, please output a special [[[[DOCUMENT]]]] token and then the
documentation in the target language's standard format (or doxygen format as a fallback):
[[[[DOCUMENT]]]]
<file name here with path on a new line>

<<<<<<< SEARCH
LINE 00001: mod xxx;
LINE 00002: use yyyy;
======= REPLACE
/// Some description here
/// Some description here
mod xxx;
use yyyy;
>>>>>>> FINISH

<<<<<<< SEARCH
LINE 00123: fn foo() {{
======= REPLACE
/// Some description here
/// Some description here
fn foo() {{
...
>>>>>>> FINISH

Note that you **MUST** output the changes in the diff-style, with "<<<<<<< SEARCH" and "======= REPLACE" and ">>>>>>> FINISH" signs!!!

And you **MUST** keep the part between "<<<<<<< SEARCH" and "======= REPLACE" AS SMALL AS POSSIBLE!!! DO NOT INCLUDE THE WHOLE FILE CONTENTS!!!

You may want to look at README (if any) and make a plan first, determine all the files to be processed.
During each step, you should always be checking if you are on the right track.
Do not leave any files unprocessed. Remember to check and update the plan carefully.

Keep track of the files you have processed and the ones you have not.
"#,
        codebase.to_str(),
        help
    );
    println!(
        "\n============= PROMPT =============\n{}\n==================================\n",
        prompt
    );
    chat_agent.set_system_prompt(&prompt);

    run_pipeline(&mut chat_agent, &toolset);
}
