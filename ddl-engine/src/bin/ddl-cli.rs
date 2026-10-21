use ddl_engine::docx::{export_docx, import_docx};
use ddl_engine::parser::Parser;
use ddl_engine::pdf::{export_pdf, import_pdf};
use std::env;
use std::fs;
use std::path::Path;

fn print_usage() {
    println!("Pagina Document Engine CLI v0.1.0");
    println!("Usage:");
    println!("  ddl-cli parse <input.dxml>");
    println!("  ddl-cli export --format <pdf|docx> <input.dxml> -o <output>");
    println!("  ddl-cli import <input.docx|input.pdf> -o <output_dir>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "parse" => {
            if args.len() < 3 {
                eprintln!("Error: Missing input file path. Usage: ddl-cli parse <input.dxml>");
                return;
            }
            let path = &args[2];
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read file {}: {}", path, e);
                    return;
                }
            };
            let mut parser = Parser::new(&content);
            match parser.parse() {
                Ok(doc) => {
                    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
                }
                Err(errs) => {
                    eprintln!("Syntax errors: {:?}", errs);
                }
            }
        }
        "export" => {
            let mut format = "pdf";
            let mut input_path = "";
            let mut output_path = "";

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--format" | "-f" => {
                        if i + 1 < args.len() {
                            format = &args[i + 1];
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "-o" | "--output" => {
                        if i + 1 < args.len() {
                            output_path = &args[i + 1];
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    val => {
                        if input_path.is_empty() && !val.starts_with('-') {
                            input_path = val;
                        }
                        i += 1;
                    }
                }
            }

            if input_path.is_empty() {
                eprintln!("Error: Missing input .dxml file.");
                return;
            }

            let content = match fs::read_to_string(input_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input_path, e);
                    return;
                }
            };

            let mut parser = Parser::new(&content);
            let doc = match parser.parse() {
                Ok(d) => d,
                Err(errs) => {
                    eprintln!("Syntax errors parsing {}: {:?}", input_path, errs);
                    return;
                }
            };

            let default_out = if format == "docx" { "output.docx" } else { "output.pdf" };
            let final_out = if output_path.is_empty() { default_out } else { output_path };

            match format.to_lowercase().as_str() {
                "docx" | "openxml" => {
                    match export_docx(&doc) {
                        Ok(bytes) => {
                            fs::write(final_out, bytes).unwrap();
                            println!("Exported OpenXML document to: {}", final_out);
                        }
                        Err(e) => eprintln!("Failed to export docx: {}", e),
                    }
                }
                "pdf" => {
                    match export_pdf(&doc) {
                        Ok(bytes) => {
                            fs::write(final_out, bytes).unwrap();
                            println!("Exported PDF document to: {}", final_out);
                        }
                        Err(e) => eprintln!("Failed to export pdf: {}", e),
                    }
                }
                other => {
                    eprintln!("Unknown format '{}'. Supported: pdf, docx", other);
                }
            }
        }
        "import" => {
            if args.len() < 3 {
                eprintln!("Error: Missing input file to import. Usage: ddl-cli import <input.docx|input.pdf> -o <output_dir>");
                return;
            }
            let input_path = &args[2];
            let out_dir = if args.len() >= 5 && args[3] == "-o" {
                &args[4]
            } else {
                "."
            };

            let bytes = match fs::read(input_path) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input_path, e);
                    return;
                }
            };

            let is_docx = input_path.ends_with(".docx");
            let is_pdf = input_path.ends_with(".pdf");

            let res = if is_docx {
                import_docx(&bytes)
            } else if is_pdf {
                import_pdf(&bytes)
            } else {
                eprintln!("Unsupported file type. Please provide a .docx or .pdf file.");
                return;
            };

            match res {
                Ok(result) => {
                    fs::create_dir_all(out_dir).unwrap();
                    let dxml_path = Path::new(out_dir).join("imported.dxml");
                    let dcss_path = Path::new(out_dir).join("styles.dcss");

                    fs::write(&dxml_path, &result.dxml).unwrap();
                    fs::write(&dcss_path, &result.dcss).unwrap();

                    println!("Successfully reverse-engineered into:");
                    println!("  DXML: {}", dxml_path.display());
                    println!("  DCSS: {}", dcss_path.display());
                }
                Err(e) => {
                    eprintln!("Failed to reverse-engineer document: {}", e);
                }
            }
        }
        _ => {
            print_usage();
        }
    }
}
