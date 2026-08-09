use super::auth::{valid_credentials, Credentials};
use super::external_http;
use super::network::{build_client, is_access_denied, network_error};
use super::tools::{self, ToolActivity};
use super::AssistantState;
use crate::app_config::AssistantConfig;
use crate::app_data::Storage;
use futures_util::StreamExt;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use tauri::{ipc::Channel, AppHandle, Manager};

const CODEX_API_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
const CODEX_MODELS_CLIENT_VERSION: &str = "0.137.0";
const MAX_TOOL_ROUNDS: usize = 24;
const MAX_TOOL_CALLS: usize = 64;
const BYTESTO4T_INSTRUCTIONS: &str = include_str!("codex_instructions.txt");

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantChatRequest {
    pub messages: Vec<AssistantMessage>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantReply {
    pub content: String,
    pub model: String,
    pub reasoning: Vec<String>,
    pub tool_activity: Vec<ToolActivity>,
    pub usage: AssistantUsage,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AssistantStreamEvent {
    RoundStarted { round: usize, finalizing: bool },
    ReasoningDelta { delta: String },
    OutputDelta { delta: String },
    ToolStarted { name: String, arguments: Value },
    ToolFinished { activity: ToolActivity },
}

#[derive(Clone, Debug)]
struct FunctionCall {
    call_id: String,
    name: String,
    arguments: String,
}

#[derive(Clone, Debug)]
struct ModelDescriptor {
    slug: String,
    instructions: Option<String>,
}

#[derive(Default)]
struct CollectedResponse {
    final_response: Value,
    text: String,
    reasoning_summary: String,
    calls: Vec<FunctionCall>,
}

pub async fn discover_models(
    config: &AssistantConfig,
    state: &AssistantState,
) -> Result<Vec<String>, String> {
    let credentials = valid_credentials(config).await?;
    let models = discover_model_catalog(config, &credentials, state).await?;
    cache_model_instructions(state, &models)?;
    Ok(models.into_iter().map(|model| model.slug).collect())
}

async fn discover_model_catalog(
    config: &AssistantConfig,
    credentials: &Credentials,
    state: &AssistantState,
) -> Result<Vec<ModelDescriptor>, String> {
    let url = format!("{CODEX_API_BASE_URL}/models?client_version={CODEX_MODELS_CLIENT_VERSION}");
    let (status, body) = if state.external_network_required.load(Ordering::SeqCst) {
        external_model_request(config, credentials, &url).await?
    } else {
        let client = build_client(config, 60)?;
        let mut request = client.get(&url).bearer_auth(&credentials.access_token);
        if let Some(account_id) = &credentials.account_id {
            request = request.header("ChatGPT-Account-ID", account_id);
        }
        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let body = response.text().await.map_err(|error| {
                    network_error("Could not read the ChatGPT model list", error)
                })?;
                (status, body)
            }
            Err(error) if is_access_denied(&error) => {
                state
                    .external_network_required
                    .store(true, Ordering::SeqCst);
                external_model_request(config, credentials, &url)
                    .await
                    .map_err(|helper_error| {
                        format!(
                            "ChatGPT model discovery failed: {error}. External VPN network helper failed: {helper_error}"
                        )
                    })?
            }
            Err(error) => {
                return Err(network_error("ChatGPT model discovery failed", error));
            }
        }
    };
    if status == 401 {
        return Err(
            "ChatGPT rejected the stored session. Disconnect and sign in again.".to_string(),
        );
    }
    if !(200..300).contains(&status) {
        return Err(format!(
            "ChatGPT model discovery failed with HTTP status {status}."
        ));
    }
    let payload: Value = serde_json::from_str(&body)
        .map_err(|error| format!("ChatGPT returned an invalid model list: {error}"))?;
    let mut models = Vec::new();
    if let Some(items) = payload.get("models").and_then(Value::as_array) {
        for item in items {
            if item.get("supported_in_api").and_then(Value::as_bool) == Some(false) {
                continue;
            }
            if let Some(slug) = item.get("slug").and_then(Value::as_str) {
                if !slug.is_empty()
                    && !models
                        .iter()
                        .any(|model: &ModelDescriptor| model.slug == slug)
                {
                    let instructions = item
                        .get("base_instructions")
                        .and_then(Value::as_str)
                        .or_else(|| {
                            item.get("model_messages")
                                .and_then(|messages| messages.get("instructions_template"))
                                .and_then(Value::as_str)
                        })
                        .filter(|instructions| !instructions.is_empty())
                        .map(str::to_string);
                    models.push(ModelDescriptor {
                        slug: slug.to_string(),
                        instructions,
                    });
                }
            }
        }
    }
    if models.is_empty() {
        Err("ChatGPT returned no models available to this account.".to_string())
    } else {
        Ok(models)
    }
}

