use neospeller::language::Language;
use serde_json::json;
use std::env;

#[test]
fn test_complete_spellcheck_workflow() {
    // Start mock server
    let mut server = mockito::Server::new();

    // Create mock API response that simulates OpenAI's response
    let mock_response = json!({
        "choices": [{
            "message": {
                "role": "assistant",
                "content": json!({
                    "single_comments": {
                        "16": "Read input from standard input (stdin)",
                        "19": "Process text for send to an API",
                        "22": "Print the modified text to standard output (stdout)"
                    },
                    "multiline_comments": {
                            "2": "Demo script for reading and writing text using standard input and output.",
                            "3": "This script simulates processing text and displaying a modified result.",
                            "10": "Simulates text processing by reversing the input.",
                            "11": "This is just a placeholder for demonstration purposes."
                        }
                }).to_string()
            },
        "index": 0
        }],
    });

    // Set mock server URL as environment variable
    env::set_var("OPENAI_API_KEY", "test_key");
    env::set_var("OPENAI_API_BASE_URL", server.url());

    // Setup mock endpoint
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    // Input Python code with intentional spelling errors
    let input = r#"
"""
Demo scrippt for reading and writting text using standart input and output.
This scrippt simulates proccessing text and displaying a modifed result.
"""

import sys

def proccess_text(text: str) -> str:
    """
    Simuulates text proccessing by reversing the input.
    This is just a placehoulder for demostration purposes.
    """
    return text[::-1]

def main():
    # Read input from standart input (stdin)
    input_text = sys.stdin.read().strip()
    
    # Process text for send to an API
    output_text = proccess_text(input_text)
    
    # Print the modifed text to standart output (stdout)
    print(output_text)

if __name__ == "__main__":
    main()
"#;

    // Expected output after spell checking
    let expected_output = r#"
"""
Demo script for reading and writing text using standard input and output.
This script simulates processing text and displaying a modified result.
"""

import sys

def proccess_text(text: str) -> str:
    """
    Simulates text processing by reversing the input.
    This is just a placeholder for demonstration purposes.
    """
    return text[::-1]

def main():
    # Read input from standard input (stdin)
    input_text = sys.stdin.read().strip()
    
    # Process text for send to an API
    output_text = proccess_text(input_text)
    
    # Print the modified text to standard output (stdout)
    print(output_text)

if __name__ == "__main__":
    main()
"#;

    let language = Language {
        name: "python".to_string(),
        comment_symbol: "#".to_string(),
        ml_comment_symbol: "\"\"\"".to_string(),
        ml_comment_symbol_close: "\"\"\"".to_string(),
    };

    // Run the spell checker through the main entry point
    let result = neospeller::check_spelling(input.to_string(), language).unwrap();

    // Verify the mock was called
    mock.assert();

    // Verify the result matches expected output
    assert_eq!(result.trim(), expected_output.trim());
}
