#[cfg(feature = "urls")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "http://api.indexnow.org/IndexNow";
    let json_data = r#"
{
    "host": "hnefatafl.org",
    "key": "514969c804234582abafaae69c947790",
    "keyLocation": "https://hnefatafl.org/514969c804234582abafaae69c947790.txt",
    "urlList": [
        "https://hnefatafl.org",
        "https://hnefatafl.org/install.html",
        "https://hnefatafl.org/rules.html",
        "https://hnefatafl.org/sitemap.xml"
    ]
}
"#;

    let client = reqwest::blocking::Client::new();

    let response = client
        .post(url)
        .header("Content-Type", "application/json; charset=utf-8")
        .body(json_data.to_string())
        .send()?;

    println!("Status: {:?}", response);

    let response_body = response.text()?;
    println!("Response body:\n{}", response_body);

    Ok(())
}

#[cfg(not(feature = "urls"))]
fn main() {
    println!("You didn't enable urls.");
}
