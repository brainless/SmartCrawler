use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HtmlTreeNode {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub text_content: Option<String>,
    pub children: Vec<HtmlTreeNode>,
}

impl HtmlTreeNode {
    pub fn new(tag: String) -> Self {
        Self {
            tag,
            id: None,
            classes: Vec::new(),
            text_content: None,
            children: Vec::new(),
        }
    }

    pub fn display_tree(&self, depth: usize) -> String {
        let mut result = String::new();

        // Create indentation with ASCII tree characters
        if depth > 0 {
            for i in 0..depth {
                if i == depth - 1 {
                    result.push_str("├── ");
                } else {
                    result.push_str("│   ");
                }
            }
        }

        // Add tag name
        result.push_str(&self.tag);

        // Add ID if present
        if let Some(id) = &self.id {
            result.push_str(&format!(" id=\"{id}\""));
        }

        // Add classes if present
        if !self.classes.is_empty() {
            let classes = self.classes.join(" ");
            result.push_str(&format!(" class=\"{classes}\""));
        }

        // Add text content if present
        if let Some(text) = &self.text_content {
            if !text.trim().is_empty() {
                let trimmed = text.trim();
                result.push_str(&format!(" [{trimmed}]"));
            }
        }

        result.push('\n');

        // Recursively display children
        for child in &self.children {
            result.push_str(&child.display_tree(depth + 1));
        }

        result
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StructuredContent {
    LongForm(String),
    ItemList(Vec<Item>),
    TabularData(Vec<HashMap<String, String>>),
    Mixed {
        long_form: Option<String>,
        items: Vec<Item>,
        tables: Vec<HashMap<String, String>>,
    },
}

impl StructuredContent {
    /// Converts the structured data into a well-formatted text prompt suitable for LLMs
    pub fn to_prompt(&self) -> String {
        match self {
            StructuredContent::LongForm(content) => {
                format!("ARTICLE CONTENT:\n\n{content}\n")
            }
            StructuredContent::ItemList(items) => {
                let mut prompt = String::from("ITEM LIST:\n\n");

                for (index, item) in items.iter().enumerate() {
                    prompt.push_str(&format!("{}. {}\n", index + 1, item.title));

                    if let Some(description) = &item.description {
                        prompt.push_str(&format!("   Description: {description}\n"));
                    }

                    if let Some(url) = &item.url {
                        prompt.push_str(&format!("   URL: {url}\n"));
                    }

                    if !item.metadata.is_empty() {
                        prompt.push_str("   Metadata:\n");
                        for (key, value) in &item.metadata {
                            prompt.push_str(&format!("     {key}: {value}\n"));
                        }
                    }

                    prompt.push('\n');
                }

                prompt
            }
            StructuredContent::TabularData(tables) => {
                let mut prompt = String::from("TABLE DATA:\n\n");

                if !tables.is_empty() {
                    // Get all unique headers across all rows
                    let mut all_headers: std::collections::HashSet<String> =
                        std::collections::HashSet::new();
                    for row in tables {
                        for key in row.keys() {
                            all_headers.insert(key.clone());
                        }
                    }
                    let headers: Vec<String> = all_headers.into_iter().collect();

                    // Create header row
                    prompt.push_str(&format!("| {} |\n", headers.join(" | ")));
                    prompt.push_str(&format!(
                        "|{}|\n",
                        headers.iter().map(|_| "---").collect::<Vec<_>>().join("|")
                    ));

                    // Create data rows
                    for row in tables {
                        let row_values: Vec<String> = headers
                            .iter()
                            .map(|header| {
                                row.get(header).cloned().unwrap_or_else(|| "-".to_string())
                            })
                            .collect();
                        prompt.push_str(&format!("| {} |\n", row_values.join(" | ")));
                    }
                }

                prompt
            }
            StructuredContent::Mixed {
                long_form,
                items,
                tables,
            } => {
                let mut prompt = String::from("MIXED CONTENT:\n\n");

                if let Some(content) = long_form {
                    prompt.push_str("ARTICLE SECTION:\n");
                    prompt.push_str(content);
                    prompt.push_str("\n\n");
                }

                if !items.is_empty() {
                    prompt.push_str("ITEMS SECTION:\n");
                    for (index, item) in items.iter().enumerate() {
                        prompt.push_str(&format!("{}. {}\n", index + 1, item.title));

                        if let Some(description) = &item.description {
                            prompt.push_str(&format!("   Description: {description}\n"));
                        }

                        if let Some(url) = &item.url {
                            prompt.push_str(&format!("   URL: {url}\n"));
                        }

                        if !item.metadata.is_empty() {
                            prompt.push_str("   Metadata:\n");
                            for (key, value) in &item.metadata {
                                prompt.push_str(&format!("     {key}: {value}\n"));
                            }
                        }

                        prompt.push('\n');
                    }
                    prompt.push('\n');
                }

                if !tables.is_empty() {
                    prompt.push_str("TABLE SECTION:\n");

                    // Get all unique headers across all rows
                    let mut all_headers: std::collections::HashSet<String> =
                        std::collections::HashSet::new();
                    for row in tables {
                        for key in row.keys() {
                            all_headers.insert(key.clone());
                        }
                    }
                    let headers: Vec<String> = all_headers.into_iter().collect();

                    // Create header row
                    prompt.push_str(&format!("| {} |\n", headers.join(" | ")));
                    prompt.push_str(&format!(
                        "|{}|\n",
                        headers.iter().map(|_| "---").collect::<Vec<_>>().join("|")
                    ));

                    // Create data rows
                    for row in tables {
                        let row_values: Vec<String> = headers
                            .iter()
                            .map(|header| {
                                row.get(header).cloned().unwrap_or_else(|| "-".to_string())
                            })
                            .collect();
                        prompt.push_str(&format!("| {} |\n", row_values.join(" | ")));
                    }
                }

                prompt
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedWebPage {
    pub url: String,
    pub title: Option<String>,
    pub content: StructuredContent,
    pub links: Vec<String>,
    pub meta_description: Option<String>,
    pub headings: Vec<String>,
    pub alternative_extraction: Option<HtmlTreeNode>,
}

/// Extracts structured data from HTML pages. The function should:
/// 1. **Input**: Raw HTML string
///
/// 2. **Output**: An enum representing different data types:
///    - LongForm(String) - for articles, blog posts, documentation
///    - ItemList(Vec<Item>) - for product listings, search results, news feeds
///    - TabularData(Vec<HashMap<String, String>>) - for tables, structured data
///    - Mixed(combination of above types)
///
/// 3. **Key requirements**:
///    - Filter out navigation, headers, footers, ads, and UI elements
///    - Focus on main content area identification
///    - Handle common HTML patterns (article tags, main tags, content divs)
///    - Extract clean text without HTML artifacts
///    - Preserve important structure (headings, lists, table relationships)
pub async fn extract_structured_data(html: &str) -> StructuredContent {
    // First, clean the HTML to remove unnecessary elements and attributes
    let cleaned_html = clean_html(html);
    let document = Html::parse_document(&cleaned_html);

    // Extract main content by trying different selectors
    let main_content = extract_main_content(&document);

    // Extract structured data components
    let long_form_content = extract_long_form_content(&document);
    let items = extract_item_list(&document);
    let tables = extract_tabular_data(&document);

    // Determine the most appropriate data type based on content
    let has_long_form = !long_form_content.is_empty();
    let has_items = !items.is_empty();
    let has_tables = !tables.is_empty();

    match (has_long_form, has_items, has_tables) {
        (true, false, false) => StructuredContent::LongForm(long_form_content),
        (false, true, false) => StructuredContent::ItemList(items),
        (false, false, true) => StructuredContent::TabularData(tables),
        (true, true, _) | (true, _, true) | (_, true, true) => StructuredContent::Mixed {
            long_form: if has_long_form {
                Some(long_form_content)
            } else {
                None
            },
            items,
            tables,
        },
        (false, false, false) => {
            // Fallback to main content if no structured data found
            StructuredContent::LongForm(main_content)
        }
    }
}

/// Extracts HTML content as a tree structure according to the alternative extraction approach
pub fn extract_alternative_tree(html: &str) -> Option<HtmlTreeNode> {
    let document = Html::parse_document(html);

    // Start with the body element or html element if body is not found
    let body_selector = Selector::parse("body").unwrap();
    let html_selector = Selector::parse("html").unwrap();

    if let Some(body_element) = document.select(&body_selector).next() {
        Some(build_tree_from_element(&body_element))
    } else {
        document.select(&html_selector).next().map(|html_element| build_tree_from_element(&html_element))
    }
}

fn build_tree_from_element(element: &scraper::ElementRef) -> HtmlTreeNode {
    let tag_name = element.value().name().to_string();
    let mut node = HtmlTreeNode::new(tag_name);

    // Extract ID attribute
    if let Some(id) = element.value().attr("id") {
        let trimmed_id = id.trim();
        if !trimmed_id.is_empty() {
            node.id = Some(trimmed_id.to_string());
        }
    }

    // Extract and filter class attributes
    if let Some(class_attr) = element.value().attr("class") {
        let classes: Vec<String> = class_attr
            .split_whitespace()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !should_ignore_class(s))
            .collect();
        node.classes = classes;
    }

    // Extract direct text content (not from children)
    let mut direct_text = String::new();
    for child in element.children() {
        if let scraper::node::Node::Text(text) = child.value() {
            direct_text.push_str(text.trim());
            direct_text.push(' ');
        }
    }
    let direct_text = direct_text.trim().to_string();
    if !direct_text.is_empty() && !is_control_characters_only(&direct_text) {
        node.text_content = Some(direct_text);
    }

    // Process children
    let mut paragraph_texts = Vec::new();
    for child in element.children() {
        if let scraper::node::Node::Element(child_element) = child.value() {
            let child_ref = scraper::ElementRef::wrap(child).unwrap();

            // Skip if should ignore this tag
            if should_ignore_element(&child_ref) {
                continue;
            }

            let child_tag = child_element.name();

            // Handle paragraph merging
            if child_tag == "p" {
                let p_text = child_ref.text().collect::<String>().trim().to_string();
                if !p_text.is_empty() && !is_control_characters_only(&p_text) {
                    paragraph_texts.push(p_text);
                }
            } else {
                // If we have accumulated paragraph texts, create a merged paragraph node
                if !paragraph_texts.is_empty() {
                    let merged_text = paragraph_texts.join(" ");
                    let mut p_node = HtmlTreeNode::new("p".to_string());
                    p_node.text_content = Some(merged_text);
                    node.children.push(p_node);
                    paragraph_texts.clear();
                }

                // Add the child node
                let child_node = build_tree_from_element(&child_ref);
                node.children.push(child_node);
            }
        }
    }

    // Handle any remaining paragraph texts
    if !paragraph_texts.is_empty() {
        let merged_text = paragraph_texts.join(" ");
        let mut p_node = HtmlTreeNode::new("p".to_string());
        p_node.text_content = Some(merged_text);
        node.children.push(p_node);
    }

    node
}

fn should_ignore_element(element: &scraper::ElementRef) -> bool {
    let tag_name = element.value().name();

    // Check if tag is empty (has no content and no children)
    if element.children().count() == 0 && element.text().collect::<String>().trim().is_empty() {
        return true;
    }

    // Check if tag is image, video, svg, path, etc.
    matches!(tag_name, "img" | "video" | "svg" | "path" | "circle" | "rect" | "line" | "polygon" | "polyline"
        | "ellipse" | "audio" | "source" | "track" | "canvas" | "embed" | "object" | "iframe")
}

fn should_ignore_class(class_name: &str) -> bool {
    matches!(class_name, "active" | "highlighted" | "selected")
}

fn is_control_characters_only(text: &str) -> bool {
    text.chars().all(|c| c.is_control() || c.is_whitespace())
}

// Static versions for testing
fn clean_text(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn extract_main_content(document: &Html) -> String {
    // Try to find main content using common selectors
    let main_selectors = [
        "main",
        "article",
        "[role='main']",
        ".main-content",
        ".content",
        "#main",
        "#content",
        ".post-content",
        ".entry-content",
        ".article-content",
    ];

    for selector_str in &main_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join(" ");
                let cleaned_text = clean_text(&text);
                if !cleaned_text.trim().is_empty() && cleaned_text.len() > 50 {
                    return cleaned_text;
                }
            }
        }
    }

    // Fallback to body content
    extract_body_content(document)
}

fn extract_body_content(document: &Html) -> String {
    if let Ok(body_selector) = Selector::parse("body") {
        if let Some(body) = document.select(&body_selector).next() {
            // Since HTML is already cleaned, we can simply extract all text content
            let text = body.text().collect::<Vec<_>>().join(" ");
            return clean_text(&text);
        }
    }

    String::new()
}

fn extract_long_form_content(document: &Html) -> String {
    // Look for article-like content
    let article_selectors = [
        "article",
        ".article",
        ".post",
        ".blog-post",
        ".content",
        ".entry",
        ".story",
        "[role='article']",
    ];

    for selector_str in &article_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join(" ");
                let cleaned_text = clean_text(&text);
                if cleaned_text.len() > 200 {
                    // Consider it long-form if substantial content
                    return cleaned_text;
                }
            }
        }
    }

