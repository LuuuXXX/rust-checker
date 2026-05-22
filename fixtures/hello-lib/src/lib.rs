//! A simple greeting and arithmetic library used as a rust-checker CI gate fixture.

/// Returns a greeting string for the given name.
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Adds two integers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
        assert_eq!(greet("Rust"), "Hello, Rust!");
    }

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }
}
