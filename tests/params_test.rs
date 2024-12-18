use command_vault::{
    db::models::Parameter,
    utils::params::{parse_parameters, substitute_parameters},
};

#[test]
fn test_parse_parameters_basic() {
    let command = "echo @name";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "name");
    assert_eq!(params[0].description, None);
}

#[test]
fn test_parse_parameters_with_description() {
    let command = "echo @name:new-name";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "name");
    assert_eq!(params[0].description, Some("new-name".to_string()));
}

#[test]
fn test_parse_multiple_parameters() {
    let command = "echo @name:new-name @age:30 @city";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 3);
    
    assert_eq!(params[0].name, "name");
    assert_eq!(params[0].description, Some("new-name".to_string()));
    
    assert_eq!(params[1].name, "age");
    assert_eq!(params[1].description, Some("30".to_string()));
    
    assert_eq!(params[2].name, "city");
    assert_eq!(params[2].description, None);
}

#[test]
fn test_parse_parameters_with_underscores() {
    let command = "echo @user_name:new-user";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "user_name");
    assert_eq!(params[0].description, Some("new-user".to_string()));
}

#[test]
fn test_substitute_parameters() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message";
    let parameters = vec![
        Parameter::with_description("message".to_string(), Some("test message".to_string())),
    ];
    
    let result = substitute_parameters(command, &parameters, Some("test-value"))?;
    assert_eq!(result, "echo test-value");
    
    let result = substitute_parameters(command, &parameters, None)?;
    assert_eq!(result, "echo test_value");
    
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_no_defaults() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message @other";
    let parameters = vec![
        Parameter::with_description("message".to_string(), None),
        Parameter::with_description("other".to_string(), None),
    ];
    
    let result = substitute_parameters(command, &parameters, Some("test-value\nother-value"))?;
    assert_eq!(result, "echo test-value other-value");
    
    let result = substitute_parameters(command, &parameters, None)?;
    assert_eq!(result, "echo test_value test_value");
    
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_parse_parameters_empty_command() {
    let command = "echo";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 0);
}

#[test]
fn test_parse_parameters_invalid_names() {
    let command = "echo @123 @!invalid @valid_name";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "valid_name");
}

#[test]
fn test_substitute_parameters_with_special_chars() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "grep @pattern @file";
    let parameters = vec![
        Parameter::with_description("pattern".to_string(), Some("Search pattern".to_string())),
        Parameter::with_description("file".to_string(), Some("File to search in".to_string())),
    ];
    
    let result = substitute_parameters(command, &parameters, Some("test-pattern\n/path/to/dir"))?;
    assert_eq!(result, "grep 'test-pattern' '/path/to/dir'");
    
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_empty_value() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message";
    let parameters = vec![
        Parameter::with_description("message".to_string(), Some("A test message".to_string())),
    ];
    
    let result = substitute_parameters(command, &parameters, Some(""))?;
    assert_eq!(result, "echo ''");
    
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_spaces() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message";
    let parameters = vec![
        Parameter::with_description("message".to_string(), Some("A test message".to_string())),
    ];
    
    let result = substitute_parameters(command, &parameters, Some("hello world"))?;
    assert_eq!(result, "echo 'hello world'");
    
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_parse_parameters_from_params_rs() {
    let command = "docker run -p @port:8080 -v @volume @image";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 3);
    
    assert_eq!(params[0].name, "port");
    assert_eq!(params[0].description, Some("8080".to_string()));
    
    assert_eq!(params[1].name, "volume");
    assert_eq!(params[1].description, None);
    
    assert_eq!(params[2].name, "image");
    assert_eq!(params[2].description, None);
}