async fn external_model_request(
    config: &AssistantConfig,
    credentials: &Credentials,
    url: &str,
) -> Result<(u16, String), String> {
    let mut headers = vec![(
        "Authorization".to_string(),
        format!("Bearer {}", credentials.access_token),
    )];
    if let Some(account_id) = &credentials.account_id {
        headers.push(("ChatGPT-Account-ID".to_string(), account_id.clone()));
    }
    let response = external_http::request("GET", url, headers, None, config, 60).await?;
    Ok((response.status, response.body))
}

fn cache_model_instructions(
    state: &AssistantState,
    models: &[ModelDescriptor],
) -> Result<(), String> {
    let mut cache = state
        .model_instructions
        .lock()
        .map_err(|error| error.to_string())?;
    for model in models {
        if let Some(instructions) = &model.instructions {
            cache.insert(model.slug.clone(), instructions.clone());
        }
    }
    Ok(())
}

fn cached_instructions(state: &AssistantState, model: &str) -> Result<Option<String>, String> {
    state
        .model_instructions
        .lock()
        .map_err(|error| error.to_string())
        .map(|cache| cache.get(model).cloned())
}

pub async fn chat(
    app_handle: AppHandle,
    state: &AssistantState,
    config: AssistantConfig,
    request: AssistantChatRequest,
    on_event: Channel<AssistantStreamEvent>,
) -> Result<AssistantReply, String> {
    validate_messages(&request.messages)?;
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    let credentials = valid_credentials(&config).await?;
    let configured_model = config.model.trim();
    let mut model = configured_model.to_string();
    let mut instructions = if configured_model.is_empty() {
        None
    } else {
        cached_instructions(state, configured_model)?
    };
    if model.is_empty() || instructions.is_none() {
        let catalog = discover_model_catalog(&config, &credentials, state).await?;
        cache_model_instructions(state, &catalog)?;
        if model.is_empty() {
            model = catalog
                .first()
                .map(|descriptor| descriptor.slug.clone())
                .ok_or_else(|| "No ChatGPT model is available.".to_string())?;
        }
        instructions = catalog
            .iter()
            .find(|descriptor| descriptor.slug == model)
            .and_then(|descriptor| descriptor.instructions.clone());
    }
    let instructions = instructions.unwrap_or_else(|| BYTESTO4T_INSTRUCTIONS.trim().to_string());

    let mut input = build_input(&app_handle, request.messages)?;
    let mut activities = Vec::new();
    let mut reasoning = Vec::new();
    let mut seen_calls = HashSet::new();
    let mut tool_call_count = 0;
    let mut bytecode_revision = 0;
    let mut completed_rounds = 0;
    let mut usage = AssistantUsage {
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
    };

    for round in 0..MAX_TOOL_ROUNDS {
        completed_rounds = round + 1;
        ensure_not_cancelled(state, generation)?;
        send_event(
            &on_event,
            AssistantStreamEvent::RoundStarted {
                round: round + 1,
                finalizing: false,
            },
        );
        let collected = send_response_request(
            &config,
            &credentials,
            &model,
            &instructions,
            &input,
            state,
            generation,
            true,
            Some(&on_event),
        )
        .await?;
        add_usage(&mut usage, &collected.final_response);
        push_reasoning_summary(&mut reasoning, &collected.reasoning_summary);

        if collected.calls.is_empty() {
            if collected.text.trim().is_empty() {
                return Err("ChatGPT completed without returning an answer.".to_string());
            }
            return Ok(AssistantReply {
                content: collected.text,
                model,
                reasoning,
                tool_activity: activities,
                usage,
            });
        }

        append_model_output(&mut input, &collected);
        for call in collected.calls {
            ensure_not_cancelled(state, generation)?;
            let arguments = serde_json::from_str(&call.arguments).unwrap_or_else(|_| json!({}));
            tool_call_count += 1;
            send_event(
                &on_event,
                AssistantStreamEvent::ToolStarted {
                    name: call.name.clone(),
                    arguments: arguments.clone(),
                },
            );
            let signature = tool_call_signature(&call.name, &arguments, bytecode_revision);
            let (output, activity) = if !seen_calls.insert(signature) {
                let output = "Tool loop guard: this exact call already completed. Use its previous result or inspect a different item.".to_string();
                let activity = ToolActivity {
                    name: call.name.clone(),
                    arguments: arguments.clone(),
                    success: false,
                    summary: output.clone(),
                };
                (output, activity)
            } else if tool_call_count > MAX_TOOL_CALLS {
                let output = format!(
                    "Tool budget exhausted after {MAX_TOOL_CALLS} calls. Synthesize the best answer from the results already available."
                );
                let activity = ToolActivity {
                    name: call.name.clone(),
                    arguments: arguments.clone(),
                    success: false,
                    summary: output.clone(),
                };
                (output, activity)
            } else {
                tools::execute(
                    app_handle.clone(),
                    &call.name,
                    arguments,
                    config.allow_bytecode_edits,
                )
                .await
            };
            if activity.success && tools::changes_loaded_bytecode(&call.name) {
                bytecode_revision += 1;
            }
            send_event(
                &on_event,
                AssistantStreamEvent::ToolFinished {
                    activity: activity.clone(),
                },
            );
            activities.push(activity);
            input.push(json!({
                "type": "function_call_output",
                "call_id": call.call_id,
                "output": output
            }));
        }
        if tool_call_count >= MAX_TOOL_CALLS {
            break;
        }
    }

    ensure_not_cancelled(state, generation)?;
    send_event(
        &on_event,
        AssistantStreamEvent::RoundStarted {
            round: completed_rounds + 1,
            finalizing: true,
        },
    );
    input.push(json!({
        "role": "developer",
        "content": [{
            "type": "input_text",
            "text": "The investigation budget is exhausted. Do not request more tools. Give the best concrete answer supported by the tool results above, and state any remaining uncertainty."
        }]
    }));
    let collected = send_response_request(
        &config,
        &credentials,
        &model,
        &instructions,
        &input,
        state,
        generation,
        false,
        Some(&on_event),
    )
    .await?;
    add_usage(&mut usage, &collected.final_response);
    push_reasoning_summary(&mut reasoning, &collected.reasoning_summary);
    if collected.text.trim().is_empty() {
        return Err(
            "ChatGPT completed without returning an answer after the tool investigation."
                .to_string(),
        );
    }
    Ok(AssistantReply {
        content: collected.text,
        model,
        reasoning,
        tool_activity: activities,
        usage,
    })
}