    String::new()
}

fn extract_item_list(document: &Html) -> Vec<Item> {
    let mut items = Vec::new();

    // Look for common list patterns
    let list_selectors = [
        ".product-item",
        ".item",
        ".result",
        ".listing",
        ".card",
        ".news-item",
        ".article-item",
        ".post-item",
        ".search-result",
        "article",
        ".entry",
        "li",
        ".tile",
        ".grid-item",
    ];

    for selector_str in &list_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let elements: Vec<_> = document.select(&selector).collect();

            // Only consider it a list if there are multiple similar items
            if elements.len() >= 2 {
                for element in elements {
                    let title = extract_item_title(&element);
                    if !title.is_empty() {
                        let description = extract_item_description(&element);
                        let url = extract_item_url(&element);
                        let metadata = extract_item_metadata(&element);

                        items.push(Item {
                            title,
                            description,
                            url,
                            metadata,
                        });
                    }
                }

                // Return early if we found a good set of items
                if !items.is_empty() {
                    break;
                }
            }
        }
    }

    items
}

fn extract_tabular_data(document: &Html) -> Vec<HashMap<String, String>> {
    let mut tables = Vec::new();

    if let Ok(table_selector) = Selector::parse("table") {
        for table in document.select(&table_selector) {
            let mut table_data = Vec::new();
            let mut headers = Vec::new();

            // Extract headers
            if let Ok(th_selector) = Selector::parse("th") {
                headers = table
                    .select(&th_selector)
                    .map(|th| th.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .filter(|h| !h.is_empty())
                    .collect();
            }

            // If no headers found, use first row as headers
            if headers.is_empty() {
                if let Ok(first_row_selector) = Selector::parse("tr:first-child td") {
                    headers = table
                        .select(&first_row_selector)
                        .map(|td| td.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .filter(|h| !h.is_empty())
                        .collect();
                }
            }

            // Extract rows
            if let Ok(row_selector) = Selector::parse("tr") {
                let rows: Vec<_> = table.select(&row_selector).collect();
                let start_idx = if headers.is_empty() { 0 } else { 1 };

                for row in rows.iter().skip(start_idx) {
                    if let Ok(cell_selector) = Selector::parse("td") {
                        let cells: Vec<String> = row
                            .select(&cell_selector)
                            .map(|td| td.text().collect::<Vec<_>>().join(" ").trim().to_string())
                            .collect();

                        if !cells.is_empty() {
                            let mut row_map = HashMap::new();
                            for (i, cell) in cells.iter().enumerate() {
                                let key = if i < headers.len() {
                                    headers[i].clone()
                                } else {
                                    format!("column_{}", i + 1)
                                };
                                row_map.insert(key, cell.clone());
                            }
                            table_data.push(row_map);
                        }
                    }
                }
            }

            if !table_data.is_empty() {
                tables.extend(table_data);
            }
        }
    }

    tables
}

fn extract_item_title(element: &scraper::ElementRef) -> String {
    let title_selectors = ["h1", "h2", "h3", "h4", ".title", ".name", ".heading", "a"];

    for selector_str in &title_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(title_element) = element.select(&selector).next() {
                let title = title_element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                if !title.is_empty() {
                    return title;
                }
            }
        }
    }

    // Fallback to element's own text if no specific title found
    let text = element.text().collect::<Vec<_>>().join(" ");
    clean_text(&text)
        .chars()
        .take(100)
        .collect::<String>()
        .trim()
        .to_string()
}

fn extract_item_description(element: &scraper::ElementRef) -> Option<String> {
    let desc_selectors = [".description", ".summary", ".excerpt", "p", ".text"];

    for selector_str in &desc_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(desc_element) = element.select(&selector).next() {
                let desc = desc_element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                if !desc.is_empty() && desc.len() > 10 {
                    return Some(desc);
                }
            }
        }
    }

    None
}

fn extract_item_url(element: &scraper::ElementRef) -> Option<String> {
    if let Ok(link_selector) = Selector::parse("a[href]") {
        if let Some(link) = element.select(&link_selector).next() {
            return link.value().attr("href").map(|href| href.to_string());
        }
    }

    None
}

fn extract_item_metadata(element: &scraper::ElementRef) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    // Extract common metadata attributes
    let meta_selectors = [
        (".price", "price"),
        (".date", "date"),
        (".author", "author"),
        (".category", "category"),
        (".tags", "tags"),
        (".rating", "rating"),
    ];

    for (selector_str, key) in &meta_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = element.select(&selector).next() {
                let value = element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                if !value.is_empty() {
                    metadata.insert(key.to_string(), value);
                }
            }
        }
    }

    metadata
}

/// Check if a filename looks like a slug (readable text with hyphens/underscores)
/// Examples of slug-like names: "my-article-image.jpg", "product_photo.png", "hero-banner.webp"
/// Examples of non-slug names: "img123.jpg", "photo.png", "image.gif", "banner1.jpg"
fn is_slug_like_filename(filename: &str) -> bool {
    // Remove file extension for analysis
    let name_without_ext = filename.split('.').next().unwrap_or(filename);

    // Must contain at least one hyphen or underscore (indicating word separation)
    let has_separators = name_without_ext.contains('-') || name_without_ext.contains('_');

    // Must be at least 5 characters long (to avoid very short names like "a-b.jpg")
    let min_length = name_without_ext.len() >= 5;

    // Should not be mostly numbers (avoid names like "12345-6.jpg")
    let char_count = name_without_ext.chars().count();
    let digit_count = name_without_ext
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count();
    let mostly_letters = digit_count < (char_count / 2);

    has_separators && min_length && mostly_letters
}

/// Check if alt text is meaningful (not just empty or whitespace)
fn has_meaningful_alt_text(alt_text: &str) -> bool {
    let trimmed = alt_text.trim();
    // Must be at least 3 characters and contain at least one letter
    trimmed.len() >= 3 && trimmed.chars().any(|c| c.is_alphabetic())
}

/// Filter images and SVGs: remove img and svg tags unless they have meaningful attributes
///
/// For IMG tags: Keep if they have slug-like filename or meaningful alt text
/// For SVG tags: Keep if they have aria-label, title, or meaningful class/id attributes
fn filter_images(html: &str) -> String {
    let mut result = html.to_string();

    // Handle IMG tags
    let img_regex = Regex::new(r#"<img[^>]*>"#).expect("Valid img regex");
    result = img_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let img_tag = caps.get(0).unwrap().as_str();

            // Extract src attribute
            let src_regex = Regex::new(r#"src\s*=\s*["']([^"']*)["']"#).expect("Valid src regex");
            let has_slug_filename = if let Some(src_match) = src_regex.captures(img_tag) {
                let src_value = src_match.get(1).unwrap().as_str();
                // Extract filename from path
                let filename = src_value.split('/').next_back().unwrap_or(src_value);
                is_slug_like_filename(filename)
            } else {
                false
            };

            // Extract alt attribute
            let alt_regex = Regex::new(r#"alt\s*=\s*["']([^"']*)["']"#).expect("Valid alt regex");
            let has_meaningful_alt = if let Some(alt_match) = alt_regex.captures(img_tag) {
                let alt_value = alt_match.get(1).unwrap().as_str();
                has_meaningful_alt_text(alt_value)
            } else {
                false
            };

            // Keep image if it has either slug-like filename OR meaningful alt text
            if has_slug_filename || has_meaningful_alt {
                img_tag.to_string()
            } else {
                String::new() // Remove the image
            }
        })
        .to_string();

    // Handle SVG tags (both self-closing and with content)
    let svg_regex = Regex::new(r#"(?s)<svg[^>]*(?:/>|>.*?</svg>)"#).expect("Valid svg regex");
    result = svg_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let svg_tag = caps.get(0).unwrap().as_str();

            // Check for meaningful attributes in SVG
            let has_meaningful_content =
                // Check for aria-label or title attributes
                svg_tag.contains("aria-label=") ||
                svg_tag.contains("title=") ||
                // Check for meaningful class names that suggest content
                (svg_tag.contains("class=") &&
                 (svg_tag.contains("icon-") || svg_tag.contains("logo") ||
                  svg_tag.contains("diagram") || svg_tag.contains("chart") ||
                  svg_tag.contains("illustration"))) ||
                // Check for meaningful id attributes
                (svg_tag.contains("id=") &&
                 (svg_tag.contains("logo") || svg_tag.contains("diagram") ||
                  svg_tag.contains("chart") || svg_tag.contains("illustration")));

            // Keep SVG if it has meaningful content indicators
            if has_meaningful_content {
                svg_tag.to_string()
            } else {
                String::new() // Remove the SVG
            }
        })
        .to_string();

    result
}

