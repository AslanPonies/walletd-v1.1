#[cfg(test)]
mod canister_tests {

    use ic_agent::Agent;

    #[allow(dead_code)]
    // Helper function to create a test agent
    fn create_test_agent() -> Agent {
        // Use localhost URL for testing - won't actually connect
        // ic-agent 0.44+ uses with_url directly
        Agent::builder()
            .with_url("http://localhost:8000")
            .build()
            .expect("Failed to build agent")
    }
}
