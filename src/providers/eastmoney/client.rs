pub fn http_get_json(url: &str, source_label: &str) -> Result<serde_json::Value, String> {
    match ureq::get(url).set("Accept", "application/json").call() {
        Ok(response) => {
            let body = response.into_string().map_err(|error| error.to_string())?;
            serde_json::from_str(&body)
                .map_err(|error| format!("{source_label} parse failed: {error}"))
        }
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().unwrap_or_default();
            if body.is_empty() {
                Err(format!("{source_label} HTTP {status}"))
            } else {
                Err(format!("{source_label} HTTP {status}: {body}"))
            }
        }
        Err(ureq::Error::Transport(error)) => Err(error.to_string()),
    }
}
