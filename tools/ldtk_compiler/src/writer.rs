use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::output_schema::OutputRoot;

/// Writes the output JSON atomically: serialise → write to `.json.tmp` → rename.
///
/// The rename step is atomic on all major operating systems, which means the
/// final file is never left in a partially-written state if the process is
/// interrupted during the write.
///
/// # Errors
/// Returns an error if serialisation, the intermediate write, or the rename fails.
pub fn write_atomic(output: &OutputRoot, path: &Path) -> Result<()> {
    // Ensure parent directory exists so the write doesn't fail with a
    // confusing "No such file or directory" error.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(output)?;

    // Use a sibling temporary file so the rename stays on the same filesystem
    // partition (cross-device renames are not atomic on most platforms).
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &json)?;
    fs::rename(&tmp_path, path)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_schema::OutputRoot;

    // ------------------------------------------------------------------
    // write_atomic_creates_file
    // ------------------------------------------------------------------

    /// write_atomic must:
    ///   1. Create the output file at the requested path.
    ///   2. Leave valid UTF-8 JSON inside it.
    ///   3. Remove the `.json.tmp` temporary file after the rename.
    #[test]
    fn write_atomic_creates_file() {
        let dir = tempfile::tempdir().expect("could not create temp dir");
        let out_path = dir.path().join("output.json");
        let tmp_path = dir.path().join("output.json.tmp");

        let root = OutputRoot::from_converted(vec![]);
        write_atomic(&root, &out_path).expect("write_atomic must succeed");

        // The final file must exist.
        assert!(out_path.exists(), "output file must exist after write_atomic");

        // The .tmp file must have been removed by the rename.
        assert!(
            !tmp_path.exists(),
            ".json.tmp must not exist after write_atomic completes"
        );

        // The file must contain valid JSON.
        let contents = fs::read_to_string(&out_path).expect("must be able to read output file");
        let value: serde_json::Value =
            serde_json::from_str(&contents).expect("file contents must be valid JSON");

        // Basic sanity: schema_version must be present and equal to 1.
        assert_eq!(value["schema_version"], 1u32);
        assert!(value["levels"].is_array());
    }
}