/// Cleans HTML by removing unnecessary tags, attributes, and content that are not useful
/// for structured data extraction.
///
/// This function performs comprehensive HTML cleaning to prepare content for structured
/// data extraction by:
///
/// ## Elements Removed:
/// - **HTML comments**: All `<!-- ... -->` comments
/// - **Navigation elements**: `<nav>`, elements with navigation roles
/// - **UI chrome**: `<header>`, `<footer>`, `<aside>`, sidebars
/// - **Interactive elements**: `<script>`, `<style>`, `<noscript>`
/// - **Resource links**: `<link>` tags for stylesheets, icons, preload, prefetch, etc.
/// - **Meta elements**: `<meta>`, `<title>` (extracted separately)
/// - **Advertising**: Elements with ad-related classes
/// - **Comments & social**: Comment sections, social media widgets
/// - **Forms**: `<form>`, `<input>`, `<button>` (not useful for content extraction)
/// - **Low-value images**: `<img>` tags without slug-like filenames or meaningful alt text
/// - **Decorative SVGs**: `<svg>` tags without aria-label, title, or meaningful class/id attributes
///
/// ## Attributes Cleaned:
/// - Removes all `style` attributes (inline CSS)
/// - Removes all `class` attributes except content-specific ones
/// - Removes all `id` attributes except semantic ones
/// - Removes event handlers (`onclick`, `onload`, etc.)
/// - Removes tracking attributes (`data-track`, `data-analytics`, etc.)
/// - Keeps structural attributes like `href`, `src`, `alt`, `title`
///
/// ## Content Structure Preserved:
/// - Main content containers (`<main>`, `<article>`, `<section>`)
/// - Text formatting (`<p>`, `<h1>`-`<h6>`, `<strong>`, `<em>`)
/// - Lists (`<ul>`, `<ol>`, `<li>`, `<dl>`, `<dt>`, `<dd>`)
/// - Tables (`<table>`, `<tr>`, `<td>`, `<th>`, `<thead>`, `<tbody>`)
/// - Links (`<a>`) with `href` attributes
/// - Images (`<img>`) with slug-like filenames or meaningful alt text
/// - SVGs (`<svg>`) with aria-label, title, or meaningful class/id attributes
///
/// ## Examples:
/// ```html
/// // Input:
/// <html>
///   <head><title>Page</title><script>...</script></head>
///   <body>
///     <nav class="navbar">...</nav>
///     <main id="content">
///       <article class="post">
///         <h1>Title</h1>
///         <p style="color: red;">Content</p>
///       </article>
///     </main>
///     <footer>...</footer>
///   </body>
/// </html>
///
/// // Output:
/// <html>
///   <body>
///     <main>
///       <article>
///         <h1>Title</h1>
///         <p>Content</p>
///       </article>
///     </main>
///   </body>
/// </html>
/// ```
///
/// # Arguments
/// * `html` - Raw HTML string to clean
///
/// # Returns
/// * Cleaned HTML string optimized for structured data extraction
pub fn clean_html(html: &str) -> String {
    // Use regex patterns to remove unwanted elements and attributes directly from HTML string
    // This is more reliable than trying to parse and manipulate the DOM

    let mut cleaned_html = html.to_string();

    // Remove unwanted elements using regex patterns
    let element_patterns = [
        // HTML comments
        r"(?s)<!--.*?-->",
        // Scripts and styles (including content)
        r"(?s)<script[^>]*>.*?</script>",
        r"(?s)<style[^>]*>.*?</style>",
        r"(?s)<noscript[^>]*>.*?</noscript>",
        // Link tags for external resources (stylesheets, icons, preload, etc.)
        r#"<link[^>]*rel="stylesheet"[^>]*>"#,
        r#"<link[^>]*rel="icon"[^>]*>"#,
        r#"<link[^>]*rel="shortcut icon"[^>]*>"#,
        r#"<link[^>]*rel="apple-touch-icon"[^>]*>"#,
        r#"<link[^>]*rel="preload"[^>]*>"#,
        r#"<link[^>]*rel="prefetch"[^>]*>"#,
        r#"<link[^>]*rel="preconnect"[^>]*>"#,
        r#"<link[^>]*rel="dns-prefetch"[^>]*>"#,
        r#"<link[^>]*rel="manifest"[^>]*>"#,
        r#"<link[^>]*rel="canonical"[^>]*>"#,
        r#"<link[^>]*rel="alternate"[^>]*>"#,
        r#"<link[^>]*rel="mask-icon"[^>]*>"#,
        r#"<link[^>]*rel="apple-touch-startup-image"[^>]*>"#,
        // Generic pattern for other link tags with common resource relationships
        r#"<link[^>]*rel="(?:next|prev|first|last|up|edit|bookmark|help|license|nofollow|noreferrer|opener|external|tag|author)"[^>]*>"#,
        // Link tags with type attributes for resources
        r#"<link[^>]*type="text/css"[^>]*>"#,
        r#"<link[^>]*type="image/"[^>]*>"#,
        // Navigation and UI chrome
        r"(?s)<nav[^>]*>.*?</nav>",
        r"(?s)<header[^>]*>.*?</header>",
        r"(?s)<footer[^>]*>.*?</footer>",
        r"(?s)<aside[^>]*>.*?</aside>",
        // Form elements
        r"(?s)<form[^>]*>.*?</form>",
        r"<input[^>]*>",
        r"<button[^>]*>.*?</button>",
        r"(?s)<textarea[^>]*>.*?</textarea>",
        r"(?s)<select[^>]*>.*?</select>",
        // Interactive/dynamic content
        r"(?s)<iframe[^>]*>.*?</iframe>",
        r"(?s)<embed[^>]*>.*?</embed>",
        r"(?s)<object[^>]*>.*?</object>",
        // Elements with problematic classes
        r#"(?s)<[^>]*class="[^"]*(?:navbar|navigation|menu|sidebar|ad|advertisement|social|comment|popup|modal)[^"]*"[^>]*>.*?</[^>]*>"#,
        // Elements with problematic roles
        r#"(?s)<[^>]*role="(?:navigation|banner|contentinfo|complementary|search|form)"[^>]*>.*?</[^>]*>"#,
    ];

    for pattern in &element_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            cleaned_html = regex.replace_all(&cleaned_html, "").to_string();
        }
    }

    // Filter images: remove images unless they have slug-like filename or alt text
    cleaned_html = filter_images(&cleaned_html);

    // Remove problematic attributes using regex patterns
    let attribute_patterns = [
        // Style attributes
        r#" style="[^"]*""#,
        r#" style='[^']*'"#,
        // Event handlers
        r#" on\w+="[^"]*""#,
        r#" on\w+='[^']*'"#,
        // Tracking attributes
        r#" data-(?:track|analytics|ga|fb)[^=]*="[^"]*""#,
        // Problematic class attributes
        r#" class="[^"]*(?:nav|menu|sidebar|ad|social|comment|popup|modal)[^"]*""#,
        // Problematic id attributes
        r#" id="[^"]*(?:nav|menu|sidebar|ad|social|comment|popup|modal)[^"]*""#,
    ];

    for pattern in &attribute_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            cleaned_html = regex.replace_all(&cleaned_html, "").to_string();
        }
    }

    // Clean up extra whitespace and empty lines first
    let lines: Vec<&str> = cleaned_html
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();

    let mut cleaned_html = lines.join("\n");

    // Remove empty HTML tags (multiple passes to handle nested empty tags)
    // We need multiple passes because removing an empty tag might make its parent empty
    let mut previous_length = 0;
    let self_closing_regex = Regex::new(r"<(\w+)\s*/>").expect("Valid regex");

    while cleaned_html.len() != previous_length {
        previous_length = cleaned_html.len();

        // Remove self-closing empty tags that don't need to be preserved
        cleaned_html = self_closing_regex
            .replace_all(&cleaned_html, |caps: &regex::Captures| {
                let tag = &caps[1];
                // Preserve certain self-closing tags even if empty
                // Note: img tags are now filtered separately by filter_images function
                match tag {
                    "br" | "hr" | "input" | "meta" | "link" | "area" | "base" | "col" | "embed"
                    | "source" | "track" | "wbr" => caps.get(0).unwrap().as_str().to_string(),
                    _ => String::new(), // Remove other empty self-closing tags
                }
            })
            .to_string();

        // Remove empty paired tags (no content between opening and closing tags)
        // Try different patterns to catch various empty tag formats

        // Remove common empty tags individually (since backreferences don't work as expected)
        let empty_tag_patterns = [
            r"<div></div>",
            r"<span></span>",
            r"<p></p>",
            r"<section></section>",
            r"<article></article>",
            r"<header></header>",
            r"<footer></footer>",
            r"<aside></aside>",
            r"<main></main>",
            r"<nav></nav>",
            r"<figure></figure>",
            r"<figcaption></figcaption>",
            r"<address></address>",
            r"<details></details>",
            r"<summary></summary>",
            r"<mark></mark>",
            r"<small></small>",
            r"<strong></strong>",
            r"<em></em>",
            r"<b></b>",
            r"<i></i>",
            r"<u></u>",
            r"<s></s>",
            r"<del></del>",
            r"<ins></ins>",
            r"<sub></sub>",
            r"<sup></sup>",
            r"<code></code>",
            r"<kbd></kbd>",
            r"<samp></samp>",
            r"<var></var>",
            r"<pre></pre>",
            r"<blockquote></blockquote>",
            r"<cite></cite>",
            r"<q></q>",
            r"<abbr></abbr>",
            r"<dfn></dfn>",
            r"<time></time>",
        ];

        for pattern in &empty_tag_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                cleaned_html = regex.replace_all(&cleaned_html, "").to_string();
            }
        }

        // Also remove empty tags with attributes
        let empty_tag_with_attrs_patterns = [
            r"<div\s[^>]*></div>",
            r"<span\s[^>]*></span>",
            r"<p\s[^>]*></p>",
            r"<section\s[^>]*></section>",
            r"<article\s[^>]*></article>",
            r"<header\s[^>]*></header>",
            r"<footer\s[^>]*></footer>",
            r"<aside\s[^>]*></aside>",
            r"<main\s[^>]*></main>",
        ];

        for pattern in &empty_tag_with_attrs_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                cleaned_html = regex.replace_all(&cleaned_html, "").to_string();
            }
        }

        // Remove tags that only contain whitespace
        let whitespace_tag_patterns = [
            r"<div>\s+</div>",
            r"<span>\s+</span>",
            r"<p>\s+</p>",
            r"<section>\s+</section>",
            r"<article>\s+</article>",
            r"<div\s[^>]*>\s+</div>",
            r"<span\s[^>]*>\s+</span>",
        ];

        for pattern in &whitespace_tag_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                cleaned_html = regex.replace_all(&cleaned_html, "").to_string();
            }
        }
    }

    cleaned_html
}

