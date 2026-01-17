/**
 * Pipeline Management API
 *
 * Wraps Tauri commands for creating, managing, and executing
 * data transformation pipelines.
 *
 * ## Backend Integration
 *
 * These functions call Rust commands defined in `src/tauri_app.rs`:
 * - `list_pipeline_specs`: Get all saved pipelines
 * - `load_pipeline_spec`: Load pipeline from JSON
 * - `save_pipeline_spec`: Save pipeline to JSON
 * - `validate_pipeline_spec`: Check pipeline validity
 * - `execute_pipeline_spec`: Run pipeline on dataset
 * - `generate_powershell`: Export pipeline as PowerShell script
 * - `pipeline_from_configs`: Create pipeline from clean configs
 *
 * ## Example Usage
 *
 * ```typescript
 * // List all pipelines
 * const pipelines = await listPipelines();
 *
 * // Load specific pipeline
 * const spec = await loadPipeline('pipelines/cleaning.json');
 *
 * // Execute pipeline
 * const result = await executePipeline(spec, 'data.csv');
 * ```
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Information about a saved pipeline file.
 */
export interface PipelineInfo {
  /** Pipeline display name */
  name: string;

  /** Full path to pipeline JSON file */
  path: string;

  /** ISO 8601 creation timestamp */
  created: string;

  /** ISO 8601 last modified timestamp */
  modified: string;

  /** Number of transformation steps */
  step_count: number;

  /** Optional pipeline description */
  description?: string;
}

/**
 * Complete pipeline specification.
 *
 * This is the main data structure for a pipeline. It's serialized
 * to JSON and can be saved, loaded, and executed.
 */
export interface PipelineSpec {
  /** Pipeline name (used for display and file naming) */
  name: string;

  /** Optional description of pipeline purpose */
  description?: string;

  /** Ordered list of transformation steps */
  steps: PipelineStep[];

  /** Input configuration */
  input?: {
    format?: string;
    path?: string;
  };

  /** Output configuration */
  output?: {
    format?: string;
    path?: string;
  };

  /** Spec version */
  version?: string;
}

/**
 * A single transformation step in a pipeline.
 *
 * Each step has a specific type and associated parameters.
 */
export interface PipelineStep {
  /** Step type (e.g., "Clean", "Filter", "Select") */
  [key: string]: unknown;
}

/**
 * Result of pipeline execution.
 */
export interface ExecutionResult {
  /** Whether execution completed successfully */
  success: boolean;

  /** Number of rows before execution */
  rows_before: number;

  /** Number of rows after execution */
  rows_after: number;

  /** Number of columns before execution */
  columns_before: number;

  /** Number of columns after execution */
  columns_after: number;

  /** Number of steps successfully applied */
  steps_applied: number;

  /** Warning messages generated during execution */
  warnings: string[];

  /** Total execution time in seconds */
  duration_secs: number;

  /** Human-readable summary */
  summary: string;
}

/**
 * Pipeline validation result.
 */
export interface ValidationResult {
  /** List of validation errors (empty if valid) */
  errors: string[];
}

/**
 * Lists all saved pipeline specifications.
 *
 * **Backend**: Calls `list_pipeline_specs` in `src/tauri_app.rs`
 *
 * Scans the pipelines directory and returns metadata for each
 * pipeline JSON file found.
 *
 * @returns Promise resolving to array of pipeline info
 * @throws Error string if pipelines directory cannot be accessed
 */
export async function listPipelines(): Promise<PipelineInfo[]> {
  try {
    const json = await invoke<string>('list_pipeline_specs');
    return JSON.parse(json) as PipelineInfo[];
  } catch (error) {
    console.error('Failed to list pipelines:', error);
    throw error;
  }
}

/**
 * Loads a pipeline specification from file.
 *
 * **Backend**: Calls `load_pipeline_spec` in `src/tauri_app.rs`
 *
 * Reads and parses a pipeline JSON file into a PipelineSpec object.
 *
 * @param path - Absolute or relative path to pipeline JSON file
 * @returns Promise resolving to pipeline specification
 * @throws Error string if file not found or invalid JSON
 *
 * @example
 * ```typescript
 * const spec = await loadPipeline('pipelines/data_cleaning.json');
 * console.log(`Pipeline has ${spec.steps.length} steps`);
 * ```
 */
