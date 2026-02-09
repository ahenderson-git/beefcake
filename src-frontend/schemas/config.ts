import { z } from 'zod';

/**
 * Zod Runtime Type Validation Schemas for AppConfig
 *
 * These schemas provide runtime validation of data coming from the Rust backend.
 * They ensure that the JSON structure matches our TypeScript types at runtime,
 * catching schema mismatches before they cause errors.
 *
 * Key Benefits:
 * - Catches type mismatches at runtime (e.g. array vs object)
 * - Provides clear, actionable error messages
 * - Self-documenting: schema is the source of truth
 * - Can generate TypeScript types from schemas (type inference)
 */

// Database Connection Schema
export const DbConnectionSchema = z.object({
  id: z.string(),
  name: z.string(),
  settings: z.object({
    db_type: z.string(),
    host: z.string(),
    port: z.string(),
    user: z.string(),
    password: z.string(),
    database: z.string(),
    schema: z.string(),
    table: z.string(),
  }),
});

// AI Configuration Schema
export const AIConfigSchema = z.object({
  enabled: z.boolean(),
  model: z.string(),
  temperature: z.number(),
  max_tokens: z.number(),
});

// Audit Entry Schema
export const AuditEntrySchema = z.object({
  timestamp: z.string(),
  action: z.string(),
  details: z.string(),
});

// **CRITICAL**: Audit Log is an OBJECT with 'entries' array, NOT a flat array
// This schema would have caught the bug we just fixed!
export const AuditLogSchema = z.object({
  entries: z.array(AuditEntrySchema),
});

// AppConfig Schema - Frontend Flat Structure
// Note: The frontend expects a flattened structure, but the backend returns nested
export const AppConfigSchema = z.object({
  connections: z.array(DbConnectionSchema),
  active_import_id: z.string().nullable(),
  active_export_id: z.string().nullable(),
  powershell_font_size: z.number(),
  python_font_size: z.number(),
  sql_font_size: z.number(),
  analysis_sample_size: z.number().optional(),
  sampling_strategy: z.string().optional(),
  first_run_completed: z.boolean().optional(),
  trusted_paths: z.array(z.string()).optional(),
  security_warning_acknowledged: z.boolean().optional(),
  ai_config: AIConfigSchema.optional(),
  // **CRITICAL FIELD**: This must be an object with entries, not a flat array!
  audit_log: AuditLogSchema,
});

// Backend Response Schema - Nested Structure from Rust
// The Rust backend returns this structure with 'settings' and 'audit_log' at top level
export const BackendAppConfigSchema = z.object({
  settings: z.object({
    connections: z.array(DbConnectionSchema),
    active_import_id: z.string().nullable(),
    active_export_id: z.string().nullable(),
    powershell_font_size: z.number(),
    python_font_size: z.number(),
    sql_font_size: z.number(),
    first_run_completed: z.boolean(),
    trusted_paths: z.array(z.string()),
    preview_row_limit: z.number(),
    security_warning_acknowledged: z.boolean(),
    skip_full_row_count: z.boolean(),
    analysis_sample_size: z.number(),
    sampling_strategy: z.string(),
    ai_config: AIConfigSchema,
  }),
  audit_log: AuditLogSchema,
});

/**
 * Validates and transforms backend config response to frontend format
 *
 * @param rawConfig - Raw config from Rust backend (nested structure)
 * @returns Validated and flattened config for frontend use
 * @throws ZodError if validation fails with detailed error message
 */
export function validateAndTransformBackendConfig(
  rawConfig: unknown
): z.infer<typeof AppConfigSchema> {
  // First, validate the backend structure
  const backendConfig = BackendAppConfigSchema.parse(rawConfig);

  // Transform to frontend flat structure
  return {
    connections: backendConfig.settings.connections,
    active_import_id: backendConfig.settings.active_import_id,
    active_export_id: backendConfig.settings.active_export_id,
    powershell_font_size: backendConfig.settings.powershell_font_size,
    python_font_size: backendConfig.settings.python_font_size,
    sql_font_size: backendConfig.settings.sql_font_size,
    analysis_sample_size: backendConfig.settings.analysis_sample_size,
    sampling_strategy: backendConfig.settings.sampling_strategy,
    first_run_completed: backendConfig.settings.first_run_completed,
    trusted_paths: backendConfig.settings.trusted_paths,
    security_warning_acknowledged: backendConfig.settings.security_warning_acknowledged,
    ai_config: backendConfig.settings.ai_config,
    audit_log: backendConfig.audit_log, // ← Correctly structured as object with entries
  };
}

/**
 * Safe validator that returns result object instead of throwing
 *
 * @param rawConfig - Raw config to validate
 * @returns Success object with data, or error object with message
 */
