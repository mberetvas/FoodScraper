use reqwest;
use scraper::{Html, Selector, ElementRef};
use serde::Serialize;
use std::fs::File;
use std::io::{self, Write};
use url::Url;

#[derive(Serialize)]
struct Recipe {
    title: Option<String>,
    description: Option<String>,
    ingredients: Option<Vec<String>>,
    steps: Option<Vec<String>>,
    image_link: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ask the user for the URL
    let input_url = loop {
        println!("Please enter the recipe URL:");
        let mut input_url = String::new();
        io::stdin().read_line(&mut input_url)?;
        let input_url = input_url.trim();

        if Url::parse(input_url).is_ok() {
            break input_url.to_string();
        } else {
            println!("Invalid URL. Please try again.");
        }
    };

    let html_body = reqwest::get(&input_url).await?.text().await?;

    // Set verbose to true for debugging
    let verbose = false;

    if verbose {
        println!("HTML Body: {}", html_body);
    }

    let document = Html::parse_document(&html_body);

    // Get the recipe details
    let recipe = Recipe {
        title: get_recipe_title(&document, verbose),
        description: get_recipe_description(&document, verbose),
        ingredients: get_recipe_ingredients(&document, verbose),
        steps: get_recipe_steps(&document, verbose),
        image_link: get_recipe_image(&document, verbose),
    };

    // Create a valid file name from the title
    let file_name = match &recipe.title {
        Some(title) => format!("recipe_{}.json", title.replace(" ", "_")),
        None => "recipe.json".to_string(),
    };

    // Serialize the recipe to JSON and write to a file
    let json = serde_json::to_string_pretty(&recipe)?;
    let mut file = File::create(&file_name)?;
    file.write_all(json.as_bytes())?;

    println!("Recipe JSON file '{}' created successfully.", file_name);

    Ok(())
}

fn select_elements<'a>(document: &'a Html, selector: &'a str) -> Option<ElementRef<'a>> {
    let parsed_selector = Selector::parse(selector).ok()?;
    document.select(&parsed_selector).next()
}

fn get_recipe_title(document: &Html, verbose: bool) -> Option<String> {
    let title = select_elements(document, "h1.text-center")
        .map(|e| e.inner_html());
    if verbose {
        println!("Title: {:?}", title);
    }
    title
}

fn get_recipe_description(document: &Html, verbose: bool) -> Option<String> {
    let description = select_elements(document, ".large-8")
        .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string());
    if verbose {
        println!("Description: {:?}", description);
    }
    description
}

fn get_recipe_ingredients(document: &Html, verbose: bool) -> Option<Vec<String>> {
    let ingredients = select_elements(document, ".detail-ingr-block").map(|e| {
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

fn get_recipe_steps(document: &Html, verbose: bool) -> Option<Vec<String>> {
    let steps = select_elements(document, "#preparation > ol:nth-child(2)").map(|e| {
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

fn get_recipe_image(document: &Html, verbose: bool) -> Option<String> {
    let image_link = select_elements(document, ".recipe-image").and_then(|e| {
        e.value().attr("src").map(|src| src.to_string())
    });
    if verbose {
        println!("Image Link: {:?}", image_link);
    }
    image_link
}