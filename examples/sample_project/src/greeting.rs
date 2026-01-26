/// A module for generating personalized greetings
pub mod greeting {
    /// Returns a simple greeting: "Hello, {name}!"
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name to greet
    ///
    /// # Examples
    ///
    /// ```
    /// use greeting::greet;
    /// let message = greet("World");
    /// assert_eq!(message, "Hello, World!");
    /// ```
    pub fn greet(name: &str) -> String {
        format!("Hello, {}!", name)
    }

    /// Returns a formal greeting: "Good day, {name}. How may I assist you?"
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name to greet
    ///
    /// # Examples
    ///
    /// ```
    /// use greeting::greet_formal;
    /// let message = greet_formal("Dr. Smith");
    /// assert_eq!(message, "Good day, Dr. Smith. How may I assist you?");
    /// ```
    pub fn greet_formal(name: &str) -> String {
        format!("Good day, {}. How may I assist you?", name)
    }

    /// Returns a casual greeting: "Hey {name}! What's up?"
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name to greet
    ///
    /// # Examples
    ///
    /// ```
    /// use greeting::greet_casual;
    /// let message = greet_casual("Alice");
    /// assert_eq!(message, "Hey Alice! What's up?");
    /// ```
    pub fn greet_casual(name: &str) -> String {
        format!("Hey {}! What's up?", name)
    }

    /// Returns a time-appropriate greeting based on the hour
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name to greet
    /// * `hour` - The hour of the day (0-23)
    ///
    /// # Examples
    ///
    /// ```
    /// use greeting::greet_with_time;
    /// let morning = greet_with_time("Alice", 9);
    /// assert_eq!(morning, "Good morning, Alice!");
    ///
    /// let evening = greet_with_time("Bob", 19);
    /// assert_eq!(evening, "Good evening, Bob!");
    /// ```
    pub fn greet_with_time(name: &str, hour: u8) -> String {
        let hour = hour % 24;
        match hour {
            5..=11 => format!("Good morning, {}!", name),
            12..=17 => format!("Good afternoon, {}!", name),
            18..=21 => format!("Good evening, {}!", name),
            _ => format!("Good night, {}!", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greeting::greet("World"), "Hello, World!");
        assert_eq!(greeting::greet("Alice"), "Hello, Alice!");
    }

    #[test]
    fn test_greet_formal() {
        assert_eq!(
            greeting::greet_formal("Dr. Smith"),
            "Good day, Dr. Smith. How may I assist you?"
        );
        assert_eq!(
            greeting::greet_formal("John"),
            "Good day, John. How may I assist you?"
        );
    }

    #[test]
    fn test_greet_casual() {
        assert_eq!(greeting::greet_casual("Alice"), "Hey Alice! What's up?");
        assert_eq!(greeting::greet_casual("Bob"), "Hey Bob! What's up?");
    }

    #[test]
    fn test_greet_with_time() {
        // Morning hours (5-11)
        assert_eq!(greeting::greet_with_time("Alice", 5), "Good morning, Alice!");
        assert_eq!(greeting::greet_with_time("Alice", 11), "Good morning, Alice!");

        // Afternoon hours (12-17)
        assert_eq!(greeting::greet_with_time("Bob", 12), "Good afternoon, Bob!");
        assert_eq!(greeting::greet_with_time("Bob", 17), "Good afternoon, Bob!");

        // Evening hours (18-21)
        assert_eq!(greeting::greet_with_time("Charlie", 18), "Good evening, Charlie!");
        assert_eq!(greeting::greet_with_time("Charlie", 21), "Good evening, Charlie!");

        // Night hours (22-4)
        assert_eq!(greeting::greet_with_time("David", 22), "Good night, David!");
        assert_eq!(greeting::greet_with_time("David", 23), "Good night, David!");
        assert_eq!(greeting::greet_with_time("David", 0), "Good night, David!");
        assert_eq!(greeting::greet_with_time("David", 4), "Good night, David!");

        // Test hour wrapping
        assert_eq!(greeting::greet_with_time("Eve", 25), "Good morning, Eve!");
        assert_eq!(greeting::greet_with_time("Eve", 30), "Good night, Eve!");
    }
}