export interface ProjectFile {
  id: string;
  name: string;
  language: 'ddl' | 'css' | 'json';
  content: string;
}

export const INITIAL_FILES: ProjectFile[] = [
  {
    id: 'document-dxml',
    name: 'document.dxml',
    language: 'ddl',
    content: `<doc>
  <props>
    <meta>
      <title>Modular DDL Report</title>
      <author>Engineering Team</author>
    </meta>

    <!-- Global Paper Defaults -->
    <paper size="A4" orientation="portrait" margin="20mm" />

    <!-- External Link to Stylesheet -->
    <link rel="stylesheet" href="styles.css" />

    <!-- Named Page Styles -->
    <page-styles>
      <page-style name="body" default="true">
        <running-header>
          <left><text>Technical Report</text></left>
          <right><page-number/> of <total-pages/></right>
        </running-header>
        <running-footer>
          <center><text>© 2026 Autonomous Document Systems</text></center>
        </running-footer>
      </page-style>
    </page-styles>
  </props>

  <body>
    <section page-style="body">
      <h1>Modular Document Architecture</h1>
      <p class="lead">
        Content and presentation are decoupled into separate files: <code>document.dxml</code> and <code>styles.css</code>.
      </p>

      <div class="callout">
        <b>Separation of Concerns:</b> Styles are linked via <code>&lt;link rel="stylesheet" href="styles.css" /&gt;</code>, allowing designers or AI agents to style documents independently.
      </div>

      <h2>Multi-Column Grid Flow</h2>
      <grid cols="1fr 1fr" gap="6mm">
        <div class="col-box">
          <h3>Left Column</h3>
          <p>
            The layout rules defined in the standalone <code>styles.css</code> file apply automatically across the document.
          </p>
        </div>
        <div class="col-box">
          <h3>Right Column</h3>
          <p>
            You can add new files, switch between them using the tabs above, or format them individually.
          </p>
        </div>
      </grid>
    </section>
  </body>
</doc>`,
  },
  {
    id: 'styles-css',
    name: 'styles.css',
    language: 'css',
    content: `/* Global Document Styles */
h1 {
  font-size: 24pt;
  font-weight: bold;
  color: #1e3a8a;
  margin-bottom: 6mm;
}

h2 {
  font-size: 15pt;
  font-weight: bold;
  color: #0f172a;
  margin-top: 6mm;
  margin-bottom: 3mm;
}

h3 {
  font-size: 11pt;
  font-weight: 600;
  color: #1e293b;
  margin-bottom: 2mm;
}

p {
  line-height: 1.5;
  color: #334155;
  margin-bottom: 4mm;
  font-size: 10.5pt;
}

ul, ol {
  padding-left: 24px;
  margin-bottom: 3.5mm;
}

li {
  margin-bottom: 1.5mm;
  line-height: 1.6;
}

.lead {
  font-size: 12pt;
  color: #475569;
}

.callout {
  background: #f0fdf4;
  border-left: 4pt solid #16a34a;
  padding: 4mm 6mm;
  margin: 4mm 0;
  border-radius: 4px;
}

.col-box {
  padding: 4mm;
  background: #f8fafc;
  border-radius: 4px;
  border: 1px solid #e2e8f0;
}
`,
  },
];
