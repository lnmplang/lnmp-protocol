use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

use crate::error::{CliError, Result};

/// Read text file from path
pub fn read_text<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    fs::read_to_string(path).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            CliError::FileNotFound(path.display().to_string())
        } else {
            CliError::Io(e)
        }
    })
}

/// Read binary file from path
pub fn read_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let path = path.as_ref();
    fs::read(path).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            CliError::FileNotFound(path.display().to_string())
        } else {
            CliError::Io(e)
        }
    })
}

/// Write text to file
pub fn write_text<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    fs::write(path, content).map_err(CliError::Io)
}

/// Write binary data to file
pub fn write_bytes<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<()> {
    fs::write(path, content).map_err(CliError::Io)
}

/// Read from stdin or file
/// If path is "-", read from stdin, otherwise read from file
pub fn read_input<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let path = path.as_ref();

    if path == Path::new("-") {
        read_stdin()
    } else {
        read_bytes(path)
    }
}

/// Read from stdin or file (text mode)
pub fn read_input_text<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();

    if path == Path::new("-") {
        read_stdin_text()
    } else {
        read_text(path)
    }
}

/// Read binary data from stdin
pub fn read_stdin() -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer).map_err(CliError::Io)?;
    Ok(buffer)
}

/// Read text from stdin
pub fn read_stdin_text() -> Result<String> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(CliError::Io)?;
    Ok(buffer)
}

/// Write to stdout or file
/// If path is "-", write to stdout, otherwise write to file
pub fn write_output<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<()> {
    let path = path.as_ref();

    if path == Path::new("-") {
        write_stdout(content)
    } else {
        write_bytes(path, content)
    }
}

/// Write text to stdout or file
pub fn write_output_text<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();

    if path == Path::new("-") {
        write_stdout_text(content)
    } else {
        write_text(path, content)
    }
}

/// Write binary data to stdout
pub fn write_stdout(content: &[u8]) -> Result<()> {
    io::stdout().write_all(content).map_err(CliError::Io)?;
    io::stdout().flush().map_err(CliError::Io)
}

/// Write text to stdout
pub fn write_stdout_text(content: &str) -> Result<()> {
    write_stdout(content.as_bytes())
}

/// Write to stderr
pub fn write_stderr(content: &str) -> Result<()> {
    io::stderr()
        .write_all(content.as_bytes())
        .map_err(CliError::Io)?;
    io::stderr().flush().map_err(CliError::Io)
}

/// Check if path exists
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Check if path is stdin marker ("-")
pub fn is_stdin<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref() == Path::new("-")
}

/// Check if path is stdout marker ("-")
pub fn is_stdout<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref() == Path::new("-")
}

/// Get file size
pub fn file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    fs::metadata(path).map(|m| m.len()).map_err(CliError::Io)
}

/// Check if file is readable
pub fn is_readable<P: AsRef<Path>>(path: P) -> bool {
    fs::File::open(path).is_ok()
}

/// Create parent directories if they don't exist
pub fn ensure_parent_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(CliError::Io)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_stdin() {
        assert!(is_stdin("-"));
        assert!(!is_stdin("file.txt"));
    }

    #[test]
    fn test_is_stdout() {
        assert!(is_stdout("-"));
        assert!(!is_stdout("file.txt"));
    }

    #[test]
    fn test_read_write_temp_file() {
        let temp_file = "/tmp/lnmp_test_io.txt";
        let content = "test content";

        write_text(temp_file, content).unwrap();
        let read_content = read_text(temp_file).unwrap();

        assert_eq!(content, read_content);

        // Cleanup
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_file_size() {
        let temp_file = "/tmp/lnmp_test_size.txt";
        let content = "12345";

        write_text(temp_file, content).unwrap();
        let size = file_size(temp_file).unwrap();

        assert_eq!(size, 5);

        // Cleanup
        let _ = fs::remove_file(temp_file);
    }
}