export function safeValidateBackendConfig(
  rawConfig: unknown
): { success: true; data: z.infer<typeof AppConfigSchema> } | { success: false; error: string } {
  try {
    const validated = validateAndTransformBackendConfig(rawConfig);
    return { success: true, data: validated };
  } catch (error) {
    if (error instanceof z.ZodError) {
      const firstError = error.issues[0];
      return {
        success: false,
        error: `Config validation failed at ${firstError?.path.join('.')}: ${firstError?.message}`,
      };
    }
    return {
      success: false,
      error: `Config validation failed: ${String(error)}`,
    };
  }
}

// Export inferred types
export type ValidatedAppConfig = z.infer<typeof AppConfigSchema>;
export type ValidatedBackendConfig = z.infer<typeof BackendAppConfigSchema>;

// ============================================================================
// Analysis Response Schemas
// ============================================================================

/**
 * Numeric statistics schema
 * CRITICAL: Rust serializes stats as { "Numeric": { ... } }, not as flat object
 */
export const NumericStatsSchema = z.object({
  min: z.number().nullable(),
  max: z.number().nullable(),
  mean: z.number().nullable(),
  median: z.number().nullable(),
  trimmed_mean: z.number().nullable(),
  std_dev: z.number().nullable(),
  q1: z.number().nullable(),
  q3: z.number().nullable(),
  p05: z.number().nullable(),
  p95: z.number().nullable(),
  skew: z.number().nullable(),
  distinct_count: z.number(),
  zero_count: z.number(),
  negative_count: z.number(),
  is_integer: z.boolean(),
  is_sorted: z.boolean(),
  is_sorted_rev: z.boolean(),
  bin_width: z.number(),
  histogram: z.array(z.tuple([z.number(), z.number(), z.number()])).nullable(),
});

export const TemporalStatsSchema = z.object({
  min: z.string().nullable(),
  max: z.string().nullable(),
  distinct_count: z.number(),
  p05: z.number().nullable(),
  p95: z.number().nullable(),
  is_sorted: z.boolean(),
  is_sorted_rev: z.boolean(),
  bin_width: z.number(),
  histogram: z.array(z.tuple([z.number(), z.number(), z.number()])).nullable(),
});

export const CategoricalStatsSchema = z.object({
  distinct_count: z.number(),
  top_values: z.array(z.tuple([z.string(), z.number()])),
});

export const BooleanStatsSchema = z.object({
  true_count: z.number(),
  false_count: z.number(),
});

export const TextStatsSchema = z.object({
  distinct_count: z.number(),
  top_value: z.tuple([z.string(), z.number()]).nullable(),
  min_length: z.number(),
  max_length: z.number(),
  avg_length: z.number(),
});

/**
 * CRITICAL: ColumnStats uses discriminated union structure
 * Rust serializes as: { "Numeric": { ... } } or { "Text": { ... } } etc.
 */
export const ColumnStatsSchema = z.object({
  Numeric: NumericStatsSchema.optional(),
  Temporal: TemporalStatsSchema.optional(),
  Categorical: CategoricalStatsSchema.optional(),
  Boolean: BooleanStatsSchema.optional(),
  Text: TextStatsSchema.optional(),
});

export const ColumnSummarySchema = z.object({
  name: z.string(),
  standardized_name: z.string(),
  kind: z.string(),
  count: z.number(),
  nulls: z.number(),
  stats: ColumnStatsSchema,
  interpretation: z.array(z.string()),
  ml_advice: z.array(z.string()),
  business_summary: z.array(z.string()),
  samples: z.array(z.string()),
});

export const FileHealthSchema = z.object({
  score: z.number(),
  risks: z.array(z.string()),
  notes: z.array(z.string()),
});

export const CorrelationMatrixSchema = z
  .object({
    columns: z.array(z.string()),
    data: z.array(z.array(z.number())),
  })
  .nullable();

export const AnalysisDurationSchema = z.object({
  secs: z.number(),
  nanos: z.number(),
});

export const AnalysisResponseSchema = z.object({
  file_name: z.string(),
  path: z.string(),
  file_size: z.number(),
  row_count: z.number(),
  total_row_count: z.number(),
  column_count: z.number(),
  analysis_duration: AnalysisDurationSchema,
  health: FileHealthSchema,
  correlation_matrix: CorrelationMatrixSchema,
  summary: z.array(ColumnSummarySchema),
});

/**
 * Validates analysis response from backend
 */
export function validateAnalysisResponse(
  rawResponse: unknown
): z.infer<typeof AnalysisResponseSchema> {
  return AnalysisResponseSchema.parse(rawResponse);
}

export function safeValidateAnalysisResponse(
  rawResponse: unknown
):
  | { success: true; data: z.infer<typeof AnalysisResponseSchema> }
  | { success: false; error: string } {
  try {
    const validated = validateAnalysisResponse(rawResponse);
    return { success: true, data: validated };
  } catch (error) {
    if (error instanceof z.ZodError) {
      const firstError = error.issues[0];
      return {
        success: false,
        error: `AnalysisResponse validation failed at ${firstError?.path.join('.')}: ${firstError?.message}`,
      };
    }
    return {
      success: false,
      error: `AnalysisResponse validation failed: ${String(error)}`,
    };
  }
}