fn tool_call_signature(name: &str, arguments: &Value, bytecode_revision: usize) -> String {
    if tools::changes_loaded_bytecode(name) {
        format!("mutation:{name}:{arguments}")
    } else if tools::is_mutating(name) {
        format!("side-effect:{bytecode_revision}:{name}:{arguments}")
    } else {
        format!("read:{bytecode_revision}:{name}:{arguments}")
    }
}

fn send_event(channel: &Channel<AssistantStreamEvent>, event: AssistantStreamEvent) {
    let _ = channel.send(event);
}

fn push_reasoning_summary(reasoning: &mut Vec<String>, summary: &str) {
    let summary = summary.trim();
    if !summary.is_empty() {
        reasoning.push(summary.to_string());
    }
}

fn validate_messages(messages: &[AssistantMessage]) -> Result<(), String> {
    if messages.is_empty() {
        return Err("The assistant request has no messages.".to_string());
    }
    if messages.len() > 100 {
        return Err("The assistant conversation is too long. Start a new chat.".to_string());
    }
    if messages
        .iter()
        .any(|message| !matches!(message.role.as_str(), "user" | "assistant"))
    {
        return Err("Assistant messages must use the user or assistant role.".to_string());
    }
    if messages
        .iter()
        .map(|message| message.content.len())
        .sum::<usize>()
        > 500_000
    {
        return Err("The assistant conversation is too large. Start a new chat.".to_string());
    }
    Ok(())
}

