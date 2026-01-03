use anyhow::{anyhow, Result};

/// Parses a command string into the command name, structured arguments, and the raw argument string.
///
/// Returns a tuple of:
/// - `cmd`: The command name (e.g., "/help")
/// - `args`: A vector of parsed arguments, supporting quoted strings and escapes
/// - `raw_args`: The raw string following the command name, trimmed
pub fn parse_command(input: &str) -> Result<(String, Vec<String>, String)> {
    let input = input.trim();
    if input.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    // Find where the command ends (first whitespace)
    let cmd_end = input.find(char::is_whitespace).unwrap_or(input.len());

    let cmd = input[..cmd_end].to_string();
    let raw_args = input[cmd_end..].trim().to_string();

    let args = parse_args(&raw_args)?;

    Ok((cmd, args, raw_args))
}

fn parse_args(input: &str) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escape = false;

    let chars = input.chars();

    for c in chars {
        if in_quote {
            if escape {
                if c == '"' || c == '\\' {
                    current.push(c);
                } else {
                    current.push('\\');
                    current.push(c);
                }
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_quote = false;
                args.push(current.clone());
                current.clear();
            } else {
                current.push(c);
            }
        } else {
            if c == '"' {
                in_quote = true;
            } else if c.is_whitespace() {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            } else {
                current.push(c);
            }
        }
    }

    if in_quote {
        return Err(anyhow!("Unterminated quote"));
    }

    if !current.is_empty() {
        args.push(current);
    }

    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_simple() {
        let (cmd, args, raw) = parse_command("/cmd arg1 arg2").unwrap();
        assert_eq!(cmd, "/cmd");
        assert_eq!(args, vec!["arg1", "arg2"]);
        assert_eq!(raw, "arg1 arg2");
    }

    #[test]
    fn test_parse_command_quoted() {
        let (cmd, args, raw) = parse_command("/cmd \"arg 1\" arg2").unwrap();
        assert_eq!(cmd, "/cmd");
        assert_eq!(args, vec!["arg 1", "arg2"]);
        assert_eq!(raw, "\"arg 1\" arg2");
    }

    #[test]
    fn test_parse_command_escaped_quote() {
        let (cmd, args, raw) = parse_command("/cmd \"arg \\\"1\\\"\"").unwrap();
        assert_eq!(cmd, "/cmd");
        assert_eq!(args, vec!["arg \"1\""]);
        assert_eq!(raw, "\"arg \\\"1\\\"\"");
    }

    #[test]
    fn test_parse_command_unterminated() {
        let res = parse_command("/cmd \"arg 1");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_command_no_args() {
        let (cmd, args, raw) = parse_command("/cmd").unwrap();
        assert_eq!(cmd, "/cmd");
        assert!(args.is_empty());
        assert_eq!(raw, "");
    }
}
