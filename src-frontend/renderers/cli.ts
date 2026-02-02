export function renderCliHelpView(): string {
  return `
    <div class="cli-help-view">
      <div class="cli-header-box">
        <h3><i class="ph ph-terminal-window"></i> Command Line Interface</h3>
        <p>Beefcake supports a robust CLI for automation and headless processing. Perfect for scheduled tasks, ETL workflows, and server-side operations.</p>
      </div>

      <div class="cli-section">
        <h3><i class="ph ph-command"></i> Base Command</h3>
        <div class="cli-card command-main">
          <code>beefcake [SUBCOMMAND] [OPTIONS]</code>
        </div>
        <p class="help-text">Run <code>beefcake --help</code> for full command documentation</p>
      </div>

      <div class="cli-section">
        <h3><i class="ph ph-list-numbers"></i> Available Subcommands</h3>
        <div class="cli-grid">
          <div class="cli-card">
            <h5><i class="ph ph-upload"></i> import</h5>
            <div class="code-block">
              <pre><code>beefcake import --file &lt;PATH&gt; --table &lt;NAME&gt; \\
  --db-url &lt;URL&gt; [--clean] [--config &lt;JSON&gt;]</code></pre>
            </div>
            <p><strong>Purpose:</strong> Import a local file (CSV, JSON, Parquet) into a PostgreSQL database table.</p>
            <ul class="feature-list">
              <li><strong>--file</strong>: Path to input file (defaults to first file in input directory)</li>
              <li><strong>--table</strong>: Target table name (defaults to filename)</li>
              <li><strong>--schema</strong>: Target schema (default: public)</li>
              <li><strong>--db-url</strong>: Database connection string (or use DATABASE_URL env var)</li>
              <li><strong>--clean</strong>: Apply automatic cleaning heuristics before import</li>
              <li><strong>--config</strong>: Path to JSON cleaning configuration file</li>
            </ul>
            <div class="example-box">
              <strong>Example:</strong>
              <pre><code>beefcake import --file data.csv --table customers \\
  --db-url postgres://user:pass@localhost/db --clean</code></pre>
            </div>
          </div>

          <div class="cli-card">
            <h5><i class="ph ph-download"></i> export</h5>
            <div class="code-block">
              <pre><code>beefcake export --input &lt;SOURCE&gt; --output &lt;PATH&gt; \\
  [--db-url &lt;URL&gt;] [--clean] [--config &lt;JSON&gt;]</code></pre>
            </div>
            <p><strong>Purpose:</strong> Export data from a database table or file to a different format.</p>
            <ul class="feature-list">
              <li><strong>--input</strong>: Source file or table name</li>
              <li><strong>--output</strong>: Output file path</li>
              <li><strong>--db-url</strong>: Database URL (required if input is a table)</li>
              <li><strong>--schema</strong>: Source schema (default: public)</li>
              <li><strong>--clean</strong>: Apply cleaning before export</li>
              <li><strong>--config</strong>: Path to JSON cleaning configuration</li>
            </ul>
            <div class="example-box">
              <strong>Example:</strong>
              <pre><code>beefcake export --input sales_table --output sales.parquet \\
  --db-url postgres://localhost/db</code></pre>
            </div>
          </div>

          <div class="cli-card">
            <h5><i class="ph ph-broom"></i> clean</h5>
            <div class="code-block">
              <pre><code>beefcake clean --file &lt;INPUT&gt; --output &lt;OUTPUT&gt; \\
  [--config &lt;JSON&gt;]</code></pre>
            </div>
            <p><strong>Purpose:</strong> Clean a data file and save the result to a new file.</p>
            <ul class="feature-list">
              <li><strong>--file</strong>: Input file path (defaults to first file in input directory)</li>
              <li><strong>--output</strong>: Output file path (defaults to processed directory)</li>
              <li><strong>--config</strong>: Path to JSON cleaning configuration file</li>
            </ul>
            <div class="example-box">
              <strong>Example:</strong>
              <pre><code>beefcake clean --file raw_data.csv --output cleaned_data.parquet \\
  --config cleaning_rules.json</code></pre>
            </div>
          </div>

          <div class="cli-card">
            <h5><i class="ph ph-flow-arrow"></i> run</h5>
            <div class="code-block">
              <pre><code>beefcake run --spec &lt;PIPELINE&gt; --input &lt;DATA&gt; \\
  [--output &lt;PATH&gt;] [--log &lt;PATH&gt;] [--fail-on-warnings]</code></pre>
            </div>
            <p><strong>Purpose:</strong> Execute a saved pipeline specification on a data file.</p>
            <ul class="feature-list">
              <li><strong>--spec</strong>: Path to pipeline JSON specification (required)</li>
              <li><strong>--input</strong>: Path to input data file (required)</li>
              <li><strong>--output</strong>: Override output path from spec</li>
              <li><strong>--date</strong>: Date string for path template (YYYY-MM-DD, default: today)</li>
              <li><strong>--log</strong>: Path to write execution log</li>
              <li><strong>--fail-on-warnings</strong>: Treat warnings as errors</li>
            </ul>
            <div class="example-box">
              <strong>Example:</strong>
              <pre><code>beefcake run --spec pipelines/clean_sales.json \\
  --input raw/sales_2026-02.csv --output processed/sales_2026-02.parquet \\
  --log logs/sales_2026-02.log</code></pre>
            </div>
          </div>
        </div>
      </div>

      <div class="cli-section">
        <h3><i class="ph ph-gear"></i> Configuration</h3>
        <div class="cli-card">
          <h5>Cleaning Configuration JSON Format</h5>
          <p>When using <code>--config</code>, provide a JSON file mapping column names to cleaning rules:</p>
          <div class="code-block">
            <pre><code>{
  "column_name": {
    "trim": true,
    "drop": false,
    "impute": "mean",
    "convert_type": "i64"
  }
}</code></pre>
          </div>
          <ul class="feature-list">
            <li><strong>trim</strong>: Remove leading/trailing whitespace (boolean)</li>
            <li><strong>drop</strong>: Drop this column entirely (boolean)</li>
            <li><strong>impute</strong>: Fill missing values ("mean", "median", "mode", "zero", or a literal value)</li>
            <li><strong>convert_type</strong>: Cast to data type ("i64", "f64", "str", "bool")</li>
          </ul>
        </div>
      </div>

      <div class="cli-section">
        <h3><i class="ph ph-info"></i> Process Information</h3>
        <div class="cli-card">
          <div class="notes-grid">
            <div class="note-item">
              <strong>Exit Codes</strong>
              <ul class="exit-codes">
                <li><code>0</code>: Success</li>
                <li><code>1</code>: Error (validation failed, file not found, etc.)</li>
                <li><code>101</code>: Panic (unexpected internal error)</li>
              </ul>
            </div>
            <div class="note-item">
              <strong>Environment Variables</strong>
              <ul>
                <li><code>DATABASE_URL</code>: Default PostgreSQL connection string</li>
                <li><code>RUST_LOG</code>: Set logging level (error, warn, info, debug, trace)</li>
              </ul>
            </div>
          </div>
        </div>
      </div>

      <div class="cli-section">
        <h3><i class="ph ph-lightbulb"></i> Usage Tips</h3>
        <div class="cli-card">
          <ul class="tips-list">
            <li><strong>Automation:</strong> All CLI commands can be scheduled with cron (Linux/macOS) or Task Scheduler (Windows)</li>
            <li><strong>Pipelines:</strong> Create pipelines in the GUI, then execute them via CLI for production workflows</li>
            <li><strong>PowerShell:</strong> Export pipelines as PowerShell scripts for Windows automation</li>
            <li><strong>Logging:</strong> Use <code>--log</code> option to capture execution details for auditing</li>
            <li><strong>Performance:</strong> Parquet format recommended for large datasets (10x faster than CSV)</li>
            <li><strong>Database URLs:</strong> Format: <code>postgres://user:password@host:port/database</code></li>
          </ul>
        </div>
      </div>

      <div class="cli-section">
        <h3><i class="ph ph-question"></i> Need More Help?</h3>
        <div class="cli-card">
          <p>For detailed command help and options:</p>
          <div class="code-block">
            <pre><code>beefcake --help              # General help
beefcake import --help       # Import command help
beefcake export --help       # Export command help
beefcake clean --help        # Clean command help
beefcake run --help          # Run command help</code></pre>
          </div>
          <p class="help-footer">See <a href="https://github.com/yourusername/beefcake/docs/AUTOMATION.md">AUTOMATION.md</a> for comprehensive automation examples and best practices.</p>
        </div>
      </div>
    </div>
  `;
}
