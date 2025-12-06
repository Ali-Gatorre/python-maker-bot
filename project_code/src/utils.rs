use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Impossible de créer le dossier {:?}", path))?;
    }
    Ok(())
}

/// Extract Python code from a response that might contain markdown code blocks
pub fn extract_python_code(response: &str) -> String {
    // Try to match markdown code blocks with optional language identifier
    let code_block_re = Regex::new(r"```(?:python)?\s*\n([\s\S]*?)\n```").unwrap();
    
    if let Some(captures) = code_block_re.captures(response) {
        if let Some(code) = captures.get(1) {
            return code.as_str().trim().to_string();
        }
    }
    
    // If no markdown block found, return trimmed response as-is
    response.trim().to_string()
}

/// Extract all import statements from Python code
/// Returns a list of package names (without submodules)
pub fn extract_imports(code: &str) -> Vec<String> {
    let mut imports = Vec::new();
    
    // Match "import package" or "import package.submodule"
    let import_re = Regex::new(r"^import\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    
    // Match "from package import ..."
    let from_import_re = Regex::new(r"^from\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+import").unwrap();
    
    for line in code.lines() {
        let trimmed = line.trim();
        
        if let Some(caps) = import_re.captures(trimmed) {
            if let Some(pkg) = caps.get(1) {
                imports.push(pkg.as_str().to_string());
            }
        }
        
        if let Some(caps) = from_import_re.captures(trimmed) {
            if let Some(pkg) = caps.get(1) {
                imports.push(pkg.as_str().to_string());
            }
        }
    }
    
    // Remove duplicates
    imports.sort();
    imports.dedup();
    imports
}

/// Check if a package is in Python's standard library
pub fn is_stdlib(package: &str) -> bool {
    // Common Python 3 standard library modules
    const STDLIB_MODULES: &[&str] = &[
        "abc", "aifc", "argparse", "array", "ast", "asynchat", "asyncio", "asyncore",
        "atexit", "audioop", "base64", "bdb", "binascii", "binhex", "bisect", "builtins",
        "bz2", "calendar", "cgi", "cgitb", "chunk", "cmath", "cmd", "code", "codecs",
        "codeop", "collections", "colorsys", "compileall", "concurrent", "configparser",
        "contextlib", "contextvars", "copy", "copyreg", "crypt", "csv", "ctypes", "curses",
        "dataclasses", "datetime", "dbm", "decimal", "difflib", "dis", "distutils", "doctest",
        "email", "encodings", "enum", "errno", "faulthandler", "fcntl", "filecmp", "fileinput",
        "fnmatch", "fractions", "ftplib", "functools", "gc", "getopt", "getpass", "gettext",
        "glob", "graphlib", "grp", "gzip", "hashlib", "heapq", "hmac", "html", "http", "idlelib",
        "imaplib", "imghdr", "imp", "importlib", "inspect", "io", "ipaddress", "itertools",
        "json", "keyword", "lib2to3", "linecache", "locale", "logging", "lzma", "mailbox",
        "mailcap", "marshal", "math", "mimetypes", "mmap", "modulefinder", "msilib", "msvcrt",
        "multiprocessing", "netrc", "nis", "nntplib", "numbers", "operator", "optparse", "os",
        "ossaudiodev", "parser", "pathlib", "pdb", "pickle", "pickletools", "pipes", "pkgutil",
        "platform", "plistlib", "poplib", "posix", "posixpath", "pprint", "profile", "pstats",
        "pty", "pwd", "py_compile", "pyclbr", "pydoc", "queue", "quopri", "random", "re",
        "readline", "reprlib", "resource", "rlcompleter", "runpy", "sched", "secrets", "select",
        "selectors", "shelve", "shlex", "shutil", "signal", "site", "smtpd", "smtplib", "sndhdr",
        "socket", "socketserver", "spwd", "sqlite3", "ssl", "stat", "statistics", "string",
        "stringprep", "struct", "subprocess", "sunau", "symbol", "symtable", "sys", "sysconfig",
        "syslog", "tabnanny", "tarfile", "telnetlib", "tempfile", "termios", "test", "textwrap",
        "threading", "time", "timeit", "tkinter", "token", "tokenize", "tomllib", "trace",
        "traceback", "tracemalloc", "tty", "turtle", "turtledemo", "types", "typing", "unicodedata",
        "unittest", "urllib", "uu", "uuid", "venv", "warnings", "wave", "weakref", "webbrowser",
        "winreg", "winsound", "wsgiref", "xdrlib", "xml", "xmlrpc", "zipapp", "zipfile", "zipimport",
        "zlib", "_thread",
    ];
    
    STDLIB_MODULES.contains(&package)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_python_code_with_markdown() {
        let input = "```python\nprint('hello')\n```";
        let result = extract_python_code(input);
        assert_eq!(result, "print('hello')");
    }

    #[test]
    fn test_extract_python_code_without_language() {
        let input = "```\nprint('hello')\n```";
        let result = extract_python_code(input);
        assert_eq!(result, "print('hello')");
    }

    #[test]
    fn test_extract_python_code_plain_text() {
        let input = "print('hello')";
        let result = extract_python_code(input);
        assert_eq!(result, "print('hello')");
    }

    #[test]
    fn test_extract_python_code_multiline() {
        let input = "```python\ndef hello():\n    print('world')\n\nhello()\n```";
        let result = extract_python_code(input);
        assert_eq!(result, "def hello():\n    print('world')\n\nhello()");
    }

    #[test]
    fn test_extract_imports_simple() {
        let code = "import os\nimport sys";
        let result = extract_imports(code);
        assert_eq!(result, vec!["os", "sys"]);
    }

    #[test]
    fn test_extract_imports_from() {
        let code = "from pathlib import Path\nfrom os import path";
        let result = extract_imports(code);
        assert_eq!(result, vec!["os", "pathlib"]);
    }

    #[test]
    fn test_extract_imports_mixed() {
        let code = "import numpy\nfrom pandas import DataFrame\nimport requests";
        let result = extract_imports(code);
        assert_eq!(result, vec!["numpy", "pandas", "requests"]);
    }

    #[test]
    fn test_extract_imports_duplicates() {
        let code = "import os\nfrom os import path\nimport os";
        let result = extract_imports(code);
        assert_eq!(result, vec!["os"]);
    }

    #[test]
    fn test_extract_imports_with_comments() {
        let code = "# import fake\nimport real\n# from fake import test";
        let result = extract_imports(code);
        assert_eq!(result, vec!["real"]);
    }

    #[test]
    fn test_is_stdlib_standard_modules() {
        assert!(is_stdlib("os"));
        assert!(is_stdlib("sys"));
        assert!(is_stdlib("json"));
        assert!(is_stdlib("datetime"));
        assert!(is_stdlib("pathlib"));
    }

    #[test]
    fn test_is_stdlib_third_party() {
        assert!(!is_stdlib("numpy"));
        assert!(!is_stdlib("pandas"));
        assert!(!is_stdlib("requests"));
        assert!(!is_stdlib("flask"));
        assert!(!is_stdlib("django"));
    }

    #[test]
    fn test_ensure_dir_creates_new() {
        use std::path::PathBuf;
        let temp_dir = PathBuf::from("test_temp_dir_unique_12345");
        
        // Clean up if exists
        let _ = fs::remove_dir_all(&temp_dir);
        
        // Test creation
        let result = ensure_dir(&temp_dir);
        assert!(result.is_ok());
        assert!(temp_dir.exists());
        
        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ensure_dir_existing() {
        use std::path::PathBuf;
        let temp_dir = PathBuf::from("test_temp_dir_existing_12345");
        
        // Create directory first
        let _ = fs::create_dir_all(&temp_dir);
        
        // Test with existing directory
        let result = ensure_dir(&temp_dir);
        assert!(result.is_ok());
        assert!(temp_dir.exists());
        
        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
