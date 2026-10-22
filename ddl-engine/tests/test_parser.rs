use ddl_engine::{parser, pdf};

#[test]
fn test_callout_spans() {
    let input = r#"<doc><props></props><body><section><div class="callout"><b>Separation of Concerns:</b> Styles are linked via <code>&lt;link rel="stylesheet" href="styles.css" /&gt;</code>, allowing designers or AI agents to style documents independently.</div></section></body></doc>"#;
    let mut p = parser::Parser::new(input);
    let doc = p.parse().unwrap();
    let bytes = pdf::export_pdf(&doc).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_basic_parse() {
    let input = r#"
    <doc>
      <props>
        <meta>
          <title>Test Doc</title>
        </meta>
        <paper size="A4" orientation="portrait" margin="20mm" />
        <style>
          h1 { color: #1e3a8a; }
          .callout { background: #f0fdf4; }
        </style>
      </props>
      <body>
        <section page-style="cover">
          <h1>Title</h1>
          <p>Hello World</p>
        </section>
      </body>
    </doc>
    "#;

    let mut parser = parser::Parser::new(input);
    let doc = parser.parse().expect("Failed to parse");
    assert_eq!(doc.props.paper.size, "A4");
    assert_eq!(doc.props.stylesheet.rules.len(), 2);
    assert_eq!(doc.body.children.len(), 1);
}
