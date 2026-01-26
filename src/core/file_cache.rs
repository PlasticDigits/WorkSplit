use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::debug;

/// Entry in the file cache
pub struct CacheEntry {
    /// File content
    pub content: String,
    /// File modification time when cached
    pub modified: SystemTime,
    /// Line count
    pub line_count: usize,
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub entry_count: usize,
    pub total_bytes: usize,
    pub total_lines: usize,
}

/// Cache for file contents to avoid re-reading unchanged files
pub struct FileCache {
    /// Cached file entries keyed by absolute path
    entries: HashMap<PathBuf, CacheEntry>,
}

impl FileCache {
    /// Create empty cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Get cached entry if present (without loading)
    pub fn get(&self, path: &Path) -> Option<&CacheEntry> {
        self.entries.get(path)
    }

    /// Get or load a file entry
    pub fn get_or_load(&mut self, path: &Path) -> Result<&CacheEntry, std::io::Error> {
        let canonical_path = path.canonicalize()?;
        
        // Check if we need to reload (cache miss or file modified)
        let needs_reload = match self.entries.get(&canonical_path) {
            Some(entry) => {
                let metadata = std::fs::metadata(&canonical_path)?;
                let current_modified = metadata.modified()?;
                if current_modified != entry.modified {
                    debug!("Cache miss for file (modified): {}", canonical_path.display());
                    true
                } else {
                    debug!("Cache hit for file: {}", canonical_path.display());
                    false
                }
            }
            None => {
                debug!("Cache miss for file (not cached): {}", canonical_path.display());
                true
            }
        };

        if needs_reload {
            // File is not cached or has been modified, read it
            let content = std::fs::read_to_string(&canonical_path)?;
            let line_count = content.lines().count();
            let metadata = std::fs::metadata(&canonical_path)?;
            let modified = metadata.modified()?;

            let entry = CacheEntry {
                content,
                modified,
                line_count,
            };

            self.entries.insert(canonical_path.clone(), entry);
        }
        
        // Return reference to the entry (guaranteed to exist now)
        Ok(self.entries.get(&canonical_path).unwrap())
    }

    /// Invalidate a specific path from cache
    pub fn invalidate(&mut self, path: &Path) {
        let canonical_path = path.canonicalize().ok();
        if let Some(canonical_path) = canonical_path {
            self.entries.remove(&canonical_path);
        }
    }

    /// Clear entire cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = CacheStats::default();
        for entry in self.entries.values() {
            stats.entry_count += 1;
            stats.total_bytes += entry.content.len();
            stats.total_lines += entry.line_count;
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_cache_new() {
        let cache = FileCache::new();
        assert_eq!(cache.stats().entry_count, 0);
    }

    #[test]
    fn test_file_cache_get_or_load() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let mut cache = FileCache::new();
        let entry = cache.get_or_load(&file_path).unwrap();
        
        assert_eq!(entry.content, "Hello, world!\n");
        assert_eq!(entry.line_count, 1);
    }

    #[test]
    fn test_file_cache_get() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let mut cache = FileCache::new();
        let _entry = cache.get_or_load(&file_path).unwrap();
        
        let entry = cache.get(&file_path);
        assert!(entry.is_some());
    }

    #[test]
    fn test_file_cache_invalidate() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let mut cache = FileCache::new();
        let _entry = cache.get_or_load(&file_path).unwrap();
        
        assert_eq!(cache.stats().entry_count, 1);
        
        cache.invalidate(&file_path);
        assert_eq!(cache.stats().entry_count, 0);
    }

    #[test]
    fn test_file_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let mut cache = FileCache::new();
        let _entry = cache.get_or_load(&file_path).unwrap();
        
        assert_eq!(cache.stats().entry_count, 1);
        
        cache.clear();
        assert_eq!(cache.stats().entry_count, 0);
    }

    #[test]
    fn test_file_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let mut cache = FileCache::new();
        let _entry = cache.get_or_load(&file_path).unwrap();
        
        let stats = cache.stats();
        assert_eq!(stats.entry_count, 1);
        assert_eq!(stats.total_lines, 1);
    }
}