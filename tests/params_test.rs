use command_vault::utils::params::{parse_parameters, substitute_parameters};
use command_vault::db::models::Parameter;
use anyhow::Result;

#[test]
fn test_parse_parameters_basic() {
    let command = "echo @name";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "name");
    assert_eq!(params[0].default_value, None);
}

#[test]
fn test_parse_parameters_with_default() {
    let command = "echo @name=John";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "name");
    assert_eq!(params[0].default_value, Some("John".to_string()));
}

#[test]
fn test_parse_multiple_parameters() {
    let command = "echo @name=John @age=30 @city";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 3);
    
    assert_eq!(params[0].name, "name");
    assert_eq!(params[0].default_value, Some("John".to_string()));
    
    assert_eq!(params[1].name, "age");
    assert_eq!(params[1].default_value, Some("30".to_string()));
    
    assert_eq!(params[2].name, "city");
    assert_eq!(params[2].default_value, None);
}

#[test]
fn test_parse_parameters_with_underscores() {
    let command = "echo @user_name=John_Doe";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "user_name");
    assert_eq!(params[0].default_value, Some("John_Doe".to_string()));
}

#[test]
fn test_substitute_parameters() -> Result<()> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @name @age";
    let parameters = vec![
        Parameter {
            name: "name".to_string(),
            description: None,
            default_value: Some("John".to_string()),
        },
        Parameter {
            name: "age".to_string(),
            description: None,
            default_value: Some("30".to_string()),
        },
    ];
    
    let result = substitute_parameters(command, &parameters)?;
    assert_eq!(result, "echo John 30");
    
    Ok(())
}

#[test]
fn test_substitute_parameters_no_defaults() -> Result<()> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @name @age";
    let parameters = vec![
        Parameter {
            name: "name".to_string(),
            description: None,
            default_value: None,
        },
        Parameter {
            name: "age".to_string(),
            description: None,
            default_value: None,
        },
    ];
    
    let result = substitute_parameters(command, &parameters)?;
    assert_eq!(result, "echo  ");
    
    Ok(())
}

#[test]
fn test_parse_parameters_invalid_names() {
    let command = "echo @1name @!invalid @valid_name";
    let params = parse_parameters(command);
    
    // Only valid_name should be parsed
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "valid_name");
}

#[test]
fn test_parse_parameters_empty_command() {
    let command = "";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 0);
}

#[test]
fn test_parse_parameters_from_params_rs() {
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