/**
 * Mirrors the Rust serde DTOs from crates/pass-tauri/src/commands/read.rs
 * Field names match serde's default snake_case serialization.
 */

/** A node in the entry tree (directory or leaf). Leaf nodes have a non-null path. */
export interface EntryNode {
  name: string;
  /** Full slash-separated path for leaf entries; null for directories. */
  path: string | null;
  children: EntryNode[];
}

/** Metadata for a single password entry — mirrors `EntryMeta` Rust struct. */
export interface EntryMeta {
  path: string;
  /** Key/value fields from the entry text (password and OTP URI excluded). */
  fields: [string, string][];
  tags: string[];
  has_otp: boolean;
}

/** A live OTP code with remaining seconds — mirrors `OtpCode` Rust struct. */
export interface OtpCode {
  code: string;
  seconds: number;
}

/**
 * Input shape for creating a new password entry — mirrors `EntryInput` Rust struct.
 * Used with the `insert` command.
 */
export interface EntryInput {
  password: string;
  fields: [string, string][];
  otp: string | null;
  tags: string[];
}

/**
 * Input shape for updating an existing password entry — mirrors `UpdateInput` Rust struct.
 * Used with the `update_entry` command. Same shape as `EntryInput`.
 */
export interface UpdateInput {
  password: string;
  fields: [string, string][];
  otp: string | null;
  tags: string[];
}

/** Environment diagnostics — mirrors `DoctorReport` Rust struct. */
export interface DoctorReport {
  pass: boolean;
  gpg: boolean;
  store_dir_exists: boolean;
  store_dir: string;
  ok: boolean;
  guidance: string;
  demo: boolean;
  init_error: string | null;
}

/**
 * Local git state of the store — mirrors `passcore::git::Status`.
 * Read from local refs only: `behind` is as fresh as the last pull, never live.
 */
export interface GitStatus {
  branch: string;
  ahead: number;
  behind: number;
  dirty: number;
  upstream: boolean;
}

/** A network git operation — mirrors `passcore::git::Op`. */
export type GitOp = 'pull' | 'push';
