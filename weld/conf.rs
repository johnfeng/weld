//! Configurations and defaults for the Weld runtime.

use std::ffi::CString;

use super::WeldConf;
use super::error::WeldResult;
use super::passes::OPTIMIZATION_PASSES;
use super::passes::Pass;

// Keys used in textual representation of conf
pub const MEMORY_LIMIT_KEY: &'static str = "weld.memory.limit";
pub const THREADS_KEY: &'static str = "weld.threads";
pub const OPTIMIZATION_PASSES_KEY: &'static str = "weld.optimization.passes";
pub const LLVM_OPTIMIZATION_LEVEL_KEY: &'static str = "weld.llvm.optimization.level";

// Default values of each key
pub const DEFAULT_MEMORY_LIMIT: i64 = 1000000000;
pub const DEFAULT_THREADS: i32 = 1;
pub const DEFAULT_LLVM_OPTIMIZATION_LEVEL: u32 = 2;
lazy_static! {
    pub static ref DEFAULT_OPTIMIZATION_PASSES: Vec<Pass> = {
        let m = ["inline-apply", "inline-let", "inline-zip", "loop-fusion", "infer-size", "predicate", "vectorize"];
        m.iter().map(|e| (*OPTIMIZATION_PASSES.get(e).unwrap()).clone()).collect()
    };
}

// A parsed configuration with correctly typed fields.
pub struct ParsedConf {
    pub memory_limit: i64,
    pub threads: i32,
    pub optimization_passes: Vec<Pass>,
    pub llvm_optimization_level: u32
}

/// Parse a configuration from a WeldConf key-value dictiomary.
pub fn parse(conf: &WeldConf) -> WeldResult<ParsedConf> {
    let value = get_value(conf, MEMORY_LIMIT_KEY);
    let memory_limit = value.map(|s| parse_memory_limit(&s))
                            .unwrap_or(Ok(DEFAULT_MEMORY_LIMIT))?;

    let value = get_value(conf, THREADS_KEY);
    let threads = value.map(|s| parse_threads(&s))
                       .unwrap_or(Ok(DEFAULT_THREADS))?;

    let value = get_value(conf, OPTIMIZATION_PASSES_KEY);
    let passes = value.map(|s| parse_passes(&s))
                      .unwrap_or(Ok(DEFAULT_OPTIMIZATION_PASSES.clone()))?;

    let value = get_value(conf, LLVM_OPTIMIZATION_LEVEL_KEY);
    let level = value.map(|s| parse_llvm_optimization_level(&s))
                      .unwrap_or(Ok(DEFAULT_LLVM_OPTIMIZATION_LEVEL))?;

    Ok(ParsedConf {
        memory_limit: memory_limit,
        threads: threads,
        optimization_passes: passes,
        llvm_optimization_level: level
    })
}

fn get_value(conf: &WeldConf, key: &str) -> Option<String> {
    let c_key = CString::new(key).unwrap();
    let c_value = conf.dict.get(&c_key);
    c_value.map(|cstr| String::from(cstr.to_string_lossy()))
}

/// Parse a number of threads.
fn parse_threads(s: &str) -> WeldResult<i32> {
    match s.parse::<i32>() {
        Ok(v) if v > 0 => Ok(v),
        _ => weld_err!("Invalid number of threads: {}", s),
    }
}

/// Parse a memory limit.
fn parse_memory_limit(s: &str) -> WeldResult<i64> {
    match s.parse::<i64>() {
        Ok(v) if v > 0 => Ok(v),
        _ => weld_err!("Invalid memory limit: {}", s),
    }
}

/// Parse a list of optimization passes.
fn parse_passes(s: &str) -> WeldResult<Vec<Pass>> {
    if s.len() == 0 {
        return Ok(vec![]); // Special case because split() creates an empty piece here
    }
    let mut result = vec![];
    for piece in s.split(",") {
        match OPTIMIZATION_PASSES.get(piece) {
            Some(pass) => result.push(pass.clone()),
            None => return weld_err!("Unknown optimization pass: {}", piece)
        }
    }
    Ok(result)
}

/// Parse an LLVM optimization level.
fn parse_llvm_optimization_level(s: &str) -> WeldResult<u32> {
    match s.parse::<u32>() {
        Ok(v) if v <= 3 => Ok(v),
        _ => weld_err!("Invalid LLVM optimization level: {}", s),
    }
}

#[test]
fn conf_parsing() {
    assert_eq!(parse_threads("1").unwrap(), 1);
    assert_eq!(parse_threads("2").unwrap(), 2);
    assert!(parse_threads("-1").is_err());
    assert!(parse_threads("").is_err());

    assert_eq!(parse_memory_limit("1000000").unwrap(), 1000000);
    assert!(parse_memory_limit("-1").is_err());
    assert!(parse_memory_limit("").is_err());

    assert_eq!(parse_passes("loop-fusion,inline-let").unwrap().len(), 2);
    assert_eq!(parse_passes("loop-fusion").unwrap().len(), 1);
    assert_eq!(parse_passes("").unwrap().len(), 0);
    assert!(parse_passes("non-existent-pass").is_err());

    assert_eq!(parse_llvm_optimization_level("0").unwrap(), 0);
    assert_eq!(parse_llvm_optimization_level("2").unwrap(), 2);
    assert!(parse_llvm_optimization_level("5").is_err());
}
