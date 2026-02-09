export interface VerificationResult {
  passed: boolean;
  message: string;
  file_path: string;
  expected_hash: string;
  actual_hash: string | null;
  receipt: IntegrityReceipt;
}

export interface IntegrityReceipt {
  receipt_version: number;
  created_utc: string;
  producer: {
    app_name: string;
    app_version: string;
    platform: string;
  };
  export: {
    filename: string;
    format: string;
    file_size_bytes: number;
    row_count: number;
    column_count: number;
    schema: Array<{ name: string; dtype: string }>;
  };
  integrity: {
    hash_algorithm: string;
    hash: string;
  };
}
