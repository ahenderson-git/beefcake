import { escapeHtml, fmtBytes } from "../utils";

export function renderDashboardView(state: any): string {
  return `
    <div class="dashboard">
      <div class="hero">
        <h1>beefcake <small>v${state.version}</small></h1>
        <p>Developed by Anthony Henderson</p>
      </div>
      <div class="info-box">
        <div class="info-section">
          <h3>What is beefcake?</h3>
          <p>
            <strong>beefcake</strong> (v${state.version}) is a high-performance desktop application designed as an 
            <strong>Advanced Data Analysis and ETL (Extract, Transform, Load) Pipeline</strong>. 
            Built with <strong>Tauri</strong>, it leverages the speed of <strong>Rust</strong> and <strong>Polars</strong> 
            to provide a robust environment for inspecting, cleaning, and moving data from local files into production-ready databases.
          </p>
        </div>
        
        <div class="info-grid">
          <div class="info-item">
            <strong><i class="ph ph-stethoscope"></i> Data Profiling</strong>
            <span>Automatic health scores, risk identification, and detailed column statistics.</span>
          </div>
          <div class="info-item">
            <strong><i class="ph ph-magic-wand"></i> Smart Cleaning</strong>
            <span>Interactive tools for normalisation, imputation, case conversion, and encoding.</span>
          </div>
          <div class="info-item">
            <strong><i class="ph ph-database"></i> Seamless ETL</strong>
            <span>Push cleaned data directly to PostgreSQL with high-speed COPY commands.</span>
          </div>
          <div class="info-item">
            <strong><i class="ph ph-brain"></i> ML Insights</strong>
            <span>Train predictive models (Regression, Trees) directly on your analysed datasets.</span>
          </div>
        </div>
        
        <div class="info-footer">
          <p><i class="ph ph-terminal-window"></i> <strong>Technical Foundation:</strong> High-performance processing powered by Rust & Polars DataFrames.</p>
        </div>
      </div>

      <div class="stats-grid">
        <div class="stat-card">
          <h3>Local Storage</h3>
          <div class="stat-value">Active</div>
          <p>~/.beefcake_config.json</p>
        </div>
        <div class="stat-card">
          <h3>Connections</h3>
          <div class="stat-value">${state.config?.connections.length || 0}</div>
          <p>Configured Endpoints</p>
        </div>
        <div class="stat-card">
          <h3>Last Analysis</h3>
          <div class="stat-value">${state.analysisResponse ? escapeHtml(state.analysisResponse.file_name) : 'None'}</div>
          <p>${state.analysisResponse ? fmtBytes(state.analysisResponse.file_size) : 'Ready for input'}</p>
        </div>
      </div>
      <div class="actions">
        <button id="btn-open-file" class="btn-primary">
          <i class="ph ph-cloud-arrow-up"></i> Analyze New Dataset
        </button>
        <button id="btn-powershell" class="btn-secondary">
          <i class="ph ph-terminal"></i> PowerShell Console
        </button>
        <button id="btn-python" class="btn-secondary">
          <i class="ph ph-code"></i> Python IDE
        </button>
        <button id="btn-sql" class="btn-secondary">
          <i class="ph ph-database"></i> SQL Lab
        </button>
      </div>
    </div>
  `;
}
