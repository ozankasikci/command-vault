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
    
    let command = "grep @pattern /path/to/dir";
    let parameters = vec![Parameter {
        name: "pattern".to_string(),
        description: None,
    }];
    
    let result = substitute_parameters(command, &parameters, Some("test-pattern"))?;
    assert_eq!(result, "grep 'test-pattern' /path/to/dir");
    
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_empty_value() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message";
    let parameters = vec![Parameter {
        name: "message".to_string(),
        description: None,
    }];

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

#[test]
fn test_substitute_parameters_basic() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message";
    let parameters = vec![Parameter::with_description("message".to_string(), None)];
    
    let result = substitute_parameters(command, &parameters, Some("hello"))?;
    assert_eq!(result, "echo hello");
    
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_defaults() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message:default value";
    let parameters = vec![Parameter {
        name: "message".to_string(),
        description: Some("default value".to_string()),
    }];

    let result = substitute_parameters(command, &parameters, Some(""))?;
    assert_eq!(result, "echo 'default value'");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_multiple() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "git commit -m @message --author @author";
    let parameters = vec![
        Parameter {
            name: "message".to_string(),
            description: None,
        },
        Parameter {
            name: "author".to_string(),
            description: None,
        },
    ];
    
    let result = substitute_parameters(command, &parameters, Some("test commit\nJohn Doe"))?;
    assert_eq!(result, "git commit -m 'test commit' --author 'John Doe'");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_quotes() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message";
    let parameters = vec![Parameter {
        name: "message".to_string(),
        description: None,
    }];

    let result = substitute_parameters(command, &parameters, Some("hello * world"))?;
    assert_eq!(result, "echo 'hello * world'");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_empty_command() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "";
    let parameters = vec![];
    
    let result = substitute_parameters(command, &parameters, None)?;
    assert_eq!(result, "");
    
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_no_parameters() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo hello";
    let parameters = vec![];
    
    let result = substitute_parameters(command, &parameters, None)?;
    assert_eq!(result, "echo hello");
    
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_git_commands() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "git commit -m @message";
    let parameters = vec![Parameter {
        name: "message".to_string(),
        description: None,
    }];

    let result = substitute_parameters(command, &parameters, Some("test commit"))?;
    assert_eq!(result, "git commit -m 'test commit'");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_grep() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "grep @pattern";
    let parameters = vec![Parameter {
        name: "pattern".to_string(),
        description: None,
    }];

    let result = substitute_parameters(command, &parameters, Some("hello * world"))?;
    assert_eq!(result, "grep 'hello * world'");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_multiple_occurrences() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    let command = "echo @message && echo @message";
    let parameters = vec![Parameter {
        name: "message".to_string(),
        description: None,
    }];

    let result = substitute_parameters(command, &parameters, Some("test")).unwrap();
    assert_eq!(result, "echo test && echo test");
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_descriptions() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    let command = "echo @message:A test message";
    let parameters = vec![Parameter {
        name: "message".to_string(),
        description: Some("A test message".to_string()),
    }];

    let result = substitute_parameters(command, &parameters, Some("test")).unwrap();
    assert_eq!(result, "echo test");
    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_empty_value_removed_duplicate() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message";
    let parameters = vec![Parameter {
        name: "message".to_string(),
        description: None,
    }];

    let result = substitute_parameters(command, &parameters, Some(""))?;
    assert_eq!(result, "echo ''");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_parse_parameters_with_adjacent_parameters() {
    let command = "echo @first@second";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name, "first");
    assert_eq!(params[1].name, "second");
}

#[test]
fn test_parse_parameters_with_special_chars_in_description() {
    let command = "echo @name:special!";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "name");
    assert_eq!(params[0].description, Some("special!".to_string()));
}

#[test]
fn test_substitute_parameters_with_semicolon() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @cmd";
    let parameters = vec![Parameter {
        name: "cmd".to_string(),
        description: None,
    }];

    let result = substitute_parameters(command, &parameters, Some("echo hello; ls"))?;
    assert_eq!(result, "echo 'echo hello; ls'");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_pipe() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @cmd";
    let parameters = vec![Parameter {
        name: "cmd".to_string(),
        description: None,
    }];

    let result = substitute_parameters(command, &parameters, Some("ls | grep test"))?;
    assert_eq!(result, "echo 'ls | grep test'");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_redirection() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @cmd";
    let parameters = vec![Parameter {
        name: "cmd".to_string(),
        description: None,
    }];

    let result = substitute_parameters(command, &parameters, Some("echo test > file.txt"))?;
    assert_eq!(result, "echo 'echo test > file.txt'");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_existing_quotes() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message";
    let parameters = vec![Parameter {
        name: "message".to_string(),
        description: None,
    }];

    let result = substitute_parameters(command, &parameters, Some("'already quoted'"))?;
    assert_eq!(result, "echo 'already quoted'");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_substitute_parameters_with_escaped_quotes() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let command = "echo @message";
    let parameters = vec![Parameter {
        name: "message".to_string(),
        description: None,
    }];

    let result = substitute_parameters(command, &parameters, Some("It's a test"))?;
    assert_eq!(result, "echo 'It'\\''s a test'");

    std::env::remove_var("COMMAND_VAULT_TEST");
    Ok(())
}

#[test]
fn test_parameter_new() {
    let param = Parameter::new("test".to_string());
    assert_eq!(param.name, "test");
    assert_eq!(param.description, None);
}

#[test]
fn test_parameter_with_description() {
    let param = Parameter::with_description("test".to_string(), Some("A test parameter".to_string()));
    assert_eq!(param.name, "test");
    assert_eq!(param.description, Some("A test parameter".to_string()));
}

#[test]
fn test_parse_parameters_with_trailing_whitespace() {
    let command = "echo @name:test";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "name");
    assert_eq!(params[0].description, Some("test".to_string()));
}

#[test]
fn test_parse_parameters_with_multiple_colons() {
    let command = "echo @name:test:value";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "name");
    assert_eq!(params[0].description, Some("test:value".to_string()));
}

#[test]
fn test_parse_parameters_with_numbers_in_description() {
    let command = "echo @port:8080 @host:localhost:8080";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name, "port");
    assert_eq!(params[0].description, Some("8080".to_string()));
    assert_eq!(params[1].name, "host");
    assert_eq!(params[1].description, Some("localhost:8080".to_string()));
}

#[test]
fn test_parse_parameters_with_dash_in_description() {
    let command = "git checkout @branch:feature-123";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "branch");
    assert_eq!(params[0].description, Some("feature-123".to_string()));
}