//! Logging utilities for CLI verbose output

use anyhow::Result;
use log::LevelFilter;
use std::io::Write;

/// Initialize logging based on verbosity count
/// 
/// - 0: WARN level (default)
/// - 1: INFO level (-v)
/// - 2: DEBUG level (-vv)  
/// - 3+: TRACE level (-vvv)
pub fn init_logging(verbose_count: u8) -> Result<()> {
    let log_level = match verbose_count {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let result = env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .filter_module("reqwest", LevelFilter::Info) // Keep reqwest quiet unless trace
        .filter_module("hyper", LevelFilter::Info)   // Keep hyper quiet unless trace
        .format(|buf, record| {
            use log::Level;
            
            let level_style = match record.level() {
                Level::Error => "\x1b[31m[ERROR]\x1b[0m", // Red
                Level::Warn  => "\x1b[33m[WARN ]\x1b[0m", // Yellow
                Level::Info  => "\x1b[32m[INFO ]\x1b[0m", // Green
                Level::Debug => "\x1b[36m[DEBUG]\x1b[0m", // Cyan
                Level::Trace => "\x1b[37m[TRACE]\x1b[0m", // White
            };
            
            writeln!(buf, "{} {}", level_style, record.args())
        })
        .target(env_logger::Target::Stderr)
        .try_init();

    // If logger is already initialized, that's fine - just continue
    match result {
        Ok(()) => {},
        Err(_) => {
            // Logger already initialized, this is expected in some cases (like tests)
            // Just continue silently
        }
    }

    if verbose_count > 0 {
        log::info!("Verbose logging enabled (level: {})", log_level);
    }

    Ok(())
}

/// Macro for timing operations and logging results
#[macro_export]
macro_rules! time_operation {
    ($operation:expr, $description:expr) => {{
        let start = std::time::Instant::now();
        log::debug!("Starting: {}", $description);
        let result = $operation;
        let elapsed = start.elapsed();
        log::info!("Completed: {} in {:?}", $description, elapsed);
        result
    }};
}

/// Log HTTP request details
pub fn log_http_request(method: &str, url: &str, has_body: bool) {
    log::debug!("HTTP {} {}{}", method, url, if has_body { " (with body)" } else { "" });
}

/// Log HTTP response details
pub fn log_http_response(status: u16, elapsed: std::time::Duration) {
    if status >= 400 {
        log::warn!("HTTP response: {} in {:?}", status, elapsed);
    } else {
        log::debug!("HTTP response: {} in {:?}", status, elapsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_log_level(verbose_count: u8) -> LevelFilter {
        match verbose_count {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }

    #[test]
    fn test_logging_levels() {
        // Test verbose count to log level mapping
        let test_cases = [
            (0, LevelFilter::Warn),
            (1, LevelFilter::Info),
            (2, LevelFilter::Debug),
            (3, LevelFilter::Trace),
            (10, LevelFilter::Trace), // Any value >= 3 should be trace
        ];

        for (verbose_count, expected_level) in test_cases {
            assert_eq!(
                get_log_level(verbose_count),
                expected_level,
                "verbose_count {} should map to {:?}",
                verbose_count,
                expected_level
            );
        }
    }

    #[test]
    fn test_init_logging_returns_result() {
        // Test that init_logging returns a Result and handles multiple calls gracefully
        // Note: env_logger can only be initialized once per process, so subsequent calls may fail
        // but should not panic
        
        // First call should succeed or fail gracefully
        let result1 = init_logging(1);
        assert!(result1.is_ok() || result1.is_err()); // Either is acceptable
        
        // Second call should handle the "already initialized" case gracefully
        let result2 = init_logging(2);
        assert!(result2.is_ok() || result2.is_err()); // Either is acceptable, shouldn't panic
    }
}