// ============================================================================
// Dataset Lifecycle Schemas
// ============================================================================

export const LifecycleStageSchema = z.enum([
  'Raw',
  'Profiled',
  'Cleaned',
  'Advanced',
  'Validated',
  'Published',
]);

/**
 * CRITICAL: DataLocation is a Rust enum
 * Serializes as { "ParquetFile": "path" } or { "OriginalFile": "path" }
 */
export const DataLocationSchema = z.object({
  ParquetFile: z.string().optional(),
  OriginalFile: z.string().optional(),
  path: z.string().optional(),
});

export const TransformSpecSchema = z.object({
  transform_type: z.string(),
  parameters: z.record(z.string(), z.unknown()),
});

export const TransformPipelineSchema = z.object({
  transforms: z.array(TransformSpecSchema),
});

export const VersionMetadataSchema = z.object({
  description: z.string(),
  tags: z.array(z.string()),
  row_count: z.number().nullable(),
  column_count: z.number().nullable(),
  file_size_bytes: z.number().nullable(),
  created_by: z.string(),
  custom_fields: z.record(z.string(), z.unknown()),
});

export const DatasetVersionSchema = z.object({
  id: z.string(),
  dataset_id: z.string(),
  parent_id: z.string().nullable(),
  stage: LifecycleStageSchema,
  pipeline: TransformPipelineSchema,
  data_location: DataLocationSchema,
  metadata: VersionMetadataSchema,
  created_at: z.string(),
});

/**
 * Validates dataset version from backend
 */
export function validateDatasetVersion(rawVersion: unknown): z.infer<typeof DatasetVersionSchema> {
  return DatasetVersionSchema.parse(rawVersion);
}

export function safeValidateDatasetVersion(
  rawVersion: unknown
):
  | { success: true; data: z.infer<typeof DatasetVersionSchema> }
  | { success: false; error: string } {
  try {
    const validated = validateDatasetVersion(rawVersion);
    return { success: true, data: validated };
  } catch (error) {
    if (error instanceof z.ZodError) {
      const firstError = error.issues[0];
      return {
        success: false,
        error: `DatasetVersion validation failed at ${firstError?.path.join('.')}: ${firstError?.message}`,
      };
    }
    return {
      success: false,
      error: `DatasetVersion validation failed: ${String(error)}`,
    };
  }
}

// ============================================================================
// Integrity Verification Schemas
// ============================================================================

export const ProducerSchema = z.object({
  app_name: z.string(),
  app_version: z.string(),
  platform: z.string(),
});

export const ColumnInfoSchema = z.object({
  name: z.string(),
  dtype: z.string(),
});

export const ExportInfoSchema = z.object({
  filename: z.string(),
  format: z.string(),
  file_size_bytes: z.number(),
  row_count: z.number(),
  column_count: z.number(),
  schema: z.array(ColumnInfoSchema),
});

export const IntegrityInfoSchema = z.object({
  hash_algorithm: z.string(),
  hash: z.string(),
});

export const IntegrityReceiptSchema = z.object({
  receipt_version: z.number(),
  created_utc: z.string(),
  producer: ProducerSchema,
  export: ExportInfoSchema,
  integrity: IntegrityInfoSchema,
});

export const VerificationResultSchema = z.object({
  passed: z.boolean(),
  message: z.string(),
  file_path: z.string(),
  expected_hash: z.string(),
  actual_hash: z.string().nullable(),
  receipt: IntegrityReceiptSchema,
});

/**
 * Validates verification result from backend
 * CRITICAL: Used for security-sensitive integrity verification
 */
export function validateVerificationResult(
  rawResult: unknown
): z.infer<typeof VerificationResultSchema> {
  return VerificationResultSchema.parse(rawResult);
}

export function safeValidateVerificationResult(
  rawResult: unknown
):
  | { success: true; data: z.infer<typeof VerificationResultSchema> }
  | { success: false; error: string } {
  try {
    const validated = validateVerificationResult(rawResult);
    return { success: true, data: validated };
  } catch (error) {
    if (error instanceof z.ZodError) {
      const firstError = error.issues[0];
      return {
        success: false,
        error: `VerificationResult validation failed at ${firstError?.path.join('.')}: ${firstError?.message}`,
      };
    }
    return {
      success: false,
      error: `VerificationResult validation failed: ${String(error)}`,
    };
  }
}

// Export inferred types for new schemas
export type ValidatedAnalysisResponse = z.infer<typeof AnalysisResponseSchema>;
export type ValidatedDatasetVersion = z.infer<typeof DatasetVersionSchema>;
export type ValidatedVerificationResult = z.infer<typeof VerificationResultSchema>;
