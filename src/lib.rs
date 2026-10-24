pub const APPLICATION_NAME: &str = "restic-monitor";

pub fn run() {
    println!("{APPLICATION_NAME}");
}

#[cfg(test)]
mod tests {
    use super::APPLICATION_NAME;

    #[test]
    fn exposes_the_application_name() {
        assert_eq!(APPLICATION_NAME, "restic-monitor");
    }
}
