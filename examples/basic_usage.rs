use gummy_search::GummySearchClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = GummySearchClient::new("http://localhost:9200");

    println!("Gummy Search Client created");
    println!("Base URL: http://localhost:9200");

    // TODO: Implement actual operations
    // Example usage:
    // client.create_index("test_index", settings).await?;
    // client.index_document("test_index", "1", &document).await?;
    // let results = client.search("test_index", query).await?;

    Ok(())
}
