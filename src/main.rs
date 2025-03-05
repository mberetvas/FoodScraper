use reqwest;
use scraper::{ ElementRef, Html, Selector };
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::{ self, Write };
use toml::Value;
use url::Url;

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
struct Selectors {
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
    loop {
        // Ask the user for the URL
        let input_url = loop {
            println!("Please enter the recipe URL:");
            let mut input_url = String::new();
            io::stdin().read_line(&mut input_url)?;
            let input_url = input_url.trim();

            match validate_url(input_url) {
                Ok(valid_url) => println!("Valid URL: {}", valid_url),
                Err(err) => println!("Error: {}", err),
            }

            if
                Url::parse(input_url).is_ok() &&
                (input_url.contains("https://15gram.be/") ||
                    input_url.contains("https://dagelijksekost.vrt.be/"))
            {
                break input_url.to_string();
            } else {
                println!(
                    "Invalid URL or URL does not contain a supported domain. Please try again."
                );
            }
        };

        let html_body = reqwest::get(&input_url).await?.text().await?;
        let document = Html::parse_document(&html_body);

        // Parse the website name from the URL
        let website_name = parse_website_name(&input_url).ok_or(
            "Failed to parse website name from URL"
        )?;

        // Load selectors from the TOML file
        let selectors = load_selectors("selectors.toml", &website_name)
            .map_err(|e| {
                println!("Failed to load selectors: {}", e);
                e
            })?;

        // Get the recipe details
        let recipe = Recipe {
            title: get_recipe_title(&document, &selectors.title, false),
            description: get_recipe_description(&document, &selectors.description, false),
            ingredients: get_recipe_ingredients(&document, &selectors.ingredients, false),
            steps: get_recipe_steps(&document, &selectors.steps, false),
            image_link: get_recipe_image(&document, &selectors.image, false),
            source_url: input_url.clone(),
        };

        // Create a valid file name from the title
        let file_name = match &recipe.title {
            Some(title) => format!("recipe_{}.json", title),
            None => "recipe.json".to_string(),
        };

        // Serialize the recipe to JSON and write to a file
        let json = serde_json::to_string_pretty(&recipe)
            .map_err(|e| {
                println!("Failed to serialize recipe to JSON: {}", e);
                e
            })?;
        std::fs::create_dir_all("recipes")
            .map_err(|e| {
                println!("Failed to create directory 'recipes': {}", e);
                e
            })?;
        let file_path = format!("recipes/{}", file_name);
        let mut file = File::create(&file_path)
            .map_err(|e| {
                println!("Failed to create file '{}': {}", file_path, e);
                e
            })?;
        file.write_all(json.as_bytes())
            .map_err(|e| {
                println!("Failed to write to file '{}': {}", file_path, e);
                e
            })?;

        println!("Recipe JSON file '{}' created successfully.", file_name);

        // Ask the user if they want to continue
        println!("Do you want to scrape another recipe? (yes/no):");
        let mut continue_input = String::new();
        io::stdin().read_line(&mut continue_input)?;
        if continue_input.trim().to_lowercase() != "yes" {
            break;
        }
    }

    Ok(())
}

fn load_selectors(file_path: &str, website: &str) -> Result<Selectors, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let value: Value = toml::from_str(&content)?;

    let website_selectors = value.get(website).ok_or("Website not found in selectors file")?;
    Ok(Selectors {
        title: website_selectors
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        description: website_selectors
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        ingredients: website_selectors
            .get("ingredients")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        steps: website_selectors
            .get("steps")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        image: website_selectors
            .get("image")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
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
    let description = select_elements(document, css_selector).map(|e|
        e.text().collect::<Vec<_>>().join(" ").trim().to_string()
    );
    if verbose {
        println!("Description: {:?}", description);
    }
    description
}

fn get_recipe_ingredients(
    document: &Html,
    css_selector: &str,
    verbose: bool
) -> Option<Vec<String>> {
    let ingredients = select_elements(document, css_selector).map(|e| {
        e.text()
            .collect::<Vec<_>>()
            .iter()
            .map(|&s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });
    if verbose {
        println!("Ingredients: {:?}", ingredients);
    }
    ingredients
}

fn get_recipe_steps(document: &Html, css_selector: &str, verbose: bool) -> Option<Vec<String>> {
    let steps = select_elements(document, css_selector).map(|e| {
        e.text()
            .collect::<Vec<_>>()
            .iter()
            .map(|&s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });
    if verbose {
        println!("Steps: {:?}", steps);
    }
    steps
}

fn get_recipe_image(document: &Html, css_selector: &str, verbose: bool) -> Option<String> {
    let image_link = select_elements(document, css_selector).and_then(|e|
        e
            .value()
            .attr("src")
            .map(|src| src.to_string())
    );
    if verbose {
        println!("Image Link: {:?}", image_link);
    }
    image_link
}

fn parse_website_name(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    url.host_str()?
        .split('.')
        .next()
        .map(|s| s.to_string())
}
