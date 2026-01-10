export function renderCliHelpView(): string {
  return `
    <div class="cli-help-view">
      <div class="cli-header-box">
        <h3><i class="ph ph-terminal-window"></i> Command Line Interface</h3>
        <p>beefcake supports a robust CLI for automation and headless processing. Perfect for scheduled tasks and server-side operations.</p>
      </div>
      
      <div class="cli-section">
        <h3><i class="ph ph-command"></i> Base Command</h3>
        <div class="cli-card command-main">
          <code>beefcake.exe [SUBCOMMAND] [OPTIONS]</code>
        </div>
      </div>

      <div class="cli-section">
        <h3><i class="ph ph-list-numbers"></i> Available Subcommands</h3>
        <div class="cli-grid">
          <div class="cli-card">
            <h5>1. Analysis</h5>
            <div class="code-block">
              <pre><code>analyze &lt;PATH&gt; [--trim &lt;PCT&gt;] [--export &lt;OUT&gt;]</code></pre>
            </div>
            <p>Analyzes a file and optionally exports a cleaned version using the default auto-cleaning rules. Ideal for rapid profiling.</p>
          </div>

          <div class="cli-card">
            <h5>2. SQL (Headless)</h5>
            <div class="code-block">
              <pre><code>query --path &lt;DATA&gt; --sql "SELECT * FROM data"</code></pre>
            </div>
            <p>Runs a SQL query against a local file without opening the UI. Results can be piped or exported.</p>
          </div>

          <div class="cli-card">
            <h5>3. Database Push</h5>
            <div class="code-block">
              <pre><code>push --path &lt;DATA&gt; --conn &lt;ID&gt; --table &lt;NAME&gt;</code></pre>
            </div>
            <p>Directly pushes a local file to a configured database connection using high-speed streaming.</p>
          </div>
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
                <li><code>1</code>: General Error / Validation Failed</li>
                <li><code>2</code>: OOM / Process Panicked</li>
              </ul>
            </div>
            <div class="note-item">
              <strong>Logs & Verbosity</strong>
              <p>Beefcake logs to <code>stdout</code> for standard output and <code>stderr</code> for errors. Use <code>--verbose</code> for detailed execution tracing.</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  `;
}
