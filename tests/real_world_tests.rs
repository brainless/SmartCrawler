use smart_crawler::{Browser, HtmlParser, UrlStorage, FetchStatus};
use tokio;

/// Helper function to process a URL and return the parsed HTML tree
async fn crawl_and_parse(url: &str) -> Result<smart_crawler::HtmlNode, String> {
    // Initialize crypto provider for rustls (required for HTTPS connections)
    let _ = rustls::crypto::ring::default_provider().install_default();
    
    let mut browser = Browser::new(4444);
    let parser = HtmlParser::new();
    let mut storage = UrlStorage::new();
    
    // Add URL to storage
    storage.add_url(url.to_string());
    
    // Connect to WebDriver
    browser.connect().await.map_err(|e| format!("Failed to connect to WebDriver: {}", e))?;
    
    // Navigate and get HTML source
    browser.navigate_to(url).await.map_err(|e| format!("Failed to navigate to {}: {}", url, e))?;
    let html_source = browser.get_html_source().await.map_err(|e| format!("Failed to get HTML source: {}", e))?;
    let title = browser.get_page_title().await.ok();
    
    // Parse HTML
    let html_tree = parser.parse(&html_source);
    
    // Update storage
    if let Some(url_data) = storage.get_url_data_mut(url) {
        url_data.set_html_data(html_source, html_tree.clone(), title);
        url_data.update_status(FetchStatus::Success);
    }
    
    // Clean up
    let _ = browser.close().await;
    
    Ok(html_tree)
}

#[tokio::test]
#[ignore] // Ignored by default, run with: cargo test real_world -- --ignored
async fn test_hacker_news_submissions() {
    println!("Testing Hacker News for 30 submission elements...");
    
    match crawl_and_parse("https://news.ycombinator.com/").await {
        Ok(tree) => {
            // Find all submission elements using the specified path
            let submissions = tree.find_by_path("html body center table tbody tr td table tbody tr.athing.submission td.title");
            
            println!("Found {} submission elements", submissions.len());
            
            // Print details of found submissions for debugging
            for (i, submission) in submissions.iter().enumerate() {
                println!("Submission {}: '{}'", i + 1, submission.content.chars().take(50).collect::<String>());
            }
            
            if submissions.len() != 30 {
                println!("❌ Expected 30 submission elements, but found {}", submissions.len());
                println!("This test failed. Please manually verify the webpage structure.");
                println!("The path used was: html body center table tbody tr td table tbody tr.athing.submission td.title");
                panic!("Expected 30 submission elements, found {}", submissions.len());
            } else {
                println!("✅ Successfully found exactly 30 submission elements!");
            }
        }
        Err(e) => {
            println!("❌ Failed to crawl Hacker News: {}", e);
            println!("Please verify that:");
            println!("1. A WebDriver server is running on port 4444");
            println!("2. The website https://news.ycombinator.com/ is accessible");
            panic!("Failed to crawl: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Ignored by default, run with: cargo test real_world -- --ignored
async fn test_mykin_ai_team_member() {
    println!("Testing Mykin.ai for Kasper Juul team member...");
    
    match crawl_and_parse("https://mykin.ai/company").await {
        Ok(tree) => {
            // Find the team member element using the specified path
            let team_members = tree.find_by_path("html.w-mod-js.w-mod-ix body div.page-wrapper main.main-wrapper section.section_team div.padding-global div.container-medium div.team_collection.is-desktop.w-dyn-list div.team_collection-list.w-dyn-items div.w-dyn-item a.team_card.w-inline-block div.team_content h4");
            
            println!("Found {} team member elements", team_members.len());
            
            // Print all team members found
            for (i, member) in team_members.iter().enumerate() {
                println!("Team member {}: '{}'", i + 1, member.content);
            }
            
            // Look for Kasper Juul specifically
            let kasper_found = team_members.iter().any(|member| member.content.contains("Kasper Juul"));
            
            if !kasper_found {
                println!("❌ Could not find 'Kasper Juul' in team members");
                println!("This test failed. Please manually verify the webpage structure.");
                println!("The path used was: html.w-mod-js.w-mod-ix body div.page-wrapper main.main-wrapper section.section_team div.padding-global div.container-medium div.team_collection.is-desktop.w-dyn-list div.team_collection-list.w-dyn-items div.w-dyn-item a.team_card.w-inline-block div.team_content h4");
                panic!("Could not find Kasper Juul in team members");
            } else {
                println!("✅ Successfully found 'Kasper Juul' in team members!");
            }
        }
        Err(e) => {
            println!("❌ Failed to crawl Mykin.ai: {}", e);
            println!("Please verify that:");
            println!("1. A WebDriver server is running on port 4444");
            println!("2. The website https://mykin.ai/company is accessible");
            panic!("Failed to crawl: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Ignored by default
async fn test_webdriver_connection() {
    println!("Testing WebDriver connection...");
    
    // Initialize crypto provider for rustls
    let _ = rustls::crypto::ring::default_provider().install_default();
    
    let mut browser = Browser::new(4444);
    match browser.connect().await {
        Ok(()) => {
            println!("✅ Successfully connected to WebDriver");
            let _ = browser.close().await;
        }
        Err(e) => {
            println!("❌ Failed to connect to WebDriver: {}", e);
            println!("Please ensure a WebDriver server is running on port 4444");
            println!("Setup options:");
            println!("   • GeckoDriver: geckodriver (uses port 4444 by default)");
            println!("   • ChromeDriver: chromedriver --port=4444");
            println!("   • Docker: docker run -d -p 4444:4444 selenium/standalone-chrome:latest");
            panic!("WebDriver connection failed: {}", e);
        }
    }
}