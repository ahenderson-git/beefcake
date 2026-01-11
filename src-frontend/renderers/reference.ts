export function renderReferenceView(): string {
  return `
    <div class="reference-view">
      <div class="reference-header-box">
        <h3><i class="ph ph-books"></i> Documentation & Reference</h3>
        <p>Your guide to the beefcake data processing engine.</p>
      </div>

      <div class="reference-grid">
        <section class="ref-section">
          <h3><i class="ph ph-magic-wand"></i> Data Cleaning Logic</h3>
          <div class="ref-card">
            <h4>The Clean Function Pipeline</h4>
            <p>The clean function processes data through multiple stages in this order:</p>
            <ol>
              <li><strong>Text Cleaning & Regex:</strong>
                <ul>
                  <li>Trim whitespace from values</li>
                  <li>Convert text case (lowercase, uppercase, titlecase)</li>
                  <li>Remove special characters (non-alphanumeric)</li>
                  <li>Remove non-ASCII characters</li>
                  <li>Apply custom regex find/replace patterns</li>
                  <li>Standardize null representations ("null", "NULL", "", "N/A", "nan", "NaN")</li>
                </ul>
              </li>
              <li><strong>Number Extraction:</strong>
                <ul>
                  <li>Extract numeric values from text using regex pattern</li>
                  <li>Converts result to Float64 type</li>
                </ul>
              </li>
              <li><strong>Type Casting:</strong>
                <ul>
                  <li>Convert to target data type (Numeric, Text, Boolean, Temporal, Categorical)</li>
                  <li>Boolean casting recognizes "true"/"false", "yes"/"no", "1"/"0"</li>
                </ul>
              </li>
              <li><strong>Imputation (Null Handling):</strong>
                <ul>
                  <li><strong>Mean:</strong> Replace nulls with column average</li>
                  <li><strong>Median:</strong> Replace nulls with middle value</li>
                  <li><strong>Mode:</strong> Replace nulls with most frequent value</li>
                  <li><strong>Zero:</strong> Replace nulls with 0</li>
                </ul>
              </li>
              <li><strong>Numeric Refinement:</strong>
                <ul>
                  <li><strong>Outlier Clipping:</strong> Clips values to 5th-95th percentile range</li>
                  <li><strong>Rounding:</strong> Round to specified decimal places</li>
                </ul>
              </li>
              <li><strong>Normalization (Scaling):</strong>
                <ul>
                  <li><strong>Z-Score:</strong> Scales data to mean=0 and std_dev=1</li>
                  <li><strong>Min-Max:</strong> Scales data to range [0, 1]</li>
                </ul>
              </li>
              <li><strong>Column Renaming:</strong>
                <ul>
                  <li>Applies new column names if specified</li>
                </ul>
              </li>
              <li><strong>Categorical Encoding:</strong>
                <ul>
                  <li><strong>One-Hot Encoding:</strong> Creates binary columns for each unique categorical value</li>
                  <li>Removes original categorical column and replaces with new binary columns</li>
                </ul>
              </li>
            </ol>
            <p><small>Note: Operations are applied lazily for optimal performance. Some operations are restricted in certain contexts.</small></p>
          </div>
        </section>

        <section class="ref-section">
          <h3><i class="ph ph-database"></i> SQL Context</h3>
          <div class="ref-card">
            <p>The SQL Lab allows you to query the currently analyzed dataset using Polars SQL syntax. The dataset is automatically registered as a table named <code>data</code>.</p>
            <div class="code-block">
              <pre><code>SELECT column_a, COUNT(*) 
FROM data 
GROUP BY column_a 
HAVING COUNT(*) > 10</code></pre>
            </div>
            <p><small>Note: SQL queries are executed lazily for performance.</small></p>
          </div>
        </section>

        <section class="ref-section">
          <h3><i class="ph ph-code"></i> Python Scripting</h3>
          <div class="ref-card">
            <p>The Python IDE provides full access to the Polars library. If an analysis is active, the dataset is available as a LazyFrame named <code>df</code>.</p>
            <div class="code-block">
              <pre><code># Example: Custom transformation
df = df.with_columns([
    (pl.col("price") * 1.1).alias("price_with_tax")
])</code></pre>
            </div>
            <p>To export from your script, ensure your final result is stored in the <code>df</code> variable.</p>
          </div>
        </section>

        <section class="ref-section">
          <h3><i class="ph ph-terminal"></i> CLI Usage</h3>
          <div class="ref-card">
            <p>beefcake can be run from the command line for automated tasks. Use <code>--help</code> to see all options.</p>
            <div class="code-block">
              <pre><code>beefcake.exe analyze "data.csv" --export "cleaned.parquet"</code></pre>
            </div>
          </div>
        </section>

        <section class="ref-section">
          <h3><i class="ph ph-link"></i> External Resources</h3>
          <div class="link-grid">
            <a href="https://pola-rs.github.io/polars/py-polars/html/reference/index.html" target="_blank" class="link-card">
              <div class="link-icon"><i class="ph ph-file-text"></i></div>
              <div class="link-content">
                <strong>Polars API Docs</strong>
                <span>Official documentation for Polars expressions.</span>
              </div>
            </a>
            <a href="https://github.com/pola-rs/polars" target="_blank" class="link-card">
              <div class="link-icon"><i class="ph ph-github-logo"></i></div>
              <div class="link-content">
                <strong>Polars GitHub</strong>
                <span>Source code and community discussions.</span>
              </div>
            </a>
            <a href="https://github.com/rust-unofficial/awesome-rust" target="_blank" class="link-card">
              <div class="link-icon"><i class="ph ph-rust-logo"></i></div>
              <div class="link-content">
                <strong>Awesome Rust</strong>
                <span>A curated list of Rust resources.</span>
              </div>
            </a>
          </div>
        </section>
      </div>

      <div class="reference-content">
        <div class="content-card">
          <h3><i class="ph ph-chart-line"></i> Understanding Data Skewness</h3>
          <div class="skew-grid">
            <div class="skew-item">
              <strong>Right Skew (Positive)</strong>
              <p>The mean is greater than the median. High-value outliers pull the average up.</p>
            </div>
            <div class="skew-item">
              <strong>Left Skew (Negative)</strong>
              <p>The mean is less than the median. Low-value outliers pull the average down.</p>
            </div>
          </div>
        </div>

        <div class="content-card">
          <h3><i class="ph ph-magic-wand"></i> Preprocessing for Machine Learning</h3>
          <div class="ml-grid">
            <div class="ml-item">
              <h4>Normalization (Scaling)</h4>
              <ul>
                <li><strong>Min-Max:</strong> Rescales to [0, 1]. Best when bounds are known.</li>
                <li><strong>Z-Score:</strong> Mean=0, StdDev=1. Robust to outliers.</li>
              </ul>
            </div>
            <div class="ml-item">
              <h4>Categorical Encoding</h4>
              <ul>
                <li><strong>One-Hot:</strong> Binary columns for each category. Best for non-ordered data.</li>
                <li><strong>Label:</strong> Assigns integers. Better for ordered data (Ordinal).</li>
              </ul>
            </div>
            <div class="ml-item">
              <h4>Imputation (Handling Nulls)</h4>
              <ul>
                <li><strong>Mean/Median:</strong> Good for numeric fields.</li>
                <li><strong>Mode:</strong> Best for categorical fields.</li>
                <li><strong>Constant:</strong> When 'missing' has a business meaning.</li>
              </ul>
            </div>
          </div>
        </div>

        <div class="content-card">
          <h3><i class="ph ph-database"></i> PostgreSQL Export Guide</h3>
          <p>Beefcake handles metadata (file size, health score) and summaries automatically. When exporting to PostgreSQL:</p>
          <ul>
            <li>Column types are inferred and mapped to Postgres types.</li>
            <li>High-speed COPY commands are used for efficiency.</li>
            <li>Metadata and statistics are saved to the target database.</li>
          </ul>
        </div>
      </div>
    </div>
  `;
}
