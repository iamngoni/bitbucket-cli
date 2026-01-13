//
//  bitbucket-cli
//  cli/api.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Direct API access command
//!
//! This command allows making direct HTTP requests to the Bitbucket API,
//! similar to `gh api` for GitHub. It's useful for accessing API endpoints
//! that aren't covered by other commands or for debugging.
//!
//! ## Examples
//!
//! ```bash
//! # Get repository info
//! bb api /2.0/repositories/workspace/repo
//!
//! # Create an issue with POST
//! bb api -X POST /2.0/repositories/workspace/repo/issues \
//!     -F title="Bug report" -F content.raw="Description here"
//!
//! # Use with Server/DC
//! bb api --hostname bitbucket.company.com /rest/api/1.0/projects
//!
//! # Paginate through results
//! bb api /2.0/repositories/workspace/repo/commits --paginate
//! ```

use std::fs;
use std::time::Duration;

use anyhow::{bail, Result};
use clap::Args;
use console::style;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};
use serde_json::Value;

use crate::auth::KeyringStore;
use crate::config::Config;
use crate::context::{ContextResolver, HostType};

use super::GlobalOptions;

/// Make direct API requests
#[derive(Args, Debug)]
pub struct ApiCommand {
    /// API endpoint (e.g., /2.0/repositories/workspace/repo)
    pub endpoint: String,

    /// HTTP method (GET, POST, PUT, PATCH, DELETE)
    #[arg(long, short = 'X', default_value = "GET")]
    pub method: String,

    /// Request headers (can be specified multiple times)
    #[arg(long, short = 'H', action = clap::ArgAction::Append)]
    pub header: Vec<String>,

    /// Request body fields as JSON (key=value, can be nested with dots)
    #[arg(long, short = 'F', action = clap::ArgAction::Append)]
    pub field: Vec<String>,

    /// Raw field values (not JSON-encoded, strings only)
    #[arg(long, action = clap::ArgAction::Append)]
    pub raw_field: Vec<String>,

    /// Read request body from file (- for stdin)
    #[arg(long, short = 'f')]
    pub input: Option<String>,

    /// Paginate through all results
    #[arg(long)]
    pub paginate: bool,

    /// Override hostname (defaults to bitbucket.org for Cloud)
    #[arg(long)]
    pub hostname: Option<String>,

    /// Include response headers in output
    #[arg(long, short = 'i')]
    pub include: bool,

    /// Suppress output (only show status)
    #[arg(long)]
    pub silent: bool,

    /// Pretty-print JSON output (default: true)
    #[arg(long, default_value = "true")]
    pub pretty: bool,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,
}

