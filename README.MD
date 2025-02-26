# FoodScraper

FoodScraper is a Rust-based tool that scrapes food recipes from a given URL and converts them into a JSON file.
> **Note:** FoodScraper only supports URLs from `15gram.be` at this time but will be extended to support more sites in the future.

## Features

- Fetches HTML content from a user-provided URL.
- Extracts recipe details such as title, description, ingredients, steps, and image link.
- Saves the extracted recipe details into a JSON file.

## Dependencies

- `reqwest` for making HTTP requests.
- `scraper` for parsing HTML content.
- `tokio` for asynchronous runtime.
- `serde` and `serde_json` for serializing data to JSON.
- `url` for URL parsing and validation.

## Usage

1. Clone the repository.
2. Navigate to the project directory.
3. Run the program using Cargo:
   ```sh
   cargo run