/// Fetches HTML content from a URL using the browser/webdriver
/// This ensures JavaScript-rendered content is properly loaded and handles dynamic pages
///
/// # Arguments
/// * `url` - The URL to fetch HTML content from
///
/// # Returns
/// * `Result<String, Box<dyn std::error::Error>>` - HTML content or error
async fn fetch_html_from_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    use crate::browser::Browser;

    let browser = Browser::new().await?;

    // Use browser to fetch raw HTML - this will handle JavaScript and dynamic content
    let html_content = browser.fetch_html(url).await?;

    // Close the browser to clean up resources
    browser.close().await?;

    Ok(html_content)
}

/// Reads an HTML file, cleans it using the clean_html function, and writes it to an output file
///
/// # Arguments
/// * `input_path` - Path to the input HTML file
/// * `output_path` - Path where the cleaned HTML should be written
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error information
///
/// # Examples
/// ```rust,no_run
/// use smart_crawler::content::clean_html_file;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Clean an HTML file
///     clean_html_file("input.html", "output.html")?;
///     Ok(())
/// }
/// ```
pub fn clean_html_file(
    input_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the input HTML file
    let html_content = std::fs::read_to_string(input_path)?;

    // Clean the HTML
    let cleaned_html = clean_html(&html_content);

    // Write the cleaned HTML to the output file
    std::fs::write(output_path, cleaned_html)?;

    Ok(())
}