export async function loadPipeline(path: string): Promise<PipelineSpec> {
  try {
    const json = await invoke<string>('load_pipeline_spec', { path });
    return JSON.parse(json) as PipelineSpec;
  } catch (error) {
    console.error(`Failed to load pipeline from ${path}:`, error);
    throw error;
  }
}

/**
 * Deletes a pipeline specification file.
 *
 * **Backend**: Calls `delete_pipeline_spec` in `src/tauri_app.rs`
 *
 * @param path - Absolute or relative path to pipeline JSON file
 * @returns Promise resolving to void
 * @throws Error string if deletion fails
 */
export async function deletePipeline(path: string): Promise<void> {
  try {
    await invoke<void>('delete_pipeline_spec', { path });
  } catch (error) {
    console.error(`Failed to delete pipeline at ${path}:`, error);
    throw error;
  }
}

/**
 * Saves a pipeline specification to file.
 *
 * **Backend**: Calls `save_pipeline_spec` in `src/tauri_app.rs`
 *
 * Serializes pipeline to JSON and writes to specified path.
 * Creates parent directories if needed.
 *
 * @param path - Path where to save pipeline (relative or absolute)
 * @param spec - Pipeline specification to save
 * @returns Promise resolving to void
 * @throws Error string if write fails or path invalid
 *
 * @example
 * ```typescript
 * const spec: PipelineSpec = {
 *     name: 'My Pipeline',
 *     steps: [
 *         { Clean: { columns: ['all'], trim_whitespace: true } }
 *     ]
 * };
 * await savePipeline('pipelines/my_pipeline.json', spec);
 * ```
 */
export async function savePipeline(path: string, spec: PipelineSpec): Promise<void> {
  try {
    const specJson = JSON.stringify(spec);
    await invoke<void>('save_pipeline_spec', { specJson, path });
  } catch (error) {
    console.error(`Failed to save pipeline to ${path}:`, error);
    throw error;
  }
}

/**
 * Validates a pipeline specification against an input dataset.
 *
 * **Backend**: Calls `validate_pipeline_spec` in `src/tauri_app.rs`
 *
 * Checks pipeline for:
 * - Valid step types
 * - Required parameters for each step
 * - Compatible data types with input schema
 * - Column references exist
 *
 * @param spec - Pipeline specification to validate
 * @param inputPath - Path to input dataset for schema validation
 * @returns Promise resolving to validation result
 *
 * @example
 * ```typescript
 * const result = await validatePipeline(spec, 'data.csv');
 * if (result.errors.length > 0) {
 *     console.error('Pipeline errors:', result.errors);
 * }
 * ```
 */
export async function validatePipeline(
  spec: PipelineSpec,
  inputPath: string
): Promise<ValidationResult> {
  try {
    const specJson = JSON.stringify(spec);
    const errors = await invoke<string[]>('validate_pipeline_spec', {
      specJson,
      inputPath,
    });
    return { errors };
  } catch (error) {
    console.error('Failed to validate pipeline:', error);
    throw error;
  }
}

/**
 * Executes a pipeline on a dataset.
 *
 * **Backend**: Calls `execute_pipeline_spec` in `src/tauri_app.rs`
 *
 * Runs each pipeline step in sequence, passing output from one
 * step as input to the next. Can optionally save result to file.
 *
 * ## Progress Tracking
 *
 * Pipeline execution can be long-running. The result includes
 * timing information and step counts.
 *
 * @param spec - Pipeline specification to execute
 * @param inputPath - Path to input dataset (CSV, JSON, or Parquet)
 * @param outputPath - Optional path for output (uses spec default if omitted)
 * @returns Promise resolving to execution result
 * @throws Error string if execution fails
 *
 * @example
 * ```typescript
 * try {
 *     const result = await executePipeline(
 *         spec,
 *         'data/sales.csv',
 *         'data/sales_cleaned.csv'
 *     );
 *
 *     if (result.success) {
 *         console.log(`Processed ${result.rows_after} rows in ${result.duration_secs}s`);
 *     }
 * } catch (error) {
 *     console.error('Pipeline failed:', error);
 * }
 * ```
 */
