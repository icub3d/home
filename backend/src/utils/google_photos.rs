use regex::Regex;
use reqwest::Client;
use std::collections::HashSet;

pub async fn extract_photos(url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()?;

    let res = client.get(url).send().await?;
    let html = res.text().await?;

    // Regex to find Google Photos URLs.
    // We look for lh*.googleusercontent.com/pw/[ID]
    // The ID is alphanumeric with dashes and underscores.
    // This targets the "Photos Web" direct links found in shared album pages.
    let re = Regex::new(r#"https://lh[0-9]+\.googleusercontent\.com/pw/[a-zA-Z0-9\-_]+"#)?;

    let mut links = HashSet::new();

    for cap in re.captures_iter(&html) {
        if let Some(match_str) = cap.get(0) {
            let url = match_str.as_str();
            // Google Photos often returns base URLs. We can append parameters to size them.
            // =w1920-h1080-no would request 1920x1080.
            // We store the base URL with these params.
            links.insert(format!("{}=w1920-h1080-no", url));
        }
    }

    let mut final_links: Vec<String> = links.into_iter().collect();
    
    // Sort for consistency if needed, or shuffle later.
    final_links.sort();

    Ok(final_links)
}
