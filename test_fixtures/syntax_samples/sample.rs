//! Module-level documentation for the sample module.
//! This file exercises every semantic token type for Rust syntax highlighting.

use std::collections::HashMap;
use std::fmt::{self, Display};
use std::io::{self, Read, Write};

// ── Constants & Statics ──────────────────────────────────────────────

const MAX_RETRIES: u32 = 3;
const API_BASE_URL: &str = "https://api.example.com/v1";
static GLOBAL_COUNTER: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

// ── Type Definitions ─────────────────────────────────────────────────

/// A lifecycle phase for a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Pending,
    Running { progress: f64 },
    Completed,
    Failed(String),
}

/// Generic container with a lifetime parameter.
#[derive(Debug)]
pub struct Container<'a, T: Display + Clone> {
    label: &'a str,
    items: Vec<T>,
    metadata: HashMap<String, serde_json::Value>,
}

/// Trait for objects that can be serialized to bytes.
pub trait Serializable {
    type Error;
    fn serialize(&self) -> Result<Vec<u8>, Self::Error>;
    fn deserialize(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

// ── Implementations ──────────────────────────────────────────────────

impl<'a, T: Display + Clone> Container<'a, T> {
    /// Creates a new container with the given label.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            items: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Adds an item and returns a mutable reference to self for chaining.
    pub fn push(&mut self, item: T) -> &mut Self {
        self.items.push(item);
        self
    }

    /// Returns items matching the predicate.
    pub fn filter<F>(&self, predicate: F) -> Vec<&T>
    where
        F: Fn(&T) -> bool,
    {
        self.items.iter().filter(|item| predicate(item)).collect()
    }

    /// Finds the first item matching a condition.
    pub fn find_first(&self, needle: &str) -> Option<&T>
    where
        T: AsRef<str>,
    {
        self.items
            .iter()
            .find(|item| item.as_ref().contains(needle))
    }
}

impl<'a, T: Display + Clone> Display for Container<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Container({}, {} items)", self.label, self.items.len())
    }
}

// ── Macros ───────────────────────────────────────────────────────────

/// A macro that logs a message with a severity level.
macro_rules! log_msg {
    ($level:expr, $($arg:tt)*) => {
        eprintln!("[{}] {}", $level, format!($($arg)*));
    };
}

/// Declarative macro for quick HashMap construction.
macro_rules! map {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut m = HashMap::new();
        $(m.insert($key, $val);)*
        m
    }};
}

// ── Functions ────────────────────────────────────────────────────────

/// Parses a version string like "1.2.3" into components.
pub fn parse_version(input: &str) -> Result<(u32, u32, u32), String> {
    let parts: Vec<&str> = input.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Invalid version format: \"{}\"", input));
    }

    let major = parts[0]
        .parse::<u32>()
        .map_err(|e| format!("Bad major: {e}"))?;
    let minor = parts[1]
        .parse::<u32>()
        .map_err(|e| format!("Bad minor: {e}"))?;
    let patch = parts[2]
        .parse::<u32>()
        .map_err(|e| format!("Bad patch: {e}"))?;

    Ok((major, minor, patch))
}

/// Demonstrates various numeric literals.
fn numeric_showcase() {
    let _decimal = 1_000_000;
    let _hex = 0xFF_AA_00;
    let _octal = 0o777;
    let _binary = 0b1010_0101;
    let _float = 3.141_592_653;
    let _scientific = 2.998e8;
    let _negative = -42i64;
    let _byte = b'A';
}

/// Demonstrates string and escape sequences.
fn string_showcase() {
    let _simple = "hello world";
    let _escaped = "line1\nline2\ttab\\backslash";
    let _unicode = "emoji: \u{1F600} and \u{00E9}";
    let _raw = r#"no \escapes "here" at all"#;
    let _raw_hash = r##"can contain # "# inside"##;
    let _byte_string = b"byte literal";
    let _multiline = "first line\
        continuation";
    let _char = '🦀';
    let _escaped_char = '\n';
}

/// An async function showing control flow and pattern matching.
async fn process_tasks(tasks: Vec<Phase>) -> io::Result<usize> {
    let mut completed = 0usize;

    for (index, task) in tasks.iter().enumerate() {
        match task {
            Phase::Pending => {
                log_msg!("INFO", "Task {} is pending", index);
                continue;
            }
            Phase::Running { progress } if *progress > 0.9 => {
                log_msg!("WARN", "Task {} almost done: {:.1}%", index, progress * 100.0);
            }
            Phase::Running { progress } => {
                log_msg!("DEBUG", "Task {} at {:.1}%", index, progress * 100.0);
            }
            Phase::Completed => {
                completed += 1;
            }
            Phase::Failed(reason) => {
                log_msg!("ERROR", "Task {} failed: {}", index, reason);
                if reason.contains("fatal") {
                    return Err(io::Error::new(io::ErrorKind::Other, reason.clone()));
                }
            }
        }
    }

    Ok(completed)
}

// ── Closures & Iterators ─────────────────────────────────────────────

fn iterator_showcase() {
    let numbers: Vec<i32> = (1..=20).collect();

    // Chained iterator operations
    let result: Vec<String> = numbers
        .iter()
        .filter(|&&n| n % 2 == 0)
        .map(|n| n * n)
        .take_while(|&n| n < 200)
        .enumerate()
        .map(|(i, val)| format!("[{i}] = {val}"))
        .collect();

    // Fold / reduce
    let sum: i64 = (1..=100).fold(0i64, |acc, x| acc + x);

    let _lookup = map! {
        "alpha" => 1,
        "beta"  => 2,
        "gamma" => 3,
    };

    println!("Sum: {sum}, Results: {result:?}");
}

// ── Trait Objects & Dynamic Dispatch ─────────────────────────────────

fn display_all(items: &[&dyn Display]) {
    for item in items {
        println!("{item}");
    }
}

// ── Unsafe & Raw Pointers ────────────────────────────────────────────

/// Demonstrates unsafe blocks and raw pointer manipulation.
unsafe fn raw_pointer_demo(ptr: *mut u8, len: usize) -> &'static [u8] {
    assert!(!ptr.is_null(), "Null pointer passed to raw_pointer_demo");
    std::slice::from_raw_parts(ptr, len)
}

// ── Generics with Complex Bounds ─────────────────────────────────────

fn complex_generics<'a, T, U>(x: &'a T, y: &'a U) -> String
where
    T: Display + Clone + Send + 'a,
    U: Into<String> + Default,
{
    format!("{x}")
}

// ── Entry Point ──────────────────────────────────────────────────────

fn main() {
    // Boolean & Option/Result
    let enabled: bool = true;
    let maybe_value: Option<i32> = Some(42);
    let result: Result<&str, &str> = Ok("success");

    // Destructuring
    let (a, b, c) = (1, "two", 3.0);
    let [first, .., last] = [10, 20, 30, 40, 50];

    // If let / while let
    if let Some(val) = maybe_value {
        println!("Got value: {val}");
    }

    // Range and loop
    for i in 0..5 {
        if i == 3 {
            break;
        }
    }

    // Box, Rc, Arc
    let _boxed: Box<dyn Display> = Box::new("heap string");
    let _rc = std::rc::Rc::new(vec![1, 2, 3]);

    // Container usage
    let mut container = Container::new("demo");
    container.push("first".to_string()).push("second".to_string());
    println!("{container}");

    numeric_showcase();
    string_showcase();
    iterator_showcase();
}
