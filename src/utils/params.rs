use crate::db::models::Parameter;
use regex::Regex;
use anyhow::Result;
use std::io::Write;

pub fn parse_parameters(command: &str) -> Vec<Parameter> {
    let re = Regex::new(r"@([a-zA-Z][a-zA-Z0-9_]*)(?:=([^\s]+))?").unwrap();
    let mut parameters = Vec::new();
    
    for cap in re.captures_iter(command) {
        let name = cap[1].to_string();
        let default_value = cap.get(2).map(|m| m.as_str().to_string());
        
        parameters.push(Parameter {
            name,
            description: None,
            default_value,
        });
    }
    
    parameters
}

pub fn substitute_parameters(command: &str, parameters: &[Parameter]) -> Result<String> {
    let mut final_command = command.to_string();

    // In test environment, use default values
    let use_default = std::env::var("COMMAND_VAULT_TEST").is_ok();

    println!("\nEnter values for command parameters:");
    println!("─────────────────────────────────────────────");
    for param in parameters {
        let value = if use_default {
            param.default_value.clone().unwrap_or_default()
        } else {
            let default_str = param.default_value.as_deref().unwrap_or("");
            let desc = param.description.as_deref().unwrap_or("");
            print!("{} ({})\nPress Enter for default [{}] or enter new value: ", param.name, desc, default_str);
            std::io::stdout().flush()?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input.is_empty() {
                param.default_value.clone().unwrap_or_default()
            } else {
                input.to_string()
            }
        };

        final_command = final_command.replace(&format!("@{}", param.name), &value);
    }

    Ok(final_command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_parameters() {
        let command = "docker run -p @port=8080 -v @volume @image";
        let params = parse_parameters(command);
        
        assert_eq!(params.len(), 3);
        
        assert_eq!(params[0].name, "port");
        assert_eq!(params[0].description, None);
        assert_eq!(params[0].default_value, Some("8080".to_string()));
        
        assert_eq!(params[1].name, "volume");
        assert_eq!(params[1].description, None);
        assert_eq!(params[1].default_value, None);
        
        assert_eq!(params[2].name, "image");
        assert_eq!(params[2].description, None);
        assert_eq!(params[2].default_value, None);
    }
}
