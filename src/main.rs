fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        // main() prints to stdout, difficult to test directly without capturing
        // just check it exists for now
    }
}
