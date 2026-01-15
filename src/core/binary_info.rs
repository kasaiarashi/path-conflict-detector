use crate::error::Result;
use crate::output::types::ExecutableInfo;
use std::fs;
use std::io::Read;

pub struct BinaryInfoExtractor {
    compute_hashes: bool,
}

impl BinaryInfoExtractor {
    pub fn new(compute_hashes: bool) -> Self {
        BinaryInfoExtractor { compute_hashes }
    }

    pub fn enrich_executables(&self, executables: &mut [ExecutableInfo]) -> Result<()> {
        for executable in executables.iter_mut() {
            if self.compute_hashes {
                executable.file_hash = self.compute_file_hash(&executable.full_path);
            }
        }

        Ok(())
    }

    fn compute_file_hash(&self, path: &std::path::Path) -> Option<String> {
        // Simple hash computation using a basic algorithm
        // In production, you might want to use a proper hashing library like sha2
        let file = fs::File::open(path).ok()?;
        let mut buffer = Vec::new();

        // Read first 8KB for hash (to avoid reading huge files)
        let mut handle = file.take(8192);
        handle.read_to_end(&mut buffer).ok()?;

        // Simple FNV-1a hash
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in buffer {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }

        Some(format!("{:016x}", hash))
    }
}

impl Default for BinaryInfoExtractor {
    fn default() -> Self {
        Self::new(false)
    }
}