/// Cleans HTML from either a local file or URL, and writes it to an output file
///
/// # Arguments
/// * `input_source` - Path to the input HTML file or URL (http/https)
/// * `output_path` - Path where the cleaned HTML should be written
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error information
///
/// # Examples
/// ```rust,no_run
/// use smart_crawler::content::clean_html_source;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Clean an HTML file
///     clean_html_source("input.html", "output.html").await?;
///     
///     // Clean HTML from a URL
///     clean_html_source("https://example.com", "output.html").await?;
///     Ok(())
/// }
/// ```
pub async fn clean_html_source(
    input_source: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine if input is URL or file path
    let html_content = if input_source.trim().starts_with("http://")
        || input_source.trim().starts_with("https://")
    {
        // Fetch from URL
        fetch_html_from_url(input_source).await?
    } else {
        // Read from file
        std::fs::read_to_string(input_source)?
    };

    // Clean the HTML
    let cleaned_html = clean_html(&html_content);

    // Write the cleaned HTML to the output file
    std::fs::write(output_path, cleaned_html)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_structured_data_long_form() {
        let html = r#"
            <html>
                <body>
                    <article>
                        <h1>Article Title</h1>
                        <p>This is a long-form article with substantial content. It contains multiple paragraphs and detailed information about a specific topic. The content is rich and informative, providing readers with comprehensive coverage of the subject matter.</p>
                        <p>This second paragraph continues the detailed discussion, adding more depth and analysis to the topic at hand.</p>
                    </article>
                </body>
            </html>
        "#;

        let document = Html::parse_document(html);
        let long_form_content = extract_long_form_content(&document);
        assert!(!long_form_content.is_empty());
        assert!(long_form_content.contains("Article Title"));
        assert!(long_form_content.contains("long-form article"));
    }

    #[tokio::test]
    async fn test_extract_item_list() {
        let html = r#"
            <html>
                <body>
                    <div class="product-item">
                        <h3>Product 1</h3>
                        <p class="description">Description of product 1</p>
                        <a href="/product1">View Details</a>
                        <span class="price">$19.99</span>
                    </div>
                    <div class="product-item">
                        <h3>Product 2</h3>
                        <p class="description">Description of product 2</p>
                        <a href="/product2">View Details</a>
                        <span class="price">$29.99</span>
                    </div>
                    <div class="product-item">
                        <h3>Product 3</h3>
                        <p class="description">Description of product 3</p>
                        <a href="/product3">View Details</a>
                        <span class="price">$39.99</span>
                    </div>
                </body>
            </html>
        "#;

        let document = Html::parse_document(html);
        let items = extract_item_list(&document);
        assert_eq!(items.len(), 3);

        assert_eq!(items[0].title, "Product 1");
        assert_eq!(
            items[0].description,
            Some("Description of product 1".to_string())
        );
        assert_eq!(items[0].url, Some("/product1".to_string()));
        assert_eq!(items[0].metadata.get("price"), Some(&"$19.99".to_string()));
    }

    #[tokio::test]
    async fn test_extract_tabular_data() {
        let html = r#"
            <html>
                <body>
                    <table>
                        <tr>
                            <th>Name</th>
                            <th>Age</th>
                            <th>City</th>
                        </tr>
                        <tr>
                            <td>John</td>
                            <td>25</td>
                            <td>New York</td>
                        </tr>
                        <tr>
                            <td>Jane</td>
                            <td>30</td>
                            <td>Los Angeles</td>
                        </tr>
                    </table>
                </body>
            </html>
        "#;

        let document = Html::parse_document(html);
        let tables = extract_tabular_data(&document);
        assert_eq!(tables.len(), 2); // Two data rows

        assert_eq!(tables[0].get("Name"), Some(&"John".to_string()));
        assert_eq!(tables[0].get("Age"), Some(&"25".to_string()));
        assert_eq!(tables[0].get("City"), Some(&"New York".to_string()));

        assert_eq!(tables[1].get("Name"), Some(&"Jane".to_string()));
        assert_eq!(tables[1].get("Age"), Some(&"30".to_string()));
        assert_eq!(tables[1].get("City"), Some(&"Los Angeles".to_string()));
    }

    #[tokio::test]
    async fn test_extract_main_content() {
        let html = r#"
            <html>
                <body>
                    <nav>Navigation</nav>
                    <header>Header</header>
                    <main>
                        <h1>Main Content Title</h1>
                        <p>This is the main content of the page with substantial text that should be extracted.</p>
                    </main>
                    <footer>Footer</footer>
                </body>
            </html>
        "#;

        let document = Html::parse_document(html);
        let main_content = extract_main_content(&document);
        assert!(!main_content.is_empty());
        assert!(main_content.contains("Main Content Title"));
        assert!(main_content.contains("main content of the page"));
        assert!(!main_content.contains("Navigation"));
        assert!(!main_content.contains("Footer"));
    }

    #[tokio::test]
    async fn test_clean_text() {
        let messy_text = "  This   is   a   messy\n\n\ntext   with   extra   spaces  ";
        let cleaned = clean_text(messy_text);
        assert_eq!(cleaned, "This is a messy text with extra spaces");
    }

    #[tokio::test]
    async fn test_extract_item_title() {
        let html = r#"
            <div class="item">
                <h2>Item Title</h2>
                <p>Some description</p>
            </div>
        "#;

        let document = Html::parse_document(html);
        let element = document
            .select(&Selector::parse(".item").unwrap())
            .next()
            .unwrap();
        let title = extract_item_title(&element);
        assert_eq!(title, "Item Title");
    }

    #[tokio::test]
    async fn test_extract_item_description() {
        let html = r#"
            <div class="item">
                <h2>Item Title</h2>
                <p class="description">This is a detailed description of the item</p>
            </div>
        "#;

        let document = Html::parse_document(html);
        let element = document
            .select(&Selector::parse(".item").unwrap())
            .next()
            .unwrap();

        let description = extract_item_description(&element);
        assert_eq!(
            description,
            Some("This is a detailed description of the item".to_string())
        );
    }

    #[tokio::test]
    async fn test_extract_item_url() {
        let html = r#"
            <div class="item">
                <h2>Item Title</h2>
                <a href="/item/123">View Details</a>
            </div>
        "#;

        let document = Html::parse_document(html);
        let element = document
            .select(&Selector::parse(".item").unwrap())
            .next()
            .unwrap();

        let url = extract_item_url(&element);
        assert_eq!(url, Some("/item/123".to_string()));
    }

    #[tokio::test]
    async fn test_extract_item_metadata() {
        let html = r#"
            <div class="item">
                <h2>Item Title</h2>
                <span class="price">$99.99</span>
                <span class="date">2023-01-01</span>
                <span class="author">John Doe</span>
            </div>
        "#;

        let document = Html::parse_document(html);
        let element = document
            .select(&Selector::parse(".item").unwrap())
            .next()
            .unwrap();

        let metadata = extract_item_metadata(&element);
        assert_eq!(metadata.get("price"), Some(&"$99.99".to_string()));
        assert_eq!(metadata.get("date"), Some(&"2023-01-01".to_string()));
        assert_eq!(metadata.get("author"), Some(&"John Doe".to_string()));
    }

    #[tokio::test]
    async fn test_table_without_headers() {
        let html = r#"
            <html>
                <body>
                    <table>
                        <tr>
                            <td>Value 1</td>
                            <td>Value 2</td>
                        </tr>
                        <tr>
                            <td>Value 3</td>
                            <td>Value 4</td>
                        </tr>
                    </table>
                </body>
            </html>
        "#;

        let document = Html::parse_document(html);

        let tables = extract_tabular_data(&document);
        assert_eq!(tables.len(), 1);

        // Should use first row as headers, so second row becomes data
        assert_eq!(tables[0].get("Value 1"), Some(&"Value 3".to_string()));
        assert_eq!(tables[0].get("Value 2"), Some(&"Value 4".to_string()));
    }

    #[tokio::test]
    async fn test_to_prompt_long_form() {
        let content = "This is a long-form article about technology and innovation.".to_string();
        let structured_data = StructuredContent::LongForm(content);

        let prompt = structured_data.to_prompt();
        assert!(prompt.starts_with("ARTICLE CONTENT:"));
        assert!(prompt.contains("This is a long-form article about technology and innovation."));
    }

    #[tokio::test]
    async fn test_to_prompt_item_list() {
        let mut metadata1 = HashMap::new();
        metadata1.insert("price".to_string(), "$19.99".to_string());
        metadata1.insert("category".to_string(), "Electronics".to_string());

        let mut metadata2 = HashMap::new();
        metadata2.insert("price".to_string(), "$29.99".to_string());

        let items = vec![
            Item {
                title: "Product 1".to_string(),
                description: Some("Great product".to_string()),
                url: Some("/product1".to_string()),
                metadata: metadata1,
            },
            Item {
                title: "Product 2".to_string(),
                description: None,
                url: Some("/product2".to_string()),
                metadata: metadata2,
            },
        ];

        let structured_data = StructuredContent::ItemList(items);
        let prompt = structured_data.to_prompt();

        assert!(prompt.starts_with("ITEM LIST:"));
        assert!(prompt.contains("1. Product 1"));
        assert!(prompt.contains("2. Product 2"));
        assert!(prompt.contains("Description: Great product"));
        assert!(prompt.contains("URL: /product1"));
        assert!(prompt.contains("URL: /product2"));
        assert!(prompt.contains("price: $19.99"));
        assert!(prompt.contains("category: Electronics"));
        assert!(prompt.contains("price: $29.99"));
    }

    #[tokio::test]
    async fn test_to_prompt_tabular_data() {
        let mut row1 = HashMap::new();
        row1.insert("Name".to_string(), "John".to_string());
        row1.insert("Age".to_string(), "25".to_string());
        row1.insert("City".to_string(), "New York".to_string());

        let mut row2 = HashMap::new();
        row2.insert("Name".to_string(), "Jane".to_string());
        row2.insert("Age".to_string(), "30".to_string());
        row2.insert("City".to_string(), "Los Angeles".to_string());

        let tables = vec![row1, row2];
        let structured_data = StructuredContent::TabularData(tables);
        let prompt = structured_data.to_prompt();

        assert!(prompt.starts_with("TABLE DATA:"));
        assert!(prompt.contains("John"));
        assert!(prompt.contains("Jane"));
        assert!(prompt.contains("25"));
        assert!(prompt.contains("30"));
        assert!(prompt.contains("New York"));
        assert!(prompt.contains("Los Angeles"));
        assert!(prompt.contains("|")); // Should contain table formatting
    }

    #[tokio::test]
    async fn test_to_prompt_mixed_content() {
        let long_form_content = "This is the main article content.".to_string();

        let mut metadata = HashMap::new();
        metadata.insert("price".to_string(), "$49.99".to_string());

        let items = vec![Item {
            title: "Featured Item".to_string(),
            description: Some("Special featured product".to_string()),
            url: Some("/featured".to_string()),
            metadata,
        }];

        let mut table_row = HashMap::new();
        table_row.insert("Feature".to_string(), "Value".to_string());
        table_row.insert("Status".to_string(), "Active".to_string());
        let tables = vec![table_row];

        let structured_data = StructuredContent::Mixed {
            long_form: Some(long_form_content),
            items,
            tables,
        };

        let prompt = structured_data.to_prompt();

        assert!(prompt.starts_with("MIXED CONTENT:"));
        assert!(prompt.contains("ARTICLE SECTION:"));
        assert!(prompt.contains("This is the main article content."));
        assert!(prompt.contains("ITEMS SECTION:"));
        assert!(prompt.contains("1. Featured Item"));
        assert!(prompt.contains("Description: Special featured product"));
        assert!(prompt.contains("TABLE SECTION:"));
        assert!(prompt.contains("Feature"));
        assert!(prompt.contains("Value"));
        assert!(prompt.contains("Active"));
    }

    #[tokio::test]
    async fn test_to_prompt_mixed_content_partial() {
        // Test mixed content with only some sections
        let items = vec![Item {
            title: "Only Item".to_string(),
            description: None,
            url: None,
            metadata: HashMap::new(),
        }];

        let structured_data = StructuredContent::Mixed {
            long_form: None,
            items,
            tables: vec![],
        };

        let prompt = structured_data.to_prompt();

        assert!(prompt.starts_with("MIXED CONTENT:"));
        assert!(prompt.contains("ITEMS SECTION:"));
        assert!(prompt.contains("1. Only Item"));
        assert!(!prompt.contains("ARTICLE SECTION:"));
        assert!(!prompt.contains("TABLE SECTION:"));
    }

    #[tokio::test]
    async fn test_to_prompt_empty_item_list() {
        let structured_data = StructuredContent::ItemList(vec![]);
        let prompt = structured_data.to_prompt();

        assert_eq!(prompt, "ITEM LIST:\n\n");
    }

    #[tokio::test]
    async fn test_to_prompt_empty_tabular_data() {
        let structured_data = StructuredContent::TabularData(vec![]);
        let prompt = structured_data.to_prompt();

        assert_eq!(prompt, "TABLE DATA:\n\n");
    }

    #[tokio::test]
    async fn test_to_prompt_item_without_optional_fields() {
        let item = Item {
            title: "Simple Item".to_string(),
            description: None,
            url: None,
            metadata: HashMap::new(),
        };

        let structured_data = StructuredContent::ItemList(vec![item]);
        let prompt = structured_data.to_prompt();

        assert!(prompt.contains("1. Simple Item"));
        assert!(!prompt.contains("Description:"));
        assert!(!prompt.contains("URL:"));
        assert!(!prompt.contains("Metadata:"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_unwanted_elements() {
        let html = r#"
            <html>
                <head>
                    <title>Test Page</title>
                    <script>alert('test');</script>
                    <style>body { color: red; }</style>
                </head>
                <body>
                    <nav class="navbar">Navigation</nav>
                    <header>Header Content</header>
                    <main>
                        <article>
                            <h1>Main Title</h1>
                            <p>Main content here</p>
                        </article>
                    </main>
                    <aside class="sidebar">Sidebar</aside>
                    <footer>Footer Content</footer>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove scripts, styles, nav, header, footer, aside
        assert!(!cleaned.contains("alert('test')"));
        assert!(!cleaned.contains("color: red"));
        assert!(!cleaned.contains("Navigation"));
        assert!(!cleaned.contains("Header Content"));
        assert!(!cleaned.contains("Sidebar"));
        assert!(!cleaned.contains("Footer Content"));

        // Should keep main content
        assert!(cleaned.contains("Main Title"));
        assert!(cleaned.contains("Main content here"));
        assert!(cleaned.contains("<main>"));
        assert!(cleaned.contains("<article>"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_ad_elements() {
        let html = r#"
            <html>
                <body>
                    <div class="content">
                        <h1>Article Title</h1>
                        <p>Article content</p>
                    </div>
                    <div class="ad">Advertisement</div>
                    <div class="advertisement">Another ad</div>
                    <div class="google-ads">Google ads</div>
                    <div class="social-media">Social widgets</div>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove ad-related elements
        assert!(!cleaned.contains("Advertisement"));
        assert!(!cleaned.contains("Another ad"));
        assert!(!cleaned.contains("Google ads"));
        assert!(!cleaned.contains("Social widgets"));

        // Should keep content
        assert!(cleaned.contains("Article Title"));
        assert!(cleaned.contains("Article content"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_form_elements() {
        let html = r#"
            <html>
                <body>
                    <article>
                        <h1>Article</h1>
                        <p>Content</p>
                    </article>
                    <form>
                        <input type="text" name="search">
                        <button>Submit</button>
                        <textarea>Comments</textarea>
                    </form>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove form elements
        assert!(!cleaned.contains("<form>"));
        assert!(!cleaned.contains("<input"));
        assert!(!cleaned.contains("<button>"));
        assert!(!cleaned.contains("<textarea>"));
        assert!(!cleaned.contains("Submit"));

        // Should keep article content
        assert!(cleaned.contains("Article"));
        assert!(cleaned.contains("Content"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_style_attributes() {
        let html = r#"
            <html>
                <body>
                    <div style="color: red; background: blue;">
                        <p style="font-size: 16px;">Styled content</p>
                        <span style='margin: 10px;'>More content</span>
                    </div>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove style attributes
        assert!(!cleaned.contains("color: red"));
        assert!(!cleaned.contains("background: blue"));
        assert!(!cleaned.contains("font-size: 16px"));
        assert!(!cleaned.contains("margin: 10px"));

        // Should keep content
        assert!(cleaned.contains("Styled content"));
        assert!(cleaned.contains("More content"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_event_handlers() {
        let html = r#"
            <html>
                <body>
                    <div onclick="alert('click')" onmouseover="highlight()">
                        <p onload="init()">Content with events</p>
                    </div>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove event handlers
        assert!(!cleaned.contains("onclick"));
        assert!(!cleaned.contains("onmouseover"));
        assert!(!cleaned.contains("onload"));
        assert!(!cleaned.contains("alert('click')"));
        assert!(!cleaned.contains("highlight()"));
        assert!(!cleaned.contains("init()"));

        // Should keep content
        assert!(cleaned.contains("Content with events"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_tracking_attributes() {
        let html = r#"
            <html>
                <body>
                    <div data-track="page-view" data-analytics="event">
                        <p data-ga="click" data-fb="share">Tracked content</p>
                    </div>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove tracking attributes
        assert!(!cleaned.contains("data-track"));
        assert!(!cleaned.contains("data-analytics"));
        assert!(!cleaned.contains("data-ga"));
        assert!(!cleaned.contains("data-fb"));

        // Should keep content
        assert!(cleaned.contains("Tracked content"));
    }

    #[tokio::test]
    async fn test_clean_html_preserves_structural_elements() {
        let html = r#"
            <html>
                <body>
                    <main>
                        <article>
                            <h1>Title</h1>
                            <h2>Subtitle</h2>
                            <p>Paragraph</p>
                            <ul>
                                <li>List item 1</li>
                                <li>List item 2</li>
                            </ul>
                            <table>
                                <tr>
                                    <th>Header</th>
                                    <td>Data</td>
                                </tr>
                            </table>
                            <a href="/link">Link</a>
                            <img src="image.jpg" alt="Description">
                        </article>
                    </main>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should preserve structural elements
        assert!(cleaned.contains("<main>"));
        assert!(cleaned.contains("<article>"));
        assert!(cleaned.contains("<h1>"));
        assert!(cleaned.contains("<h2>"));
        assert!(cleaned.contains("<p>"));
        assert!(cleaned.contains("<ul>"));
        assert!(cleaned.contains("<li>"));
        assert!(cleaned.contains("<table>"));
        assert!(cleaned.contains("<tr>"));
        assert!(cleaned.contains("<th>"));
        assert!(cleaned.contains("<td>"));
        assert!(cleaned.contains("<a href=\"/link\">"));
        assert!(cleaned.contains("<img"));
        assert!(cleaned.contains("src=\"image.jpg\""));
        assert!(cleaned.contains("alt=\"Description\""));

        // Should preserve content
        assert!(cleaned.contains("Title"));
        assert!(cleaned.contains("Subtitle"));
        assert!(cleaned.contains("Paragraph"));
        assert!(cleaned.contains("List item 1"));
        assert!(cleaned.contains("Header"));
        assert!(cleaned.contains("Data"));
        assert!(cleaned.contains("Link"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_problematic_class_ids() {
        let html = r#"
            <html>
                <body>
                    <div class="navbar-content" id="nav-menu">Should be removed</div>
                    <div class="sidebar-widget" id="sidebar-main">Should be removed</div>
                    <div class="ad-container" id="ad-banner">Should be removed</div>
                    <div class="content-main" id="content-area">Should be kept</div>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove problematic class/id attributes
        assert!(!cleaned.contains("navbar-content"));
        assert!(!cleaned.contains("nav-menu"));
        assert!(!cleaned.contains("sidebar-widget"));
        assert!(!cleaned.contains("sidebar-main"));
        assert!(!cleaned.contains("ad-container"));
        assert!(!cleaned.contains("ad-banner"));

        // Content should be preserved even if containers are modified
        assert!(cleaned.contains("Should be kept"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_aria_roles() {
        let html = r#"
            <html>
                <body>
                    <div role="navigation">Navigation</div>
                    <div role="banner">Banner</div>
                    <div role="contentinfo">Content info</div>
                    <div role="complementary">Complementary</div>
                    <div role="main">
                        <p>Main content</p>
                    </div>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove elements with problematic ARIA roles
        assert!(!cleaned.contains("Navigation"));
        assert!(!cleaned.contains("Banner"));
        assert!(!cleaned.contains("Content info"));
        assert!(!cleaned.contains("Complementary"));

        // Should keep main content (role="main" is not in remove list)
        assert!(cleaned.contains("Main content"));
    }

    #[tokio::test]
    async fn test_clean_html_empty_input() {
        let cleaned = clean_html("");
        assert_eq!(cleaned, "");
    }

    #[tokio::test]
    async fn test_clean_html_only_unwanted_content() {
        let html = r#"
            <html>
                <head><script>alert('test');</script></head>
                <body>
                    <nav>Navigation</nav>
                    <footer>Footer</footer>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should result in minimal structure
        assert!(!cleaned.contains("alert"));
        assert!(!cleaned.contains("Navigation"));
        assert!(!cleaned.contains("Footer"));

        // Should still have basic HTML structure
        assert!(cleaned.contains("<html>"));
        assert!(cleaned.contains("<body>"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_empty_tags() {
        let html = r#"
            <html>
                <body>
                    <div></div>
                    <p>Content here</p>
                    <span></span>
                    <h1>Title</h1>
                    <div>  </div>
                    <article>
                        <h2>Article Title</h2>
                        <div></div>
                    </article>
                    <section></section>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove empty tags
        assert!(!cleaned.contains("<div></div>"));
        assert!(!cleaned.contains("<span></span>"));
        assert!(!cleaned.contains("<div>  </div>"));
        assert!(!cleaned.contains("<section></section>"));

        // Should preserve tags with content
        assert!(cleaned.contains("<p>Content here</p>"));
        assert!(cleaned.contains("<h1>Title</h1>"));
        assert!(cleaned.contains("<h2>Article Title</h2>"));
        assert!(cleaned.contains("<article>"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_nested_empty_tags() {
        let html = r#"
            <html>
                <body>
                    <div>
                        <span>
                            <p></p>
                        </span>
                    </div>
                    <article>
                        <h1>Real Content</h1>
                        <div>
                            <span></span>
                        </div>
                    </article>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove all nested empty tags
        assert!(!cleaned.contains("<div>"));
        assert!(!cleaned.contains("<span>"));
        assert!(!cleaned.contains("<p></p>"));

        // Should preserve content-containing tags
        assert!(cleaned.contains("<article>"));
        assert!(cleaned.contains("<h1>Real Content</h1>"));
    }

    #[tokio::test]
    async fn test_clean_html_preserves_important_empty_tags() {
        let html = r#"
            <html>
                <body>
                    <p>Text with line break<br/>more text</p>
                    <hr/>
                    <img src="image.jpg" alt="test"/>
                    <input type="text"/>
                    <div></div>
                    <span></span>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should preserve important self-closing tags
        assert!(cleaned.contains("<br/>"));
        assert!(cleaned.contains("<hr/>"));
        assert!(cleaned.contains("<img"));

        // Should remove empty div and span
        assert!(!cleaned.contains("<div></div>"));
        assert!(!cleaned.contains("<span></span>"));

        // Should preserve content
        assert!(cleaned.contains("Text with line break"));
        assert!(cleaned.contains("more text"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_empty_tags_with_attributes() {
        let html = r#"
            <html>
                <body>
                    <div class="empty-container" id="test"></div>
                    <p class="content">Real content</p>
                    <span style="color: red;"></span>
                    <article data-id="123">
                        <h1>Title</h1>
                    </article>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove empty tags even if they have attributes
        assert!(!cleaned.contains("<div class=\"empty-container\""));
        assert!(!cleaned.contains("<span style="));

        // Should preserve tags with content
        assert!(cleaned.contains("<p"));
        assert!(cleaned.contains("Real content"));
        assert!(cleaned.contains("<article"));
        assert!(cleaned.contains("<h1>Title</h1>"));
    }

    #[tokio::test]
    async fn test_clean_html_handles_whitespace_only_tags() {
        let html = r#"
            <html>
                <body>
                    <div>
                        
                    </div>
                    <p>   </p>
                    <span>
                    
                    </span>
                    <h1>Real Title</h1>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove tags that only contain whitespace
        assert!(!cleaned.contains("<div>"));
        assert!(!cleaned.contains("<p>   </p>"));
        assert!(!cleaned.contains("<span>"));

        // Should preserve tags with actual content
        assert!(cleaned.contains("<h1>Real Title</h1>"));
    }

    #[tokio::test]
    async fn test_clean_html_complex_empty_tag_scenario() {
        let html = r#"
            <html>
                <head>
                    <title>Test</title>
                </head>
                <body>
                    <div class="wrapper">
                        <div class="empty-section">
                            <div></div>
                            <span></span>
                        </div>
                        <div class="content-section">
                            <h1>Main Title</h1>
                            <p>Some content</p>
                            <div class="empty-subsection"></div>
                        </div>
                        <div class="another-empty"></div>
                    </div>
                    <footer></footer>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // After removing empty tags, the wrapper should still exist because it contains content
        // But many nested empty divs should be gone
        assert!(cleaned.contains("<h1>Main Title</h1>"));
        assert!(cleaned.contains("<p>Some content</p>"));

        // Empty sections should be removed (check that empty divs are gone)
        // After cleaning, we should not have standalone empty divs
        let empty_div_count = cleaned.matches("<div></div>").count();
        assert_eq!(empty_div_count, 0, "Should not contain any empty div tags");

        // The main wrapper should remain as it contains actual content
        // (though its class attributes might be removed by other cleaning rules)
    }

    #[tokio::test]
    async fn test_clean_html_removes_link_tags() {
        let html = r#"
            <html>
                <head>
                    <title>Test Page</title>
                    <link rel="stylesheet" href="styles.css">
                    <link rel="icon" href="favicon.ico">
                    <link rel="shortcut icon" href="favicon.ico">
                    <link rel="apple-touch-icon" href="icon.png">
                    <link rel="preload" href="font.woff2" as="font">
                    <link rel="prefetch" href="next-page.html">
                    <link rel="preconnect" href="https://fonts.googleapis.com">
                    <link rel="dns-prefetch" href="//example.com">
                    <link rel="manifest" href="manifest.json">
                    <link rel="canonical" href="https://example.com/page">
                </head>
                <body>
                    <main>
                        <h1>Main Content</h1>
                        <p>This content should remain</p>
                    </main>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove all link tags
        assert!(!cleaned.contains("rel=\"stylesheet\""));
        assert!(!cleaned.contains("rel=\"icon\""));
        assert!(!cleaned.contains("rel=\"shortcut icon\""));
        assert!(!cleaned.contains("rel=\"apple-touch-icon\""));
        assert!(!cleaned.contains("rel=\"preload\""));
        assert!(!cleaned.contains("rel=\"prefetch\""));
        assert!(!cleaned.contains("rel=\"preconnect\""));
        assert!(!cleaned.contains("rel=\"dns-prefetch\""));
        assert!(!cleaned.contains("rel=\"manifest\""));
        assert!(!cleaned.contains("rel=\"canonical\""));
        assert!(!cleaned.contains("<link"));

        // Should preserve main content
        assert!(cleaned.contains("<h1>Main Content</h1>"));
        assert!(cleaned.contains("<p>This content should remain</p>"));
        assert!(cleaned.contains("<main>"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_link_tags_with_type() {
        let html = r#"
            <html>
                <head>
                    <link type="text/css" rel="stylesheet" href="styles.css">
                    <link type="image/x-icon" rel="icon" href="favicon.ico">
                    <link type="image/png" rel="apple-touch-icon" href="icon.png">
                </head>
                <body>
                    <h1>Content</h1>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove link tags with type attributes
        assert!(!cleaned.contains("type=\"text/css\""));
        assert!(!cleaned.contains("type=\"image/x-icon\""));
        assert!(!cleaned.contains("type=\"image/png\""));
        assert!(!cleaned.contains("<link"));

        // Should preserve content
        assert!(cleaned.contains("<h1>Content</h1>"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_navigation_link_tags() {
        let html = "<html><head>\
            <link rel=\"next\" href=\"page2.html\">\
            <link rel=\"prev\" href=\"page0.html\">\
            <link rel=\"first\" href=\"page1.html\">\
            <link rel=\"last\" href=\"page10.html\">\
            <link rel=\"alternate\" type=\"application/rss+xml\" href=\"feed.xml\">\
            <link rel=\"bookmark\" href=\"#section1\">\
            <link rel=\"help\" href=\"help-page.html\">\
            <link rel=\"license\" href=\"license-page.html\">\
            </head><body>\
            <article><h1>Article Title</h1><p>Article content</p></article>\
            </body></html>";

        let cleaned = clean_html(html);

        // Should remove navigation and metadata link tags
        assert!(!cleaned.contains("rel=\"next\""));
        assert!(!cleaned.contains("rel=\"prev\""));
        assert!(!cleaned.contains("rel=\"first\""));
        assert!(!cleaned.contains("rel=\"last\""));
        assert!(!cleaned.contains("rel=\"alternate\""));
        assert!(!cleaned.contains("rel=\"bookmark\""));
        assert!(!cleaned.contains("rel=\"help\""));
        assert!(!cleaned.contains("rel=\"license\""));
        assert!(!cleaned.contains("<link"));

        // Should preserve article content
        assert!(cleaned.contains("<article>"));
        assert!(cleaned.contains("<h1>Article Title</h1>"));
        assert!(cleaned.contains("<p>Article content</p>"));
    }

    #[tokio::test]
    async fn test_clean_html_removes_apple_specific_links() {
        let html = "<html><head>\
            <link rel=\"apple-touch-icon\" sizes=\"180x180\" href=\"apple-touch-icon.png\">\
            <link rel=\"apple-touch-startup-image\" href=\"startup.png\">\
            <link rel=\"mask-icon\" href=\"icon.svg\" color=\"#000000\">\
            <meta name=\"apple-mobile-web-app-title\" content=\"App Name\">\
            </head><body>\
            <h1>App Content</h1>\
            </body></html>";

        let cleaned = clean_html(html);

        // Should remove Apple-specific link tags
        assert!(!cleaned.contains("rel=\"apple-touch-icon\""));
        assert!(!cleaned.contains("rel=\"apple-touch-startup-image\""));
        assert!(!cleaned.contains("rel=\"mask-icon\""));
        assert!(!cleaned.contains("<link"));

        // Should preserve content
        assert!(cleaned.contains("<h1>App Content</h1>"));
    }

    #[tokio::test]
    async fn test_clean_html_comprehensive_link_removal() {
        let html = r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <title>Comprehensive Test</title>
                    <meta charset="utf-8">
                    <link rel="stylesheet" href="main.css">
                    <link rel="stylesheet" href="print.css" media="print">
                    <link rel="icon" type="image/x-icon" href="/favicon.ico">
                    <link rel="apple-touch-icon" sizes="152x152" href="icon-152.png">
                    <link rel="preload" href="critical.css" as="style">
                    <link rel="preload" href="font.woff2" as="font" type="font/woff2" crossorigin>
                    <link rel="prefetch" href="next-page.html">
                    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
                    <link rel="dns-prefetch" href="//analytics.example.com">
                    <link rel="manifest" href="/manifest.json">
                    <link rel="canonical" href="https://example.com/current-page">
                    <link rel="alternate" hreflang="es" href="/es/page">
                    <script>console.log('test');</script>
                </head>
                <body>
                    <header>Site Header</header>
                    <nav>Navigation</nav>
                    <main>
                        <article>
                            <h1>Main Article</h1>
                            <p>Important content that should remain after cleaning.</p>
                        </article>
                    </main>
                    <footer>Site Footer</footer>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove all types of link tags
        assert!(!cleaned.contains("<link"));
        assert!(!cleaned.contains("rel=\"stylesheet\""));
        assert!(!cleaned.contains("rel=\"icon\""));
        assert!(!cleaned.contains("rel=\"preload\""));
        assert!(!cleaned.contains("rel=\"manifest\""));
        assert!(!cleaned.contains("rel=\"canonical\""));

        // Should also remove other unwanted elements
        assert!(!cleaned.contains("<script"));
        assert!(!cleaned.contains("<header"));
        assert!(!cleaned.contains("<nav"));
        assert!(!cleaned.contains("<footer"));

        // Should preserve main content
        assert!(cleaned.contains("<main>"));
        assert!(cleaned.contains("<article>"));
        assert!(cleaned.contains("<h1>Main Article</h1>"));
        assert!(cleaned.contains("<p>Important content that should remain after cleaning.</p>"));
    }

    #[tokio::test]
    async fn test_clean_html_source_with_file() {
        // Create a temporary HTML file for testing
        let temp_dir = std::env::temp_dir();
        let input_file = temp_dir.join("test_input.html");
        let output_file = temp_dir.join("test_output.html");

        let test_html = r#"
            <html>
                <head>
                    <script>alert('test');</script>
                    <style>body { color: red; }</style>
                </head>
                <body>
                    <nav>Navigation</nav>
                    <main>
                        <h1>Test Title</h1>
                        <p>Test content</p>
                    </main>
                    <footer>Footer</footer>
                </body>
            </html>
        "#;

        // Write test HTML to file
        std::fs::write(&input_file, test_html).expect("Failed to write test file");

        // Test cleaning from file
        let result =
            clean_html_source(input_file.to_str().unwrap(), output_file.to_str().unwrap()).await;

        assert!(result.is_ok(), "clean_html_source should succeed");

        // Read and verify output
        let cleaned_content =
            std::fs::read_to_string(&output_file).expect("Failed to read output file");

        assert!(!cleaned_content.contains("alert('test')"));
        assert!(!cleaned_content.contains("color: red"));
        assert!(!cleaned_content.contains("Navigation"));
        assert!(!cleaned_content.contains("Footer"));
        assert!(cleaned_content.contains("Test Title"));
        assert!(cleaned_content.contains("Test content"));

        // Clean up
        let _ = std::fs::remove_file(&input_file);
        let _ = std::fs::remove_file(&output_file);
    }

    #[tokio::test]
    async fn test_clean_html_source_with_invalid_file() {
        let temp_dir = std::env::temp_dir();
        let output_file = temp_dir.join("test_output.html");

        // Test with non-existent file
        let result =
            clean_html_source("/non/existent/file.html", output_file.to_str().unwrap()).await;

        assert!(
            result.is_err(),
            "clean_html_source should fail with non-existent file"
        );

        // Clean up
        let _ = std::fs::remove_file(&output_file);
    }

    #[tokio::test]
    async fn test_fetch_html_from_url_invalid_url() {
        // Initialize crypto provider for browser operations in test
        let _ = rustls::crypto::ring::default_provider().install_default();

        // Test with invalid URL that doesn't exist
        // Note: This test may fail if WebDriver server is not running
        // In production, this would be handled by the browser's error handling
        let result = fetch_html_from_url("https://this-domain-does-not-exist-12345.com").await;

        // The test should fail, but the exact error depends on WebDriver availability
        // If WebDriver is not available, it will fail with "Browser error"
        // If WebDriver is available, it will fail with navigation error
        assert!(
            result.is_err(),
            "fetch_html_from_url should fail with invalid URL"
        );
    }

    // Note: We can't easily test successful URL fetching in unit tests without
    // setting up a mock server, so we'll rely on integration tests for that.
    // The URL detection logic is already tested in cli.rs

    #[tokio::test]
    async fn test_is_slug_like_filename() {
        // Test slug-like filenames (should return true)
        assert!(is_slug_like_filename("my-article-image.jpg"));
        assert!(is_slug_like_filename("product_photo.png"));
        assert!(is_slug_like_filename("hero-banner.webp"));
        assert!(is_slug_like_filename("user-profile-pic.svg"));
        assert!(is_slug_like_filename("company_logo.gif"));
        assert!(is_slug_like_filename("feature-screenshot.jpeg"));

        // Test non-slug filenames (should return false)
        assert!(!is_slug_like_filename("img123.jpg"));
        assert!(!is_slug_like_filename("photo.png"));
        assert!(!is_slug_like_filename("image.gif"));
        assert!(!is_slug_like_filename("banner1.jpg"));
        assert!(!is_slug_like_filename("pic.svg"));
        assert!(!is_slug_like_filename("logo.webp"));

        // Test edge cases
        assert!(!is_slug_like_filename("a-b.jpg")); // Too short
        assert!(!is_slug_like_filename("12345-6.jpg")); // Mostly numbers
        assert!(!is_slug_like_filename("image")); // No extension, no separators
        assert!(!is_slug_like_filename("")); // Empty
    }

    #[tokio::test]
    async fn test_has_meaningful_alt_text() {
        // Test meaningful alt text (should return true)
        assert!(has_meaningful_alt_text("A beautiful sunset"));
        assert!(has_meaningful_alt_text("Company logo"));
        assert!(has_meaningful_alt_text("User profile picture"));
        assert!(has_meaningful_alt_text("Product photo showing features"));
        assert!(has_meaningful_alt_text("123 Main Street building"));

        // Test non-meaningful alt text (should return false)
        assert!(!has_meaningful_alt_text(""));
        assert!(!has_meaningful_alt_text("   "));
        assert!(!has_meaningful_alt_text("ab")); // Too short
        assert!(!has_meaningful_alt_text("123")); // Only numbers
        assert!(!has_meaningful_alt_text("   a  ")); // Too short after trim
    }

    #[tokio::test]
    async fn test_filter_images_keep_slug_like() {
        let html = r#"
            <div>
                <p>Some content</p>
                <img src="/images/hero-banner.jpg" width="800" height="400">
                <img src="/photos/product_photo.png" alt="">
                <img src="/assets/company-logo.svg" class="logo">
                <p>More content</p>
            </div>
        "#;

        let filtered = filter_images(html);

        // Should keep images with slug-like filenames
        assert!(filtered.contains("hero-banner.jpg"));
        assert!(filtered.contains("product_photo.png"));
        assert!(filtered.contains("company-logo.svg"));
        assert!(filtered.contains("<img"));
    }

    #[tokio::test]
    async fn test_filter_images_keep_meaningful_alt() {
        let html = r#"
            <div>
                <p>Some content</p>
                <img src="/img123.jpg" alt="Company headquarters building">
                <img src="/photo.png" alt="User profile picture">
                <img src="/banner1.gif" alt="Welcome message">
                <p>More content</p>
            </div>
        "#;

        let filtered = filter_images(html);

        // Should keep images with meaningful alt text
        assert!(filtered.contains("alt=\"Company headquarters building\""));
        assert!(filtered.contains("alt=\"User profile picture\""));
        assert!(filtered.contains("alt=\"Welcome message\""));
        assert!(filtered.contains("<img"));
    }

    #[tokio::test]
    async fn test_filter_images_remove_low_value() {
        let html = r#"
            <div>
                <p>Some content</p>
                <img src="/img123.jpg">
                <img src="/photo.png" alt="">
                <img src="/banner1.gif" alt="   ">
                <img src="/image.webp" alt="ab">
                <p>More content</p>
            </div>
        "#;

        let filtered = filter_images(html);

        // Should remove images without slug-like names or meaningful alt text
        assert!(!filtered.contains("img123.jpg"));
        assert!(!filtered.contains("photo.png"));
        assert!(!filtered.contains("banner1.gif"));
        assert!(!filtered.contains("image.webp"));
        assert!(!filtered.contains("<img"));
    }

    #[tokio::test]
    async fn test_filter_images_mixed_scenario() {
        let html = r#"
            <div>
                <img src="/hero-banner.jpg" alt="">  <!-- Keep: slug-like name -->
                <img src="/img123.jpg" alt="Product photo">  <!-- Keep: meaningful alt -->
                <img src="/photo.png" alt="">  <!-- Remove: neither -->
                <img src="/company_logo.svg" alt="Our logo">  <!-- Keep: both -->
                <img src="/banner1.gif">  <!-- Remove: neither -->
            </div>
        "#;

        let filtered = filter_images(html);

        // Should keep first, second, and fourth images
        assert!(filtered.contains("hero-banner.jpg"));
        assert!(filtered.contains("alt=\"Product photo\""));
        assert!(filtered.contains("company_logo.svg"));

        // Should remove third and fifth images
        assert!(!filtered.contains("photo.png"));
        assert!(!filtered.contains("banner1.gif"));

        // Should have exactly 3 img tags
        let img_count = filtered.matches("<img").count();
        assert_eq!(img_count, 3);
    }

    #[tokio::test]
    async fn test_clean_html_removes_comments() {
        let html = r#"
            <html>
                <!-- This is a comment -->
                <head>
                    <title>Test Page</title>
                    <!-- TODO: Add more meta tags -->
                </head>
                <body>
                    <!-- Begin main content -->
                    <main>
                        <h1>Title</h1>
                        <!-- This content is great -->
                        <p>Content here</p>
                    </main>
                    <!-- End main content -->
                </body>
                <!-- Footer comment -->
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove all HTML comments
        assert!(!cleaned.contains("<!-- This is a comment -->"));
        assert!(!cleaned.contains("<!-- TODO: Add more meta tags -->"));
        assert!(!cleaned.contains("<!-- Begin main content -->"));
        assert!(!cleaned.contains("<!-- This content is great -->"));
        assert!(!cleaned.contains("<!-- End main content -->"));
        assert!(!cleaned.contains("<!-- Footer comment -->"));
        assert!(!cleaned.contains("<!--"));
        assert!(!cleaned.contains("-->"));

        // Should preserve actual content
        assert!(cleaned.contains("<h1>Title</h1>"));
        assert!(cleaned.contains("<p>Content here</p>"));
        assert!(cleaned.contains("<main>"));
    }

    #[tokio::test]
    async fn test_clean_html_filters_images() {
        let html = r#"
            <html>
                <body>
                    <article>
                        <h1>Article Title</h1>
                        <img src="/hero-image.jpg" alt="">  <!-- Keep: slug-like -->
                        <p>Article content</p>
                        <img src="/img123.jpg" alt="Important diagram">  <!-- Keep: meaningful alt -->
                        <img src="/photo.png" alt="">  <!-- Remove: neither -->
                        <img src="/company_logo.svg" alt="Logo">  <!-- Keep: both -->
                        <img src="/banner1.gif">  <!-- Remove: neither -->
                    </article>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should keep meaningful images
        assert!(cleaned.contains("hero-image.jpg"));
        assert!(cleaned.contains("alt=\"Important diagram\""));
        assert!(cleaned.contains("company_logo.svg"));

        // Should remove low-value images
        assert!(!cleaned.contains("photo.png"));
        assert!(!cleaned.contains("banner1.gif"));

        // Should preserve other content
        assert!(cleaned.contains("<h1>Article Title</h1>"));
        assert!(cleaned.contains("<p>Article content</p>"));
        assert!(cleaned.contains("<article>"));
    }

    #[tokio::test]
    async fn test_clean_html_combined_improvements() {
        let html = r#"
            <html>
                <!-- Page comment -->
                <head>
                    <title>Test Page</title>
                    <script>alert('test');</script>
                    <!-- Meta comment -->
                </head>
                <body>
                    <!-- Content starts here -->
                    <main>
                        <h1>Article</h1>
                        <img src="/hero-banner.jpg" alt="">  <!-- Keep -->
                        <p>Content with meaning</p>
                        <img src="/ad123.jpg">  <!-- Remove -->
                        <!-- More content below -->
                        <img src="/diagram.png" alt="Important chart">  <!-- Keep -->
                    </main>
                    <!-- Content ends here -->
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);

        // Should remove all comments
        assert!(!cleaned.contains("<!--"));
        assert!(!cleaned.contains("-->"));

        // Should remove scripts
        assert!(!cleaned.contains("<script"));

        // Should filter images appropriately
        assert!(cleaned.contains("hero-banner.jpg")); // Slug-like name
        assert!(cleaned.contains("alt=\"Important chart\"")); // Meaningful alt
        assert!(!cleaned.contains("ad123.jpg")); // Neither condition met

        // Should preserve content
        assert!(cleaned.contains("<h1>Article</h1>"));
        assert!(cleaned.contains("<p>Content with meaning</p>"));
        assert!(cleaned.contains("<main>"));
    }

    #[tokio::test]
    async fn test_filter_images_svg_keep_with_aria_label() {
        let html = r#"
            <html>
                <body>
                    <svg aria-label="Company logo" width="100" height="50">
                        <circle cx="50" cy="25" r="20" fill="blue"/>
                    </svg>
                </body>
            </html>
        "#;

        let cleaned = filter_images(html);

        // Should keep SVG with aria-label
        assert!(cleaned.contains("<svg aria-label=\"Company logo\""));
        assert!(cleaned.contains("</svg>"));
    }

    #[tokio::test]
    async fn test_filter_images_svg_keep_with_title() {
        let html = r#"
            <html>
                <body>
                    <svg title="Data visualization" width="200" height="100">
                        <rect width="200" height="100" fill="green"/>
                    </svg>
                </body>
            </html>
        "#;

        let cleaned = filter_images(html);

        // Should keep SVG with title attribute
        assert!(cleaned.contains("<svg title=\"Data visualization\""));
        assert!(cleaned.contains("</svg>"));
    }

    #[tokio::test]
    async fn test_filter_images_svg_keep_with_meaningful_class() {
        let html = r#"
            <html>
                <body>
                    <svg class="icon-chart" width="50" height="50">
                        <path d="M10,10 L40,40"/>
                    </svg>
                    <svg class="company-logo" width="100" height="50">
                        <text x="10" y="30">Logo</text>
                    </svg>
                </body>
            </html>
        "#;

        let cleaned = filter_images(html);

        // Should keep SVGs with meaningful class names
        assert!(cleaned.contains("class=\"icon-chart\""));
        assert!(cleaned.contains("class=\"company-logo\""));
    }

    #[tokio::test]
    async fn test_filter_images_svg_keep_with_meaningful_id() {
        let html = r#"
            <html>
                <body>
                    <svg id="diagram-flow" width="300" height="200">
                        <circle cx="150" cy="100" r="50"/>
                    </svg>
                </body>
            </html>
        "#;

        let cleaned = filter_images(html);

        // Should keep SVG with meaningful id
        assert!(cleaned.contains("id=\"diagram-flow\""));
        assert!(cleaned.contains("</svg>"));
    }

    #[tokio::test]
    async fn test_filter_images_svg_remove_decorative() {
        let html = r#"
            <html>
                <body>
                    <svg width="20" height="20">
                        <circle cx="10" cy="10" r="5" fill="red"/>
                    </svg>
                    <svg class="btn-icon" width="16" height="16">
                        <path d="M8,8 L12,12"/>
                    </svg>
                </body>
            </html>
        "#;

        let cleaned = filter_images(html);

        // Should remove decorative SVGs without meaningful attributes
        assert!(!cleaned.contains("<svg width=\"20\""));
        assert!(!cleaned.contains("class=\"btn-icon\""));
        assert!(!cleaned.contains("</svg>"));
    }

    #[tokio::test]
    async fn test_filter_images_svg_self_closing() {
        let html = r#"
            <html>
                <body>
                    <svg aria-label="Icon" width="24" height="24"/>
                    <svg class="generic" width="16" height="16"/>
                </body>
            </html>
        "#;

        let cleaned = filter_images(html);

        // Should keep meaningful self-closing SVG
        assert!(cleaned.contains("aria-label=\"Icon\""));

        // Should remove decorative self-closing SVG
        assert!(!cleaned.contains("class=\"generic\""));
    }

    #[tokio::test]
    async fn test_filter_images_mixed_img_and_svg() {
        let html = r#"
            <html>
                <body>
                    <main>
                        <h1>Article</h1>
                        <!-- Keep: Image with slug name -->
                        <img src="/hero-banner.jpg" width="800" height="400">
                        
                        <!-- Keep: SVG with aria-label -->
                        <svg aria-label="Chart showing growth" width="400" height="200">
                            <rect width="100" height="150" fill="blue"/>
                        </svg>
                        
                        <!-- Remove: Image without meaningful attributes -->
                        <img src="/photo123.png" alt="">
                        
                        <!-- Remove: Decorative SVG -->
                        <svg width="10" height="10">
                            <circle cx="5" cy="5" r="3"/>
                        </svg>
                        
                        <!-- Keep: Image with meaningful alt -->
                        <img src="/tmp456.jpg" alt="Product demonstration">
                        
                        <!-- Keep: SVG with meaningful class -->
                        <svg class="logo-main" width="150" height="75">
                            <text x="10" y="40">Company</text>
                        </svg>
                        
                        <p>Article content</p>
                    </main>
                </body>
            </html>
        "#;

        let cleaned = filter_images(html);

        // Should keep meaningful images
        assert!(cleaned.contains("hero-banner.jpg"));
        assert!(cleaned.contains("alt=\"Product demonstration\""));

        // Should keep meaningful SVGs
        assert!(cleaned.contains("aria-label=\"Chart showing growth\""));
        assert!(cleaned.contains("class=\"logo-main\""));

        // Should remove low-value images
        assert!(!cleaned.contains("photo123.png"));

        // Should remove decorative SVGs
        assert!(!cleaned.contains("width=\"10\" height=\"10\""));

        // Should preserve content structure
        assert!(cleaned.contains("<h1>Article</h1>"));
        assert!(cleaned.contains("<p>Article content</p>"));
    }
}
