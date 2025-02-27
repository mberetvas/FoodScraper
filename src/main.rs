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

        match validate_url(input_url) {
            Ok(valid_url) => println!("Valid URL: {}", valid_url),
            Err(err) => println!("Error: {}", err),
        }

        if Url::parse(input_url).is_ok() && input_url.contains("https://15gram.be/") {
            break input_url.to_string();
        } else {
            println!("Invalid URL or URL does not contain 'https://15gram.be/'. Please try again.");
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
        Some(title) => format!("recipe_{}.json", title),
        None => "recipe.json".to_string(),
    };

    // Serialize the recipe to JSON and write to a file
    let json = serde_json::to_string_pretty(&recipe)?;
    std::fs::create_dir_all("recipes")?;
    let file_path = format!("recipes/{}", file_name);
    let mut file = File::create(&file_path)?;
    file.write_all(json.as_bytes())?;

    println!("Recipe JSON file '{}' created successfully.", file_name);

    Ok(())
}

/// Validates a given URL string.
///
/// This function trims the input URL string, parses it, and checks if it uses
/// the `http` or `https` scheme and contains a valid host. If the URL is valid,
/// it returns an `Ok(Url)`; otherwise, it returns an `Err(String)` with an
/// appropriate error message.
///
/// # Arguments
///
/// * `input_url` - A string slice that holds the URL to be validated.
///
/// # Returns
///
/// * `Result<Url, String>` - Returns `Ok(Url)` if the URL is valid, otherwise
///   returns `Err(String)` with an error message.
///
/// # Examples
///
/// ```
/// let url = "https://example.com";
/// match validate_url(url) {
///     Ok(valid_url) => println!("Valid URL: {}", valid_url),
///     Err(err) => println!("Error: {}", err),
/// }
/// ```
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

/// Selects the first element that matches the given CSS selector from the HTML document.
///
/// This function parses the provided CSS selector and uses it to find the first matching
/// element in the HTML document. If a matching element is found, it returns `Some(ElementRef)`,
/// otherwise it returns `None`.
///
/// # Arguments
///
/// * `document` - A reference to the `Html` document to search within.
/// * `selector` - A string slice that holds the CSS selector to match elements.
///
/// # Returns
///
/// * `Option<ElementRef>` - Returns `Some(ElementRef)` if a matching element is found,
///   otherwise returns `None`.
///
/// # Examples
///
/// ```
/// let document = Html::parse_document("<div class='example'>Content</div>");
/// let selector = ".example";
/// let element = select_elements(&document, selector);
/// assert!(element.is_some());
/// ```
fn select_elements<'a>(document: &'a Html, selector: &'a str) -> Option<ElementRef<'a>> {
    let parsed_selector = Selector::parse(selector).ok()?;
    document.select(&parsed_selector).next()
}

/// Extracts the recipe title from the HTML document.
///
/// This function searches for the first element that matches the CSS selector
/// `h1.text-center` in the provided HTML document and retrieves its inner HTML content
/// as the recipe title. If the `verbose` flag is set to `true`, it prints the extracted
/// title to the console.
///
/// # Arguments
///
/// * `document` - A reference to the `Html` document to search within.
/// * `verbose` - A boolean flag to enable verbose output.
///
/// # Returns
///
/// * `Option<String>` - Returns `Some(String)` containing the recipe title if found,
///   otherwise returns `None`.
///
/// # Examples
///
/// ```
/// let document = Html::parse_document("<h1 class='text-center'>Delicious Recipe</h1>");
/// let title = get_recipe_title(&document, true);
/// assert_eq!(title, Some("Delicious Recipe".to_string()));
/// ```
fn get_recipe_title(document: &Html, verbose: bool) -> Option<String> {
    let title = select_elements(document, "h1.text-center")
        .map(|e| e.inner_html());
    if verbose {
        println!("Title: {:?}", title);
    }
    title
}

