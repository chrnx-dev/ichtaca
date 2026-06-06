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
