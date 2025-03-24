use anyhow::Result;
use bytes::Bytes;
use futures_util::stream::StreamExt;
use reqwest;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use serde_json;
use std::io::BufRead;
use tokio;

pub struct Agent {
    url: String,
    token: String,
    config: serde_json::Value,
    system_prompt: String,
    messages: AllocRingBuffer<serde_json::Value>,
    async_runtime: tokio::runtime::Runtime,
}

impl Agent {
    pub fn new(config: serde_json::Value) -> Self {
        let mut this = Self {
            url: config["url"].as_str().unwrap().to_string(),
            token: std::env::var("TOKEN").unwrap_or(config["token"].as_str().unwrap().to_string()),
            config: serde_json::json!({
                "model": config["model"].as_str().unwrap(),
                "stream": true,
                "max_tokens": config["max_tokens"].as_u64().unwrap_or(4096),
            }),
            system_prompt: config["system_prompt"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            messages: AllocRingBuffer::new(config["max_history"].as_u64().unwrap_or(20) as usize),
            async_runtime: tokio::runtime::Runtime::new().unwrap(),
        };
        if let Some(value) = config.get("temperature") {
            this.config["temperature"] = value.clone();
        }
        if let Some(value) = config.get("top_p") {
            this.config["top_p"] = value.clone();
        }
        if let Some(value) = config.get("top_k") {
            this.config["top_k"] = value.clone();
        }
        if let Some(value) = config.get("frequency_penalty") {
            this.config["frequency_penalty"] = value.clone();
        }
        this
    }

    pub fn new_with_config_file(config_file: &std::path::Path) -> Result<Self> {
        Ok(Self::new(serde_json::from_reader(std::fs::File::open(
            config_file,
        )?)?))
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(serde_json::json!({
            "role": role,
            "content": content
        }));
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    pub fn post(&self) -> Result<String> {
        let mut messages = Vec::new();
        if !self.system_prompt.is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": self.system_prompt
            }));
        }
        messages.extend(self.messages.iter().cloned());
        let mut data = self.config.clone();
        data["messages"] = serde_json::Value::Array(messages);
        self.async_runtime
            .block_on(post_request(&self.url, &self.token, &data))
    }

    pub fn list_models(&self) -> Vec<String> {
        self.async_runtime
            .block_on(get_model_list(&self.url, &self.token))
            .unwrap_or_default()
    }
    
    pub fn get_config(&self) -> &serde_json::Value {
        &self.config
    }

    pub fn set_model(&mut self, model: &str) {
        self.config["model"] = model.into()
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        self.config["temperature"] = temperature.into()
    }

    pub fn set_top_p(&mut self, top_p: f64) {
        self.config["top_p"] = top_p.into()
    }

    pub fn set_top_k(&mut self, top_k: u64) {
        self.config["top_k"] = top_k.into()
    }

    pub fn set_frequency_penalty(&mut self, frequency_penalty: f64) {
        self.config["frequency_penalty"] = frequency_penalty.into()
    }

    pub fn set_max_tokens(&mut self, max_tokens: u64) {
        self.config["max_tokens"] = max_tokens.into()
    }
}

struct Reply {
    reasoning_content: String,
    content: String,
}

impl Default for Reply {
    fn default() -> Self {
        Self {
            reasoning_content: String::new(),
            content: String::new(),
        }
    }
}

async fn get_model_list(url: &str, token: &str) -> Result<Vec<String>> {
    let url = url.replace("chat/completions", "models");
    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(token)
        .send()
        .await?
        .text()
        .await?;
    let json: serde_json::Value = serde_json::from_str(&response)?;
    let models = json["data"]
        .as_array()
        .ok_or(anyhow::anyhow!("Invalid response"))?
        .iter()
        .filter_map(|model| model["id"].as_str().map(|id| id.trim().to_string()))
        .collect();
    Ok(models)
}

fn parse_response_line(line: Result<String>) -> Result<Reply> {
    let json: serde_json::Value = serde_json::from_str(&line?.trim().trim_start_matches("data: "))?;
    let reasoning_content = json["choices"][0]["delta"]["reasoning_content"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let content = json["choices"][0]["delta"]["content"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    Ok(Reply {
        reasoning_content,
        content,
    })
}

fn parse_response(response: reqwest::Result<Bytes>, is_reasoning: &mut bool) -> String {
    if let Ok(response) = response {
        response
            .lines()
            .map(|line| {
                let reply = parse_response_line(line.map_err(Into::into)).unwrap_or_default();
                let mut delta = String::new();
                if !reply.reasoning_content.is_empty() {
                    if !*is_reasoning {
                        *is_reasoning = true;
                        delta.push_str("<think>\n");
                    }
                    delta.push_str(&reply.reasoning_content);
                }
                if !reply.content.is_empty() {
                    if *is_reasoning {
                        *is_reasoning = false;
                        delta.push_str("</think>\n");
                    }
                    delta.push_str(&reply.content);
                }
                eprint!("{}", delta);
                delta
            })
            .collect::<Vec<_>>()
            .join("")
    } else {
        String::new()
    }
}

async fn post_request(url: &str, token: &str, data: &serde_json::Value) -> Result<String> {
    let mut is_reasoning = false;
    let text = reqwest::Client::new()
        .post(url)
        .bearer_auth(token)
        .json(&data)
        .send()
        .await?
        .bytes_stream()
        .map(|bytes| parse_response(bytes, &mut is_reasoning))
        .collect::<Vec<_>>()
        .await
        .join("");
    Ok(text)
}
