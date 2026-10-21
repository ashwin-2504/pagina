use ddl_engine::parser::Parser;
use std::io::{self, Read};

fn main() {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    // If stdin has input, read it; otherwise run a sample
    if let Ok(_) = handle.read_to_string(&mut buffer) {
        if !buffer.trim().is_empty() {
            run_parse(&buffer);
            return;
        }
    }

    println!("DDL Engine CLI v0.1 ready. Running sample document parse...");
    let sample = r#"<doc>
      <props>
        <meta><title>CLI Demo</title></meta>
        <paper size="A4" orientation="portrait" margin="20mm" />
      </props>
      <body>
        <section page-style="default">
          <h1>Hello from DDL Engine</h1>
          <p>Typesetting engine running natively.</p>
        </section>
      </body>
    </doc>"#;
    run_parse(sample);
}

fn run_parse(input: &str) {
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(doc) => {
            let json = serde_json::to_string_pretty(&doc).unwrap();
            println!("{}", json);
        }
        Err(errs) => {
            eprintln!("Parse errors: {:?}", errs);
        }
    }
}
