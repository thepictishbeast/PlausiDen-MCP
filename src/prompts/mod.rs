//! MCP prompts — workflow templates for common use cases.

use serde_json::Value;
use std::collections::HashMap;

/// List all available prompts.
pub fn list_all_prompts() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "quick_start",
            "description": "Set up basic browsing pollution in under 60 seconds",
            "arguments": []
        }),
        serde_json::json!({
            "name": "journalist_protection",
            "description": "Configure protection for a journalist threat model",
            "arguments": [
                { "name": "country", "description": "Country context for locale-appropriate generation", "required": false }
            ]
        }),
        serde_json::json!({
            "name": "maximum_protection",
            "description": "Maximum protection — all categories, all channels, swarm enabled",
            "arguments": []
        }),
    ]
}

/// Get a prompt by name, returning MCP prompt messages.
pub fn get_prompt(name: &str, arguments: &HashMap<String, String>) -> Result<Vec<Value>, String> {
    match name {
        "quick_start" => Ok(vec![serde_json::json!({
            "role": "user",
            "content": {
                "type": "text",
                "text": concat!(
                    "Set up basic plausible deniability protection:\n",
                    "1. Enable the 'generate' capability\n",
                    "2. Generate 50 browser history entries with the 'casual' profile\n",
                    "3. Generate 30 matching cookies\n",
                    "4. Generate 20 search queries with topic drift\n",
                    "5. Show me the status when done\n\n",
                    "This creates synthetic browsing artifacts in memory. ",
                    "To actually inject them into your browser databases, you'll need ",
                    "to enable the 'inject' capability separately."
                )
            }
        })]),

        "journalist_protection" => {
            let country = arguments
                .get("country")
                .map(|c| c.as_str())
                .unwrap_or("unspecified");

            Ok(vec![serde_json::json!({
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        concat!(
                            "Configure journalist threat model protection (country: {}):\n",
                            "1. Enable 'generate' and 'query' capabilities\n",
                            "2. Create a 'journalist' profile\n",
                            "3. Generate diverse browsing history (news sites, press freedom orgs, public records)\n",
                            "4. Generate network traffic patterns consistent with research activity\n",
                            "5. Generate contact list with plausible professional contacts\n",
                            "6. Show forensic weight analysis — which categories matter most\n\n",
                            "Do NOT enable inject or swarm unless I explicitly ask."
                        ),
                        country
                    )
                }
            })])
        }

        "maximum_protection" => Ok(vec![serde_json::json!({
            "role": "user",
            "content": {
                "type": "text",
                "text": concat!(
                    "Enable maximum plausible deniability protection:\n",
                    "1. Enable ALL safe capabilities (generate, query, profile_management, schedule)\n",
                    "2. Create an 'activist' profile with maximum intensity\n",
                    "3. Generate artifacts in ALL categories: browser, cookies, searches, files, contacts, location, network\n",
                    "4. Show the forensic weight analysis\n\n",
                    "WARNING: This prompt does NOT enable inject or swarm. Those require ",
                    "separate explicit acknowledgment because they modify real data and open network connections."
                )
            }
        })]),

        _ => Err(format!("Unknown prompt: {name}")),
    }
}
