use clap::Parser;
use reqwest;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use toml::Value;
use url::Url;

/// Command-line arguments for the FoodScraper application.
#[derive(Parser, Debug)]
#[command(version = "1.0", author = "Maxime Beretvas", about = "Scrapes recipes from supported websites")]
struct Args {
    /// The URL of the recipe to scrape.
    #[arg(short, long)]
    url: String,

    /// The output folder to save the recipe JSON. Defaults to the script's directory.
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Serialize)]
struct Recipe {
    /// The title of the recipe.
    title: Option<String>,
    /// A brief description of the recipe.
    description: Option<String>,
    /// A list of ingredients required for the recipe.
    ingredients: Option<Vec<String>>,
    /// A list of steps to prepare the recipe.
    steps: Option<Vec<String>>,
    /// A link to an image of the prepared recipe.
    image_link: Option<String>,
    /// The URL source of the recipe.
    source_url: String,
}

#[derive(Debug)]
struct RecipeCssSelectors {
    /// The CSS selector for the recipe title.
    title: String,
    /// The CSS selector for the recipe description.
    description: String,
    /// The CSS selector for the recipe ingredients.
    ingredients: String,
    /// The CSS selector for the recipe steps.
    steps: String,
    /// The CSS selector for the recipe image.
    image: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    let input_url = &args.url;
    let output_folder = args.output.unwrap_or_else(|| {
        env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|p| p.to_str().unwrap_or(".").to_string()))
            .unwrap_or_else(|| ".".to_string())
    });

    // Validate the URL
    if !validate_supported_url(input_url) {
        return Err("Invalid URL or unsupported domain.".into());
    }

    let document = fetch_html_document(input_url).await?;
    let website_name = parse_website_name(input_url).ok_or("Failed to parse website name from URL")?;
    let selectors = load_selectors("selectors.toml", &website_name)?;

    let recipe = extract_recipe(&document, &selectors, input_url);
    save_recipe_to_file(&recipe, &output_folder)?;

    println!("Recipe scraping completed successfully.");
    Ok(())
}

/// Validates if the URL belongs to a supported domain.
fn validate_supported_url(input_url: &str) -> bool {
    Url::parse(input_url).is_ok()
        && (input_url.contains("https://15gram.be/") || input_url.contains("https://dagelijksekost.vrt.be/"))
}

/// Fetches the HTML document from the given URL.
async fn fetch_html_document(url: &str) -> Result<Html, Box<dyn std::error::Error>> {
    let html_body = reqwest::get(url).await?.text().await?;
    Ok(Html::parse_document(&html_body))
}

/// Extracts the recipe details from the HTML document using the provided selectors.
fn extract_recipe(document: &Html, selectors: &RecipeCssSelectors, source_url: &str) -> Recipe {
    Recipe {
        title: get_recipe_title(document, &selectors.title, false),
        description: get_recipe_description(document, &selectors.description, false),
        ingredients: get_recipe_ingredients(document, &selectors.ingredients, false),
        steps: get_recipe_steps(document, &selectors.steps, false),
        image_link: get_recipe_image(document, &selectors.image, false),
        source_url: source_url.to_string(),
    }
}

/// Saves the recipe to a JSON file in the specified output folder.
fn save_recipe_to_file(recipe: &Recipe, output_folder: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = match &recipe.title {
        Some(title) => format!("recipe_{}.json", title),
        None => "recipe.json".to_string(),
    };

    let json = serde_json::to_string_pretty(recipe)?;
    std::fs::create_dir_all(output_folder)?;
    let file_path = format!("{}/{}", output_folder, file_name);
    let mut file = File::create(&file_path)?;
    file.write_all(json.as_bytes())?;

    println!("Recipe JSON file '{}' created successfully in '{}'.", file_name, output_folder);
    Ok(())
}

fn load_selectors(file_path: &str, website: &str) -> Result<RecipeCssSelectors, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let value: Value = toml::from_str(&content)?;

    let website_selectors = value.get(website).ok_or("Website not found in selectors file")?;
    Ok(RecipeCssSelectors {
        title: website_selectors.get("title").and_then(Value::as_str).unwrap_or_default().to_string(),
        description: website_selectors.get("description").and_then(Value::as_str).unwrap_or_default().to_string(),
        ingredients: website_selectors.get("ingredients").and_then(Value::as_str).unwrap_or_default().to_string(),
        steps: website_selectors.get("steps").and_then(Value::as_str).unwrap_or_default().to_string(),
        image: website_selectors.get("image").and_then(Value::as_str).unwrap_or_default().to_string(),
    })
}

fn validate_url(input_url: &str) -> Result<Url, String> {
    let trimmed_url = input_url.trim();
    match Url::parse(trimmed_url) {
        Ok(url) => {
            if url.scheme() == "http" || url.scheme() == "https" {
                if url.host().is_some() {
                    Ok(url)
                } else {
                    Err("URL must contain a valid host.".to_string())
                }
            } else {
                Err("URL must use http or https scheme.".to_string())
            }
        }
        Err(_) => Err("Invalid URL format.".to_string()),
    }
}

fn select_elements<'a>(document: &'a Html, selector: &'a str) -> Option<ElementRef<'a>> {
    let parsed_selector = Selector::parse(selector).ok()?;
    document.select(&parsed_selector).next()
}

fn get_recipe_title(document: &Html, css_selector: &str, verbose: bool) -> Option<String> {
    let title = select_elements(document, css_selector).map(|e| e.inner_html());
    if verbose {
        println!("Title: {:?}", title);
    }
    title
}

fn get_recipe_description(document: &Html, css_selector: &str, verbose: bool) -> Option<String> {
    let description = select_elements(document, css_selector).map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string());
    if verbose {
        println!("Description: {:?}", description);
    }
    description
}

fn get_recipe_ingredients(document: &Html, css_selector: &str, verbose: bool) -> Option<Vec<String>> {
    let ingredients = select_elements(document, css_selector).map(|e| {
        e.text().collect::<Vec<_>>().iter().map(|&s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    });
    if verbose {
        println!("Ingredients: {:?}", ingredients);
    }
    ingredients
}

fn get_recipe_steps(document: &Html, css_selector: &str, verbose: bool) -> Option<Vec<String>> {
    let steps = select_elements(document, css_selector).map(|e| {
        e.text().collect::<Vec<_>>().iter().map(|&s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    });
    if verbose {
        println!("Steps: {:?}", steps);
    }
    steps
}

fn get_recipe_image(document: &Html, css_selector: &str, verbose: bool) -> Option<String> {
    let image_link = select_elements(document, css_selector).and_then(|e| e.value().attr("src").map(|src| src.to_string()));
    if verbose {
        println!("Image Link: {:?}", image_link);
    }
    image_link
}

fn parse_website_name(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    url.host_str()?.split('.').next().map(|s| s.to_string())
}