impl ApiCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        // Determine host and base URL
        let (host, base_url) = self.determine_host(global)?;

        // Get authentication token
        let token = self.get_token(&host)?;

        // Parse HTTP method
        let method = self.parse_method()?;

        // Build the full URL
        let url = format!("{}{}", base_url, self.endpoint);

        // Build request body
        let body = self.build_body()?;

        // Build headers
        let headers = self.build_headers()?;

        // Create HTTP client
        let client = Client::builder()
            .timeout(Duration::from_secs(self.timeout))
            .build()?;

        if self.paginate {
            self.execute_paginated(
                &client,
                &url,
                &method,
                &headers,
                &token,
                body.clone(),
                global,
            )
            .await
        } else {
            self.execute_single(&client, &url, &method, &headers, &token, body, global)
                .await
        }
    }

    fn determine_host(&self, global: &GlobalOptions) -> Result<(String, String)> {
        if let Some(hostname) = &self.hostname {
            // Explicit hostname provided
            let base_url = if hostname == "bitbucket.org" || hostname == "api.bitbucket.org" {
                "https://api.bitbucket.org".to_string()
            } else {
                format!("https://{}", hostname)
            };
            let host = hostname.replace("api.", "");
            Ok((host, base_url))
        } else if let Some(host) = &global.host {
            // From global options
            let base_url = if host == "bitbucket.org" {
                "https://api.bitbucket.org".to_string()
            } else {
                format!("https://{}", host)
            };
            Ok((host.clone(), base_url))
        } else {
            // Try to detect from git context
            let config = Config::load().unwrap_or_default();
            let resolver = ContextResolver::new(config);

            if let Ok(ctx) = resolver.resolve(global) {
                let base_url = if ctx.host_type == HostType::Cloud {
                    "https://api.bitbucket.org".to_string()
                } else {
                    format!("https://{}", ctx.host)
                };
                Ok((ctx.host, base_url))
            } else {
                // Default to Cloud
                Ok((
                    "bitbucket.org".to_string(),
                    "https://api.bitbucket.org".to_string(),
                ))
            }
        }
    }

    fn get_token(&self, host: &str) -> Result<String> {
        let keyring = KeyringStore::new();
        keyring.get(host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated with {}. Run 'bb auth login' first.",
                host
            )
        })
    }

    fn parse_method(&self) -> Result<Method> {
        match self.method.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "PATCH" => Ok(Method::PATCH),
            "DELETE" => Ok(Method::DELETE),
            "HEAD" => Ok(Method::HEAD),
            "OPTIONS" => Ok(Method::OPTIONS),
            _ => bail!("Unsupported HTTP method: {}", self.method),
        }
    }

    fn build_body(&self) -> Result<Option<Value>> {
        // If input file is specified, read from it
        if let Some(input) = &self.input {
            let content = if input == "-" {
                let mut buffer = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer)?;
                buffer
            } else {
                fs::read_to_string(input)?
            };

            let value: Value = serde_json::from_str(&content)?;
            return Ok(Some(value));
        }

        // Build body from fields
        if self.field.is_empty() && self.raw_field.is_empty() {
            return Ok(None);
        }

        let mut body = serde_json::Map::new();

        // Process JSON-encoded fields
        for field in &self.field {
            let (key, value) = self.parse_field(field)?;
            self.set_nested_value(&mut body, &key, value);
        }

        // Process raw fields (string values)
        for field in &self.raw_field {
            let (key, value) = self.parse_raw_field(field)?;
            self.set_nested_value(&mut body, &key, Value::String(value));
        }

        Ok(Some(Value::Object(body)))
    }

    fn parse_field(&self, field: &str) -> Result<(String, Value)> {
        let parts: Vec<&str> = field.splitn(2, '=').collect();
        if parts.len() != 2 {
            bail!("Invalid field format: {}. Expected key=value", field);
        }

        let key = parts[0].to_string();
        let value_str = parts[1];

        // Try to parse as JSON
        let value = if value_str == "true" {
            Value::Bool(true)
        } else if value_str == "false" {
            Value::Bool(false)
        } else if value_str == "null" {
            Value::Null
        } else if let Ok(n) = value_str.parse::<i64>() {
            Value::Number(n.into())
        } else if let Ok(n) = value_str.parse::<f64>() {
            serde_json::Number::from_f64(n)
                .map(Value::Number)
                .unwrap_or(Value::String(value_str.to_string()))
        } else if value_str.starts_with('[') || value_str.starts_with('{') {
            serde_json::from_str(value_str).unwrap_or(Value::String(value_str.to_string()))
        } else {
            Value::String(value_str.to_string())
        };

        Ok((key, value))
    }

    fn parse_raw_field(&self, field: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = field.splitn(2, '=').collect();
        if parts.len() != 2 {
            bail!("Invalid field format: {}. Expected key=value", field);
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    #[allow(clippy::only_used_in_recursion)]
    fn set_nested_value(&self, obj: &mut serde_json::Map<String, Value>, key: &str, value: Value) {
        let parts: Vec<&str> = key.split('.').collect();

        if parts.len() == 1 {
            obj.insert(key.to_string(), value);
        } else {
            let first = parts[0];
            let rest = parts[1..].join(".");

            if !obj.contains_key(first) {
                obj.insert(first.to_string(), Value::Object(serde_json::Map::new()));
            }

            if let Some(Value::Object(nested)) = obj.get_mut(first) {
                self.set_nested_value(nested, &rest, value);
            }
        }
    }

    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        for header in &self.header {
            let parts: Vec<&str> = header.splitn(2, ':').collect();
            if parts.len() != 2 {
                bail!("Invalid header format: {}. Expected 'Name: Value'", header);
            }

            let name = HeaderName::from_bytes(parts[0].trim().as_bytes())?;
            let value = HeaderValue::from_str(parts[1].trim())?;
            headers.insert(name, value);
        }

        Ok(headers)
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_single(
        &self,
        client: &Client,
        url: &str,
        method: &Method,
        headers: &HeaderMap,
        token: &str,
        body: Option<Value>,
        global: &GlobalOptions,
    ) -> Result<()> {
        let mut request = client
            .request(method.clone(), url)
            .bearer_auth(token)
            .headers(headers.clone());

        if let Some(body) = body {
            request = request
                .header("Content-Type", "application/json")
                .json(&body);
        }

        let response = request.send().await?;
        let status = response.status();
        let response_headers = response.headers().clone();

        // Print headers if requested
        if self.include {
            println!("{} {}", style("HTTP").dim(), status);
            for (name, value) in response_headers.iter() {
                println!("{}: {}", name, value.to_str().unwrap_or(""));
            }
            println!();
        }

        // Handle response
        if self.silent {
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                bail!("Request failed with status {}: {}", status, body);
            }
            return Ok(());
        }

        let body_text = response.text().await?;

        // Try to parse as JSON and pretty-print
        if let Ok(json) = serde_json::from_str::<Value>(&body_text) {
            if global.json || self.pretty {
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("{}", json);
            }
        } else {
            // Print raw response
            println!("{}", body_text);
        }

        // Check for errors
        if !status.is_success() {
            bail!("Request failed with status {}", status);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_paginated(
        &self,
        client: &Client,
        url: &str,
        method: &Method,
        headers: &HeaderMap,
        token: &str,
        body: Option<Value>,
        global: &GlobalOptions,
    ) -> Result<()> {
        let mut all_values: Vec<Value> = Vec::new();
        let mut current_url = url.to_string();
        let mut page = 1;

        loop {
            let mut request = client
                .request(method.clone(), &current_url)
                .bearer_auth(token)
                .headers(headers.clone());

            if let Some(ref body) = body {
                request = request
                    .header("Content-Type", "application/json")
                    .json(body);
            }

            let response = request.send().await?;
            let status = response.status();

            if !status.is_success() {
                let body_text = response.text().await.unwrap_or_default();
                bail!("Request failed with status {}: {}", status, body_text);
            }

            let body_text = response.text().await?;
            let json: Value = serde_json::from_str(&body_text)?;

            // Extract values from paginated response
            if let Some(values) = json.get("values").and_then(|v| v.as_array()) {
                all_values.extend(values.clone());
            } else {
                // Not a paginated response, just return it
                if global.json || self.pretty {
                    println!("{}", serde_json::to_string_pretty(&json)?);
                } else {
                    println!("{}", json);
                }
                return Ok(());
            }

            // Check for next page
            if let Some(next) = json.get("next").and_then(|v| v.as_str()) {
                current_url = next.to_string();
                page += 1;

                // Safety limit
                if page > 100 {
                    eprintln!("{} Stopping at 100 pages", style("!").yellow());
                    break;
                }
            } else {
                break;
            }
        }

        // Output all values
        let result = serde_json::json!({
            "values": all_values,
            "size": all_values.len(),
        });

        if global.json || self.pretty {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{}", result);
        }

        Ok(())
    }
}
