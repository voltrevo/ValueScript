pub fn parse_command_line(input: &str) -> Vec<String> {
  let mut args = Vec::new();
  let mut current_arg = String::new();
  let mut in_single_quote = false;
  let mut in_double_quote = false;
  let mut escape_next_char = false; // To handle escaping of characters

  for c in input.chars() {
    if escape_next_char {
      current_arg.push(c);
      escape_next_char = false;
      continue;
    }

    match c {
      ' ' if !in_single_quote && !in_double_quote => {
        if !current_arg.is_empty() {
          args.push(current_arg.clone());
          current_arg.clear();
        }
      }
      '\'' if !in_double_quote => {
        if !in_single_quote || !current_arg.ends_with('\\') {
          in_single_quote = !in_single_quote;
        } else {
          current_arg.pop(); // Remove the escape character
          current_arg.push('\''); // Add the literal quote
        }
      }
      '"' if !in_single_quote => {
        if !in_double_quote || !current_arg.ends_with('\\') {
          in_double_quote = !in_double_quote;
        } else {
          current_arg.pop(); // Remove the escape character
          current_arg.push('"'); // Add the literal quote
        }
      }
      '\\' if in_single_quote || in_double_quote => escape_next_char = true,
      _ => {
        current_arg.push(c);
      }
    }
  }

  if escape_next_char {
    // If the input ends with an unprocessed escape character, add it to the argument
    current_arg.push('\\');
  }

  if !current_arg.is_empty() {
    args.push(current_arg);
  }

  args
}