fn build_input(
    app_handle: &AppHandle,
    messages: Vec<AssistantMessage>,
) -> Result<Vec<Value>, String> {
    let selected_item = {
        let storage = app_handle.state::<Storage>();
        let selected_item = storage
            .ui
            .lock()
            .map_err(|error| error.to_string())?
            .selected_item
            .clone();
        selected_item
    };
    let selection = selected_item
        .map(|item| {
            format!(
                "The current selection is {} at vector index {}.",
                item.typ, item.index
            )
        })
        .unwrap_or_else(|| "No bytecode item is currently selected.".to_string());
    let developer_prompt = format!(
        "{}\n\n## Session context\n\n{selection}",
        BYTESTO4T_INSTRUCTIONS.trim()
    );
    let mut input = vec![json!({
        "role": "developer",
        "content": [{"type":"input_text","text":developer_prompt}]
    })];
    for message in messages {
        let content_type = if message.role == "assistant" {
            "output_text"
        } else {
            "input_text"
        };
        input.push(json!({
            "role": message.role,
            "content": [{"type":content_type,"text":message.content}]
        }));
    }
    Ok(input)
}

fn request_headers(
    request: reqwest::RequestBuilder,
    credentials: &Credentials,
) -> reqwest::RequestBuilder {
    let mut request = request
        .bearer_auth(&credentials.access_token)
        .header("originator", "codex_cli_rs")
        .header("OpenAI-Beta", "responses=experimental")
        .header(header::ACCEPT, "text/event-stream")
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(account_id) = &credentials.account_id {
        request = request.header("chatgpt-account-id", account_id);
    }
    request
}

fn external_response_headers(credentials: &Credentials) -> Vec<(String, String)> {
    let mut headers = vec![
        (
            "Authorization".to_string(),
            format!("Bearer {}", credentials.access_token),
        ),
        ("originator".to_string(), "codex_cli_rs".to_string()),
        (
            "OpenAI-Beta".to_string(),
            "responses=experimental".to_string(),
        ),
        ("Accept".to_string(), "text/event-stream".to_string()),
        ("Content-Type".to_string(), "application/json".to_string()),
    ];
    if let Some(account_id) = &credentials.account_id {
        headers.push(("chatgpt-account-id".to_string(), account_id.clone()));
    }
    headers
}

async fn send_response_request(
    config: &AssistantConfig,
    credentials: &Credentials,
    model: &str,
    instructions: &str,
    input: &[Value],
    state: &AssistantState,
    generation: u64,
    allow_tools: bool,
    on_event: Option<&Channel<AssistantStreamEvent>>,
) -> Result<CollectedResponse, String> {
    let client = build_client(config, 180)?;
    let payload = build_response_payload(config, model, instructions, input, allow_tools);
    let url = format!("{CODEX_API_BASE_URL}/responses");
    if state.external_network_required.load(Ordering::SeqCst) {
        return external_response_request(
            config,
            credentials,
            &url,
            &payload,
            state,
            generation,
            on_event,
        )
        .await;
    }

    let response = match request_headers(client.post(&url), credentials)
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) if is_access_denied(&error) => {
            state
                .external_network_required
                .store(true, Ordering::SeqCst);
            return external_response_request(
                config,
                credentials,
                &url,
                &payload,
                state,
                generation,
                on_event,
            )
            .await;
        }
        Err(error) => {
            return Err(network_error("ChatGPT assistant request failed", error));
        }
    };
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        let body = response.text().await.unwrap_or_default();
        return Err(response_status_error(status, &body));
    }
    collect_sse(response, state, generation, on_event).await
}

fn build_response_payload(
    config: &AssistantConfig,
    model: &str,
    instructions: &str,
    input: &[Value],
    allow_tools: bool,
) -> Value {
    let mut payload = json!({
        "model": model,
        "input": input,
        "instructions": instructions,
        "store": false,
        "stream": true
    });
    if allow_tools {
        payload["tools"] = json!(tools::definitions(config.allow_bytecode_edits));
        payload["tool_choice"] = json!("auto");
        payload["parallel_tool_calls"] = json!(true);
    } else {
        payload["tools"] = json!([]);
        payload["tool_choice"] = json!("none");
        payload["parallel_tool_calls"] = json!(false);
    }
    if config.reasoning_effort != "none" && !config.reasoning_effort.is_empty() {
        payload["reasoning"] = json!({
            "effort": config.reasoning_effort,
            "summary": "auto"
        });
        payload["include"] = json!(["reasoning.encrypted_content"]);
    }
    payload
}