export async function executePipeline(
  spec: PipelineSpec,
  inputPath: string,
  outputPath?: string
): Promise<ExecutionResult> {
  try {
    const specJson = JSON.stringify(spec);
    const json = await invoke<string>('execute_pipeline_spec', {
      specJson,
      inputPath,
      outputPath: outputPath ?? null,
    });
    return JSON.parse(json) as ExecutionResult;
  } catch (error) {
    console.error('Failed to execute pipeline:', error);
    throw error;
  }
}

/**
 * Generates a PowerShell script from a pipeline specification.
 *
 * **Backend**: Calls `generate_powershell` in `src/tauri_app.rs`
 *
 * Creates a standalone PowerShell script that can execute the
 * pipeline without the GUI, useful for automation and scheduling.
 *
 * @param spec - Pipeline specification to export
 * @param outputPath - Path where to save the .ps1 file
 * @returns Promise resolving to success message
 *
 * @example
 * ```typescript
 * await generatePowerShell(spec, 'scripts/data_pipeline.ps1');
 * ```
 */
export async function generatePowerShell(spec: PipelineSpec, outputPath: string): Promise<string> {
  try {
    const specJson = JSON.stringify(spec);
    return await invoke<string>('generate_powershell', {
      specJson,
      outputPath,
    });
  } catch (error) {
    console.error('Failed to generate PowerShell:', error);
    throw error;
  }
}

/**
 * Creates a pipeline from cleaning configurations.
 *
 * **Backend**: Calls `pipeline_from_configs` in `src/tauri_app.rs`
 *
 * Converts column cleaning configs (from the Cleaning stage)
 * into a reusable pipeline specification.
 *
 * @param name - Name for the pipeline
 * @param configs - Column cleaning configurations
 * @param inputFormat - Input file format
 * @param outputPath - Output file path
 * @returns Promise resolving to pipeline spec JSON
 */
export async function pipelineFromConfigs(
  name: string,
  configs: Record<string, unknown>,
  inputFormat: string,
  outputPath: string
): Promise<PipelineSpec> {
  try {
    const configsJson = JSON.stringify(configs);
    const json = await invoke<string>('pipeline_from_configs', {
      name,
      configsJson,
      inputFormat,
      outputPath,
    });
    return JSON.parse(json) as PipelineSpec;
  } catch (error) {
    console.error('Failed to create pipeline from configs:', error);
    throw error;
  }
}

/**
 * Lists all available pipeline templates.
 *
 * **Backend**: Calls `list_pipeline_templates` in `src/tauri_app.rs`
 *
 * Scans the templates directory and returns metadata for each
 * template JSON file found.
 *
 * @returns Promise resolving to array of template info
 * @throws Error string if templates directory cannot be accessed
 */
export async function listTemplates(): Promise<PipelineInfo[]> {
  try {
    const json = await invoke<string>('list_pipeline_templates');
    return JSON.parse(json) as PipelineInfo[];
  } catch (error) {
    console.error('Failed to list templates:', error);
    throw error;
  }
}

/**
 * Loads a pipeline template by name.
 *
 * **Backend**: Calls `load_pipeline_template` in `src/tauri_app.rs`
 *
 * Reads and parses a template JSON file into a PipelineSpec object.
 * The template name is converted to the corresponding filename
 * (e.g., "Data Cleaning" -> "data-cleaning.json").
 *
 * @param templateName - Name of the template to load
 * @returns Promise resolving to pipeline specification
 * @throws Error string if template not found or invalid JSON
 *
 * @example
 * ```typescript
 * const spec = await loadTemplate('Data Cleaning');
 * console.log(`Template has ${spec.steps.length} steps`);
 * ```
 */
export async function loadTemplate(templateName: string): Promise<PipelineSpec> {
  try {
    const json = await invoke<string>('load_pipeline_template', {
      templateName,
    });
    return JSON.parse(json) as PipelineSpec;
  } catch (error) {
    console.error(`Failed to load template ${templateName}:`, error);
    throw error;
  }
}
