//! Build script for downloading the OpenCLI specification schema.
//!
//! When the `fetch_opencli_schema` cfg flag is enabled, this script downloads
//! the latest OpenCLI spec from GitHub and saves it to `tests/assets/opencli.spec.json`.

fn main() {
    #[cfg(not(fetch_opencli_schema))]
    {
        println!(
            "cargo:debug=Config 'fetch_opencli_schema' not enabled: Skipping OpenCLI spec download"
        );
    }
    #[cfg(fetch_opencli_schema)]
    {
        /// The URL to the OpenCLI spec file (main branch)
        const SPEC_URL: &str = "https://raw.githubusercontent.com/nrranjithnr/open-cli-specification/refs/heads/main/opencli.spec.json";

        /// The path to the OpenCLI spec file
        const SPEC_PATH: &str = "tests/assets/opencli.spec.json";

        println!("cargo:warning=Config 'fetch_opencli_schema' enabled: Downloading OpenCLI spec");

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .expect("Failed to get CARGO_MANIFEST_DIR");

        let spec_path = manifest_dir.join(SPEC_PATH);

        // Create the assets directory if it doesn't exist
        if let Some(parent) = spec_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create assets directory");
        }

        // Download the spec file using reqwest
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(SPEC_URL)
            .send()
            .expect("Failed to download OpenCLI spec");

        if !response.status().is_success() {
            panic!(
                "Failed to download opencli.spec.json file: {}",
                response.status()
            );
        }

        let content = response
            .text()
            .expect("Failed to read OpenCLI spec response body");

        // Write the content to the file
        std::fs::write(&spec_path, content).expect("Failed to write opencli.spec.json to file");

        println!("cargo:warning=Downloaded latest version of opencli.spec.json from GitHub");
    }
}
