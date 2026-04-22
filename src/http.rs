use crate::exports::wasco_dev::heyreach_api::heyreach_api::{ApiError, ApiErrorCode};
use crate::wasi::http::outgoing_handler;
use crate::wasi::http::types::*;
use crate::wasi::io::streams::StreamError;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub enum HttpMethod {
    Get,
    Post,
    Delete,
}

pub fn make_request<T: DeserializeOwned>(
    method: HttpMethod,
    path: &str,
    api_key: &str,
    body: Option<&impl Serialize>,
) -> Result<T, ApiError> {
    println!("[DEBUG] make_request called - path: {}", path);

    let headers = Fields::new();

    println!("[DEBUG] Creating headers...");
    headers
        .append(
            &"content-type".to_string(),
            &b"application/json; charset=utf-8".to_vec(),
        )
        .map_err(|e| {
            println!("[ERROR] Failed to append content-type header: {:?}", e);
            api_error(
                ApiErrorCode::Unknown,
                "failed to append content-type header",
            )
        })?;

    headers
        .append(&"x-api-key".to_string(), api_key.as_bytes())
        .map_err(|e| {
            println!("[ERROR] Failed to set API key header: {:?}", e);
            api_error(ApiErrorCode::Unauthorized, "Failed to set API key header")
        })?;

    let outgoing_request = OutgoingRequest::new(headers);

    let method_value = match method {
        HttpMethod::Get => Method::Get,
        HttpMethod::Post => Method::Post,
        HttpMethod::Delete => Method::Delete,
    };

    println!("[DEBUG] Setting method: {:?}", method_value);
    outgoing_request.set_method(&method_value).map_err(|e| {
        println!("[ERROR] Failed to set method: {:?}", e);
        api_error(ApiErrorCode::Unknown, "Failed to set method")
    })?;

    println!("[DEBUG] Setting path: {}", path);
    outgoing_request
        .set_path_with_query(Some(path))
        .map_err(|e| {
            println!("[ERROR] Failed to set path: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to set path")
        })?;

    println!("[DEBUG] Setting scheme to HTTPS");
    outgoing_request
        .set_scheme(Some(&Scheme::Https))
        .map_err(|e| {
            println!("[ERROR] Failed to set scheme: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to set scheme")
        })?;

    println!("[DEBUG] Setting authority to api.heyreach.io");
    outgoing_request
        .set_authority(Some("api.heyreach.io"))
        .map_err(|e| {
            println!("[ERROR] Failed to set authority: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to set authority")
        })?;

    if let Some(body_data) = body {
        println!("[DEBUG] Serializing body...");
        let body_bytes = serde_json::to_vec(body_data).map_err(|e| {
            println!("[ERROR] Failed to serialize body: {}", e);
            api_error(
                ApiErrorCode::BadRequest,
                &format!("Failed to serialize body: {}", e),
            )
        })?;

        println!("[DEBUG] Body size: {} bytes", body_bytes.len());

        let outgoing_body = outgoing_request.body().map_err(|e| {
            println!("[ERROR] Failed to get outgoing body: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to get outgoing body")
        })?;

        let body_stream = outgoing_body.write().map_err(|e| {
            println!("[ERROR] Failed to get body stream: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to get body stream")
        })?;

        println!("[DEBUG] Writing body...");
        body_stream
            .blocking_write_and_flush(&body_bytes)
            .map_err(|e| {
                println!("[ERROR] Failed to write body: {:?}", e);
                api_error(ApiErrorCode::Unknown, "Failed to write body")
            })?;

        drop(body_stream);
        OutgoingBody::finish(outgoing_body, None).map_err(|e| {
            println!("[ERROR] Failed to finish body: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to finish body")
        })?;
    } else {
        println!("[DEBUG] No body to send");
    }

    println!("[DEBUG] Sending request...");
    let future_response = outgoing_handler::handle(outgoing_request, None).map_err(|e| {
        println!("[ERROR] Failed to send request: {:?}", e);
        api_error(ApiErrorCode::Unknown, "Failed to send request")
    })?;

    println!("[DEBUG] Waiting for response...");
    future_response.subscribe().block();

    println!("[DEBUG] Getting response...");
    let incoming_response = future_response
        .get()
        .ok_or_else(|| {
            println!("[ERROR] Request not completed");
            api_error(ApiErrorCode::Unknown, "Request not completed")
        })?
        .map_err(|e| {
            println!("[ERROR] Request failed: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Request failed")
        })?
        .map_err(|e| {
            println!("[ERROR] Request error: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Request error")
        })?;

    let status = incoming_response.status();
    println!("[DEBUG] Response status: {}", status);

    println!("[DEBUG] Getting response body...");
    let incoming_body = incoming_response.consume().map_err(|e| {
        println!("[ERROR] Failed to get response body: {:?}", e);
        api_error(ApiErrorCode::Unknown, "Failed to get response body")
    })?;

    println!("[DEBUG] Getting body stream...");
    let body_stream = incoming_body.stream().map_err(|e| {
        println!("[ERROR] Failed to get body stream: {:?}", e);
        api_error(ApiErrorCode::Unknown, "Failed to get body stream")
    })?;

    let mut response_bytes = Vec::new();
    println!("[DEBUG] Reading response chunks...");
    loop {
        match body_stream.blocking_read(8192) {
            Ok(chunk) => {
                if chunk.is_empty() {
                    println!("[DEBUG] Finished reading response");
                    break;
                }
                println!("[DEBUG] Read chunk of {} bytes", chunk.len());
                response_bytes.extend_from_slice(&chunk);
            }
            Err(StreamError::Closed) => {
                println!("[DEBUG] Stream closed (end of response)");
                break;
            }
            Err(e) => {
                println!("[ERROR] Failed to read response chunk: {:?}", e);
                return Err(api_error(ApiErrorCode::Unknown, "Failed to read response"));
            }
        }
    }

    println!("[DEBUG] Total response bytes: {}", response_bytes.len());
    drop(body_stream);

    if status >= 400 {
        println!("[DEBUG] Error status code detected: {}", status);
        let error_code = match status {
            401 => ApiErrorCode::Unauthorized,
            404 => ApiErrorCode::NotFound,
            429 => ApiErrorCode::TooManyRequests,
            400 => ApiErrorCode::BadRequest,
            422 => ApiErrorCode::Validation,
            _ => ApiErrorCode::Unknown,
        };

        let error_message = if let Ok(text) = String::from_utf8(response_bytes.clone()) {
            println!("[DEBUG] Error response body: {}", text);
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&text) {
                error_json
                    .get("detail")
                    .or_else(|| error_json.get("errorMessage"))
                    .or_else(|| error_json.get("message"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("HTTP {}", status))
            } else {
                text
            }
        } else {
            println!("[DEBUG] Error response body is not valid UTF-8");
            format!("HTTP {}", status)
        };

        return Err(api_error(error_code, &error_message));
    }

    println!("[DEBUG] Converting response to UTF-8...");
    let response_text = String::from_utf8(response_bytes).map_err(|e| {
        println!("[ERROR] Invalid UTF-8 in response: {:?}", e);
        api_error(ApiErrorCode::Unknown, "Invalid UTF-8 in response")
    })?;

    println!("[DEBUG] Response text: '{}'", response_text);
    println!("[DEBUG] Parsing JSON...");

    serde_json::from_str(&response_text).map_err(|e| {
        println!("[ERROR] Failed to parse response JSON: {}", e);
        println!("[ERROR] Response was: '{}'", response_text);
        api_error(
            ApiErrorCode::Unknown,
            &format!("Failed to parse response: {}", e),
        )
    })
}

