//! A simple module for generating personalized greetings.

/// Returns a simple greeting.
///
/// # Arguments
///
/// * `name` - The name to greet
///
/// # Returns
///
/// A formatted greeting string.
///
/// # Examples
///
/// ```
/// use greeting::greet;
/// assert_eq!(greet("World"), "Hello, World!");
/// ```
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Returns a formal greeting.
///
/// # Arguments
///
/// * `name` - The name to greet
///
/// # Returns
///
/// A formal greeting string.
///
/// # Examples
///
/// ```
/// use greeting::greet_formal;
/// assert_eq!(greet_formal("Dr. Smith"), "Good day, Dr. Smith. How may I assist you?");
/// ```
pub fn greet_formal(name: &str) -> String {
    format!("Good day, {}. How may I assist you?", name)
}

/// Returns a casual greeting.
///
/// # Arguments
///
/// * `name` - The name to greet
///
/// # Returns
///
/// A casual greeting string.
///
/// # Examples
///
/// ```
/// use greeting::greet_casual;
/// assert_eq!(greet_casual("Alice"), "Hey Alice! What's up?");
/// ```
pub fn greet_casual(name: &str) -> String {
    format!("Hey {}! What's up?", name)
}

/// Returns a time-appropriate greeting.
///
/// # Arguments
///
/// * `name` - The name to greet
/// * `hour` - The hour of the day (0-23)
///
/// # Returns
///
/// A time-appropriate greeting string.
///
/// # Examples
///
/// ```
/// use greeting::greet_with_time;
/// assert_eq!(greet_with_time("Alice", 9), "Good morning, Alice!");
/// assert_eq!(greet_with_time("Bob", 15), "Good afternoon, Bob!");
/// assert_eq!(greet_with_time("Charlie", 20), "Good evening, Charlie!");
/// assert_eq!(greet_with_time("Dave", 23), "Good night, Dave!");
/// ```
pub fn greet_with_time(name: &str, hour: u8) -> String {
    let hour = hour.clamp(0, 23);
    
    match hour {
        5..=11 => format!("Good morning, {}!", name),
        12..=17 => format!("Good afternoon, {}!", name),
        18..=21 => format!("Good evening, {}!", name),
        _ => format!("Good night, {}!", name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
        assert_eq!(greet("Alice"), "Hello, Alice!");
    }

    #[test]
    fn test_greet_formal() {
        assert_eq!(
            greet_formal("Dr. Smith"),
            "Good day, Dr. Smith. How may I assist you?"
        );
        assert_eq!(greet_formal("Alice"), "Good day, Alice. How may I assist you?");
    }

    #[test]
    fn test_greet_casual() {
        assert_eq!(greet_casual("Alice"), "Hey Alice! What's up?");
        assert_eq!(greet_casual("Bob"), "Hey Bob! What's up?");
    }

    #[test]
    fn test_greet_with_time_morning() {
        assert_eq!(greet_with_time("Alice", 9), "Good morning, Alice!");
        assert_eq!(greet_with_time("Bob", 11), "Good morning, Bob!");
    }

    #[test]
    fn test_greet_with_time_afternoon() {
        assert_eq!(greet_with_time("Alice", 12), "Good afternoon, Alice!");
        assert_eq!(greet_with_time("Bob", 15), "Good afternoon, Bob!");
        assert_eq!(greet_with_time("Charlie", 17), "Good afternoon, Charlie!");
    }

    #[test]
    fn test_greet_with_time_evening() {
        assert_eq!(greet_with_time("Alice", 18), "Good evening, Alice!");
        assert_eq!(greet_with_time("Bob", 20), "Good evening, Bob!");
        assert_eq!(greet_with_time("Charlie", 21), "Good evening, Charlie!");
    }

    #[test]
    fn test_greet_with_time_night() {
        assert_eq!(greet_with_time("Alice", 22), "Good night, Alice!");
        assert_eq!(greet_with_time("Bob", 23), "Good night, Bob!");
        assert_eq!(greet_with_time("Charlie", 0), "Good night, Charlie!");
        assert_eq!(greet_with_time("Dave", 4), "Good night, Dave!");
    }

    #[test]
    fn test_greet_with_time_clamping() {
        assert_eq!(greet_with_time("Alice", 30), "Good night, Alice!"); // clamped to 23
        assert_eq!(greet_with_time("Bob", 255), "Good night, Bob!"); // clamped to 23
        assert_eq!(greet_with_time("Charlie", 3), "Good night, Charlie!"); // clamped to 3
    }
}