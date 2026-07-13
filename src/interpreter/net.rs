use crate::engine::value::Value;
use std::thread;
use std::time::Duration;

const MAX_ATTEMPTS: u32 = 3;

fn get_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .user_agent("NodeStract-Agent/1.0")
        .build()
}

fn perform_request<F>(method_name: &str, url: &str, request_fn: F) -> Result<Value, String> 
where F: Fn(&ureq::Agent) -> Result<ureq::Response, ureq::Error> 
{
    let agent = get_agent();
    let mut attempt = 0;

    loop {
        attempt += 1;
        match request_fn(&agent) {
            Ok(resp) => {
                return match resp.into_string() {
                    Ok(text) => Ok(Value::String(text)),
                    Err(e) => {
                        Err(format!("NET ERROR [{}]: Body read failed. {}", method_name, e))
                    }
                };
            },
            Err(e) => {
                // Determine if we should retry (server errors or transport errors)
                let is_server_error = match &e {
                    ureq::Error::Status(code, _) => *code >= 500 || *code == 429,
                    ureq::Error::Transport(_) => true,
                };

                if is_server_error && attempt < MAX_ATTEMPTS {
                    let wait = Duration::from_millis(500 * 2_u64.pow(attempt - 1));
                    thread::sleep(wait);
                    continue;
                }
                return Err(format!("NET ERROR: {} {} failed after {} attempts. {}", method_name, url, MAX_ATTEMPTS, e));
            }
        }
    }
}

pub fn get(url: &str) -> Result<Value, String> {
    perform_request("GET", url, |agent| agent.get(url).call())
}

pub fn post(url: &str, body: &str) -> Result<Value, String> {
    let json_val: serde_json::Value = serde_json::from_str(body)
        .unwrap_or(serde_json::Value::Null);

    perform_request("POST", url, |agent| {
        agent.post(url)
            .send_json(&json_val)
    })
}