async fn external_response_request(
    config: &AssistantConfig,
    credentials: &Credentials,
    url: &str,
    payload: &Value,
    state: &AssistantState,
    generation: u64,
    on_event: Option<&Channel<AssistantStreamEvent>>,
) -> Result<CollectedResponse, String> {
    ensure_not_cancelled(state, generation)?;
    let body = serde_json::to_string(payload)
        .map_err(|error| format!("Could not encode the ChatGPT request: {error}"))?;
    let response = external_http::request(
        "POST",
        url,
        external_response_headers(credentials),
        Some(body),
        config,
        180,
    )
    .await?;
    ensure_not_cancelled(state, generation)?;
    if !(200..300).contains(&response.status) {
        return Err(response_status_error(response.status, &response.body));
    }
    collect_sse_text(&response.body, state, generation, on_event)
}

fn response_status_error(status: u16, _body: &str) -> String {
    match status {
        401 => "ChatGPT rejected the stored session. Disconnect and sign in again.".to_string(),
        429 => "ChatGPT rate limit reached. Wait a moment and try again.".to_string(),
        _ => format!("ChatGPT assistant request failed with HTTP status {status}."),
    }
}

async fn collect_sse(
    response: reqwest::Response,
    state: &AssistantState,
    generation: u64,
    on_event: Option<&Channel<AssistantStreamEvent>>,
) -> Result<CollectedResponse, String> {
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut collected = CollectedResponse::default();
    while let Some(chunk) = stream.next().await {
        ensure_not_cancelled(state, generation)?;
        let chunk = chunk.map_err(|error| network_error("ChatGPT stream failed", error))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(newline) = buffer.find('\n') {
            let line: String = buffer.drain(..=newline).collect();
            process_sse_line(line.trim(), &mut collected, on_event)?;
        }
    }
    if !buffer.trim().is_empty() {
        process_sse_line(buffer.trim(), &mut collected, on_event)?;
    }
    hydrate_and_emit(&mut collected, on_event);
    Ok(collected)
}

fn collect_sse_text(
    body: &str,
    state: &AssistantState,
    generation: u64,
    on_event: Option<&Channel<AssistantStreamEvent>>,
) -> Result<CollectedResponse, String> {
    let mut collected = CollectedResponse::default();
    for line in body.lines() {
        ensure_not_cancelled(state, generation)?;
        process_sse_line(line.trim(), &mut collected, on_event)?;
    }
    hydrate_and_emit(&mut collected, on_event);
    Ok(collected)
}

fn process_sse_line(
    line: &str,
    collected: &mut CollectedResponse,
    on_event: Option<&Channel<AssistantStreamEvent>>,
) -> Result<(), String> {
    let Some(data) = line.strip_prefix("data:").map(str::trim) else {
        return Ok(());
    };
    if data.is_empty() || data == "[DONE]" {
        return Ok(());
    }
    let event: Value = serde_json::from_str(data)
        .map_err(|error| format!("ChatGPT returned an invalid stream event: {error}"))?;
    match event
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "response.output_text.delta" => {
            if let Some(delta) = event.get("delta").and_then(Value::as_str) {
                collected.text.push_str(delta);
                if let Some(channel) = on_event {
                    send_event(
                        channel,
                        AssistantStreamEvent::OutputDelta {
                            delta: delta.to_string(),
                        },
                    );
                }
            }
        }
        "response.reasoning_summary_text.delta" => {
            if let Some(delta) = event.get("delta").and_then(Value::as_str) {
                collected.reasoning_summary.push_str(delta);
                if let Some(channel) = on_event {
                    send_event(
                        channel,
                        AssistantStreamEvent::ReasoningDelta {
                            delta: delta.to_string(),
                        },
                    );
                }
            }
        }
        "response.output_item.done" => {
            if let Some(item) = event.get("item") {
                push_function_call(item, &mut collected.calls);
            }
        }
        "response.completed" | "response.done" => {
            collected.final_response = event.get("response").cloned().unwrap_or(Value::Null);
        }
        "error" => {
            return Err("ChatGPT reported a streaming error.".to_string());
        }
        _ => {}
    }
    Ok(())
}

fn hydrate_and_emit(
    collected: &mut CollectedResponse,
    on_event: Option<&Channel<AssistantStreamEvent>>,
) {
    let had_text = !collected.text.is_empty();
    let had_reasoning = !collected.reasoning_summary.is_empty();
    hydrate_from_final_response(collected);
    if let Some(channel) = on_event {
        if !had_text && !collected.text.is_empty() {
            send_event(
                channel,
                AssistantStreamEvent::OutputDelta {
                    delta: collected.text.clone(),
                },
            );
        }
        if !had_reasoning && !collected.reasoning_summary.is_empty() {
            send_event(
                channel,
                AssistantStreamEvent::ReasoningDelta {
                    delta: collected.reasoning_summary.clone(),
                },
            );
        }
    }
}