pub fn make_request_empty(
    method: HttpMethod,
    path: &str,
    api_key: &str,
    body: Option<&impl Serialize>,
) -> Result<(), ApiError> {
    println!("[DEBUG] make_request_empty called - path: {}", path);

    let headers = Fields::new();

    println!("[DEBUG] Creating headers...");
    headers
        .append(
            &"content-type".to_string(),
            &b"application/json; charset=utf-8".to_vec(),
        )
        .map_err(|e| {
            println!("[ERROR] Failed to append content-type header: {:?}", e);
            api_error(
                ApiErrorCode::Unknown,
                "failed to append content-type header",
            )
        })?;

    headers
        .append(&"x-api-key".to_string(), api_key.as_bytes())
        .map_err(|e| {
            println!("[ERROR] Failed to set API key header: {:?}", e);
            api_error(ApiErrorCode::Unauthorized, "Failed to set API key header")
        })?;

    let outgoing_request = OutgoingRequest::new(headers);

    let method_value = match method {
        HttpMethod::Get => Method::Get,
        HttpMethod::Post => Method::Post,
        HttpMethod::Delete => Method::Delete,
    };

    println!("[DEBUG] Setting method: {:?}", method_value);
    outgoing_request.set_method(&method_value).map_err(|e| {
        println!("[ERROR] Failed to set method: {:?}", e);
        api_error(ApiErrorCode::Unknown, "Failed to set method")
    })?;

    println!("[DEBUG] Setting path: {}", path);
    outgoing_request
        .set_path_with_query(Some(path))
        .map_err(|e| {
            println!("[ERROR] Failed to set path: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to set path")
        })?;

    println!("[DEBUG] Setting scheme to HTTPS");
    outgoing_request
        .set_scheme(Some(&Scheme::Https))
        .map_err(|e| {
            println!("[ERROR] Failed to set scheme: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to set scheme")
        })?;

    println!("[DEBUG] Setting authority to api.heyreach.io");
    outgoing_request
        .set_authority(Some("api.heyreach.io"))
        .map_err(|e| {
            println!("[ERROR] Failed to set authority: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to set authority")
        })?;

    if let Some(body_data) = body {
        println!("[DEBUG] Serializing body...");
        let body_bytes = serde_json::to_vec(body_data).map_err(|e| {
            println!("[ERROR] Failed to serialize body: {}", e);
            api_error(
                ApiErrorCode::BadRequest,
                &format!("Failed to serialize body: {}", e),
            )
        })?;

        println!("[DEBUG] Body size: {} bytes", body_bytes.len());

        let outgoing_body = outgoing_request.body().map_err(|e| {
            println!("[ERROR] Failed to get outgoing body: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to get outgoing body")
        })?;

        let body_stream = outgoing_body.write().map_err(|e| {
            println!("[ERROR] Failed to get body stream: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to get body stream")
        })?;

        println!("[DEBUG] Writing body...");
        body_stream
            .blocking_write_and_flush(&body_bytes)
            .map_err(|e| {
                println!("[ERROR] Failed to write body: {:?}", e);
                api_error(ApiErrorCode::Unknown, "Failed to write body")
            })?;

        drop(body_stream);
        OutgoingBody::finish(outgoing_body, None).map_err(|e| {
            println!("[ERROR] Failed to finish body: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to finish body")
        })?;
    } else {
        println!("[DEBUG] No body to send");
    }

    println!("[DEBUG] Sending request...");
    let future_response = outgoing_handler::handle(outgoing_request, None).map_err(|e| {
        println!("[ERROR] Failed to send request: {:?}", e);
        api_error(ApiErrorCode::Unknown, "Failed to send request")
    })?;

    println!("[DEBUG] Waiting for response...");
    future_response.subscribe().block();

    println!("[DEBUG] Getting response...");
    let incoming_response = future_response
        .get()
        .ok_or_else(|| {
            println!("[ERROR] Request not completed");
            api_error(ApiErrorCode::Unknown, "Request not completed")
        })?
        .map_err(|e| {
            println!("[ERROR] Request failed: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Request failed")
        })?
        .map_err(|e| {
            println!("[ERROR] Request error: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Request error")
        })?;

    let status = incoming_response.status();
    println!("[DEBUG] Response status: {}", status);

    // Check status first before trying to read body
    if status >= 400 {
        println!("[DEBUG] Error status code detected: {}", status);

        // Try to read error body
        let incoming_body = incoming_response.consume().map_err(|e| {
            println!("[ERROR] Failed to get response body: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to get response body")
        })?;

        let body_stream = incoming_body.stream().map_err(|e| {
            println!("[ERROR] Failed to get body stream: {:?}", e);
            api_error(ApiErrorCode::Unknown, "Failed to get body stream")
        })?;

        let mut response_bytes = Vec::new();
        loop {
            match body_stream.blocking_read(8192) {
                Ok(chunk) if chunk.is_empty() => break,
                Ok(chunk) => response_bytes.extend_from_slice(&chunk),
                Err(_) => break, // Stream closed or error, just use what we have
            }
        }

        drop(body_stream);

        let error_code = match status {
            401 => ApiErrorCode::Unauthorized,
            404 => ApiErrorCode::NotFound,
            429 => ApiErrorCode::TooManyRequests,
            400 => ApiErrorCode::BadRequest,
            422 => ApiErrorCode::Validation,
            _ => ApiErrorCode::Unknown,
        };

        let error_message = if let Ok(text) = String::from_utf8(response_bytes.clone()) {
            println!("[DEBUG] Error response body: {}", text);
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&text) {
                error_json
                    .get("detail")
                    .or_else(|| error_json.get("errorMessage"))
                    .or_else(|| error_json.get("message"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("HTTP {}", status))
            } else {
                text
            }
        } else {
            println!("[DEBUG] Error response body is not valid UTF-8");
            format!("HTTP {}", status)
        };

        return Err(api_error(error_code, &error_message));
    }

    // For successful responses (status < 400), we don't need to read the body
    // Just consume it to clean up resources, but ignore any errors
    println!("[DEBUG] Success status, consuming body (but ignoring content)...");
    let _ = incoming_response.consume();

    println!("[DEBUG] Success! Returning Ok(())");
    Ok(())
}

fn api_error(code: ApiErrorCode, message: &str) -> ApiError {
    ApiError {
        code,
        message: message.to_string(),
    }
}