/// Extracts the recipe description from the HTML document.
///
/// This function searches for the first element that matches the CSS selector
/// `.large-8` in the provided HTML document and retrieves its text content
/// as the recipe description. If the `verbose` flag is set to `true`, it prints
/// the extracted description to the console.
///
/// # Arguments
///
/// * `document` - A reference to the `Html` document to search within.
/// * `verbose` - A boolean flag to enable verbose output.
///
/// # Returns
///
/// * `Option<String>` - Returns `Some(String)` containing the recipe description if found,
///   otherwise returns `None`.
///
/// # Examples
///
/// ```
/// let document = Html::parse_document("<div class='large-8'>This is a description.</div>");
/// let description = get_recipe_description(&document, true);
/// assert_eq!(description, Some("This is a description.".to_string()));
/// ```
fn get_recipe_description(document: &Html, verbose: bool) -> Option<String> {
    let description = select_elements(document, ".large-8")
        .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string());
    if verbose {
        println!("Description: {:?}", description);
    }
    description
}

/// Extracts the recipe ingredients from the HTML document.
///
/// This function searches for the first element that matches the CSS selector
/// `.detail-ingr-block` in the provided HTML document and retrieves its text content
/// as a vector of strings, each representing an ingredient. If the `verbose` flag is set
/// to `true`, it prints the extracted ingredients to the console.
///
/// # Arguments
///
/// * `document` - A reference to the `Html` document to search within.
/// * `verbose` - A boolean flag to enable verbose output.
///
/// # Returns
///
/// * `Option<Vec<String>>` - Returns `Some(Vec<String>)` containing the ingredients if found,
///   otherwise returns `None`.
///
/// # Examples
///
/// ```
/// let document = Html::parse_document("<div class='detail-ingr-block'>Ingredient 1\nIngredient 2</div>");
/// let ingredients = get_recipe_ingredients(&document, true);
/// assert_eq!(ingredients, Some(vec!["Ingredient 1".to_string(), "Ingredient 2".to_string()]));
/// ```
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

/// Extracts the recipe steps from the HTML document.
///
/// This function searches for the first element that matches the CSS selector
/// `#preparation > ol:nth-child(2)` in the provided HTML document and retrieves its text content
/// as a vector of strings, each representing a step. If the `verbose` flag is set to `true`,
/// it prints the extracted steps to the console.
///
/// # Arguments
///
/// * `document` - A reference to the `Html` document to search within.
/// * `verbose` - A boolean flag to enable verbose output.
///
/// # Returns
///
/// * `Option<Vec<String>>` - Returns `Some(Vec<String>)` containing the steps if found,
///   otherwise returns `None`.
///
/// # Examples
///
/// ```
/// let document = Html::parse_document("<ol id='preparation'><li>Step 1</li><li>Step 2</li></ol>");
/// let steps = get_recipe_steps(&document, true);
/// assert_eq!(steps, Some(vec!["Step 1".to_string(), "Step 2".to_string()]));
/// ```
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

/// Extracts the recipe image link from the HTML document.
///
/// This function searches for the first element that matches the CSS selector
/// `.recipe-image` in the provided HTML document and retrieves the value of its
/// `src` attribute as the image link. If the `verbose` flag is set to `true`,
/// it prints the extracted image link to the console.
///
/// # Arguments
///
/// * `document` - A reference to the `Html` document to search within.
/// * `verbose` - A boolean flag to enable verbose output.
///
/// # Returns
///
/// * `Option<String>` - Returns `Some(String)` containing the image link if found,
///   otherwise returns `None`.
///
/// # Examples
///
/// ```
/// let document = Html::parse_document("<img class='recipe-image' src='image.jpg' />");
/// let image_link = get_recipe_image(&document, true);
/// assert_eq!(image_link, Some("image.jpg".to_string()));
/// ```
fn get_recipe_image(document: &Html, verbose: bool) -> Option<String> {
    let image_link = select_elements(document, ".recipe-image").and_then(|e| {
        e.value().attr("src").map(|src| src.to_string())
    });
    if verbose {
        println!("Image Link: {:?}", image_link);
    }
    image_link
}