fn hydrate_from_final_response(collected: &mut CollectedResponse) {
    let Some(output) = collected
        .final_response
        .get("output")
        .and_then(Value::as_array)
    else {
        return;
    };
    let mut final_text = String::new();
    let mut final_reasoning = String::new();
    for item in output {
        push_function_call(item, &mut collected.calls);
        if item.get("type").and_then(Value::as_str) == Some("message") {
            if let Some(content) = item.get("content").and_then(Value::as_array) {
                for part in content {
                    if part.get("type").and_then(Value::as_str) == Some("output_text") {
                        if let Some(text) = part.get("text").and_then(Value::as_str) {
                            final_text.push_str(text);
                        }
                    }
                }
            }
        } else if item.get("type").and_then(Value::as_str) == Some("reasoning") {
            if let Some(summary) = item.get("summary").and_then(Value::as_array) {
                for part in summary {
                    if part.get("type").and_then(Value::as_str) == Some("summary_text") {
                        if let Some(text) = part.get("text").and_then(Value::as_str) {
                            final_reasoning.push_str(text);
                        }
                    }
                }
            }
        }
    }
    if collected.text.is_empty() {
        collected.text = final_text;
    }
    if collected.reasoning_summary.is_empty() {
        collected.reasoning_summary = final_reasoning;
    }
}

fn push_function_call(item: &Value, calls: &mut Vec<FunctionCall>) {
    if item.get("type").and_then(Value::as_str) != Some("function_call") {
        return;
    }
    let call_id = item
        .get("call_id")
        .or_else(|| item.get("id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if call_id.is_empty() || calls.iter().any(|call| call.call_id == call_id) {
        return;
    }
    calls.push(FunctionCall {
        call_id: call_id.to_string(),
        name: item
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        arguments: item
            .get("arguments")
            .and_then(Value::as_str)
            .unwrap_or("{}")
            .to_string(),
    });
}

fn append_model_output(input: &mut Vec<Value>, collected: &CollectedResponse) {
    let mut call_ids = HashSet::new();
    if let Some(output) = collected
        .final_response
        .get("output")
        .and_then(Value::as_array)
    {
        for item in output {
            if item.get("type").and_then(Value::as_str) == Some("function_call") {
                if let Some(call_id) = item.get("call_id").and_then(Value::as_str) {
                    call_ids.insert(call_id.to_string());
                }
            }
            input.push(item.clone());
        }
    }
    for call in &collected.calls {
        if call_ids.insert(call.call_id.clone()) {
            input.push(json!({
                "type":"function_call",
                "call_id":call.call_id,
                "name":call.name,
                "arguments":call.arguments
            }));
        }
    }
}

fn add_usage(usage: &mut AssistantUsage, response: &Value) {
    if let Some(round) = response.get("usage") {
        usage.input_tokens += round
            .get("input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        usage.output_tokens += round
            .get("output_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        usage.total_tokens += round
            .get("total_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0);
    }
}

fn ensure_not_cancelled(state: &AssistantState, generation: u64) -> Result<(), String> {
    if state.generation.load(Ordering::SeqCst) != generation {
        Err("Assistant request cancelled.".to_string())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_parser_collects_text_and_tool_calls() {
        let mut collected = CollectedResponse::default();
        process_sse_line(
            r#"data: {"type":"response.output_text.delta","delta":"hello"}"#,
            &mut collected,
            None,
        )
        .unwrap();
        process_sse_line(
            r#"data: {"type":"response.reasoning_summary_text.delta","delta":"Inspecting the selected function."}"#,
            &mut collected,
            None,
        )
        .unwrap();
        process_sse_line(
            r#"data: {"type":"response.output_item.done","item":{"type":"function_call","call_id":"call_1","name":"get_dashboard_info","arguments":"{}"}}"#,
            &mut collected,
            None,
        )
        .unwrap();
        assert_eq!(collected.text, "hello");
        assert_eq!(
            collected.reasoning_summary,
            "Inspecting the selected function."
        );
        assert_eq!(collected.calls.len(), 1);
        assert_eq!(collected.calls[0].name, "get_dashboard_info");
    }

    #[test]
    fn appends_sse_tool_call_when_final_response_output_is_empty() {
        let mut collected = CollectedResponse::default();
        process_sse_line(
            r#"data: {"type":"response.output_item.done","item":{"type":"function_call","call_id":"call_1","name":"get_dashboard_info","arguments":"{}"}}"#,
            &mut collected,
            None,
        )
        .unwrap();
        process_sse_line(
            r#"data: {"type":"response.completed","response":{"output":[]}}"#,
            &mut collected,
            None,
        )
        .unwrap();

        let mut input = Vec::new();
        append_model_output(&mut input, &collected);
        input.push(json!({
            "type": "function_call_output",
            "call_id": "call_1",
            "output": "result"
        }));

        assert_eq!(input.len(), 2);
        assert_eq!(input[0]["type"], "function_call");
        assert_eq!(input[0]["call_id"], "call_1");
        assert_eq!(input[1]["type"], "function_call_output");
        assert_eq!(input[1]["call_id"], input[0]["call_id"]);
    }

    #[test]
    fn merges_final_output_with_collected_calls_without_duplicates() {
        let response_message = json!({
            "type": "message",
            "content": [{"type": "output_text", "text": "Checking."}]
        });
        let response_call = json!({
            "type": "function_call",
            "call_id": "call_1",
            "name": "get_dashboard_info",
            "arguments": "{}"
        });
        let collected = CollectedResponse {
            final_response: json!({
                "output": [response_message.clone(), response_call.clone()]
            }),
            calls: vec![
                FunctionCall {
                    call_id: "call_1".to_string(),
                    name: "get_dashboard_info".to_string(),
                    arguments: "{}".to_string(),
                },
                FunctionCall {
                    call_id: "call_2".to_string(),
                    name: "get_module_info".to_string(),
                    arguments: "{}".to_string(),
                },
            ],
            ..CollectedResponse::default()
        };

        let mut input = Vec::new();
        append_model_output(&mut input, &collected);

        assert_eq!(input.len(), 3);
        assert_eq!(input[0], response_message);
        assert_eq!(input[1], response_call);
        assert_eq!(input[2]["type"], "function_call");
        assert_eq!(input[2]["call_id"], "call_2");
    }

    #[test]
    fn response_payload_preserves_reasoning_and_supports_forced_finalization() {
        let config = AssistantConfig::default();
        let input = vec![json!({"role": "user", "content": "inspect"})];

        let tool_payload =
            build_response_payload(&config, "gpt-test", "instructions", &input, true);
        assert_eq!(tool_payload["parallel_tool_calls"], true);
        assert_eq!(tool_payload["tool_choice"], "auto");
        assert_eq!(
            tool_payload["include"],
            json!(["reasoning.encrypted_content"])
        );
        assert_eq!(tool_payload["reasoning"]["summary"], "auto");
        assert!(tool_payload["tools"]
            .as_array()
            .unwrap()
            .iter()
            .all(|tool| { tool.get("name").and_then(Value::as_str) != Some("update_function") }));

        let mut editable_config = config.clone();
        editable_config.allow_bytecode_edits = true;
        let editable_payload =
            build_response_payload(&editable_config, "gpt-test", "instructions", &input, true);
        assert!(editable_payload["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| { tool.get("name").and_then(Value::as_str) == Some("update_function") }));

        let final_payload =
            build_response_payload(&config, "gpt-test", "instructions", &input, false);
        assert_eq!(final_payload["parallel_tool_calls"], false);
        assert_eq!(final_payload["tool_choice"], "none");
        assert!(final_payload["tools"].as_array().unwrap().is_empty());
    }

    #[test]
    fn tool_loop_signatures_allow_reads_after_mutation() {
        let arguments = json!({"index": 7});
        assert_ne!(
            tool_call_signature("get_function_full_info", &arguments, 0),
            tool_call_signature("get_function_full_info", &arguments, 1)
        );
        assert_eq!(
            tool_call_signature("update_function", &arguments, 0),
            tool_call_signature("update_function", &arguments, 1)
        );
        assert_ne!(
            tool_call_signature("save_bytecode", &arguments, 0),
            tool_call_signature("save_bytecode", &arguments, 1)
        );
    }

    #[test]
    fn invalid_roles_are_rejected() {
        let messages = vec![AssistantMessage {
            role: "system".to_string(),
            content: "hidden".to_string(),
        }];
        assert!(validate_messages(&messages).is_err());
    }
}
