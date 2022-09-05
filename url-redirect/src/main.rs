mod prisma;

#[tokio::main]
async fn main() {
    let client = prisma::new_client().await.unwrap();
    use prisma::url;
    // TODO: Don't unwrap
    let entry_result: Result<Option<url::Data>, prisma_client_rust::QueryError> = client
        .url() // Model to query on
        .find_unique(prisma::url::slug::equals("yessir".to_string())) // Query to execute
        .exec() // Ends query
        .await; // All queries are async and return Result
    
    let entry_data: Option<url::Data> = match entry_result{
        Ok(entry) => entry,
        Err(e)=> panic!("Failed to find entry, {}",e)
    };
    
    let found_entry = match entry_data{
        Some(data)=>data,
        None => panic!("Couldn't find data in entry")
    };

    println!("Found! {}, {}", found_entry.slug, found_entry.url);
    
    // let url = client
    // .url()
    // .create("aalhendi.com".to_string(), "yessir".to_string(), vec![])
    // .exec()
    // .await
    // .unwrap();

    // println!("Hello, the url is {}, the slug is {}", url.url, url.slug)

}
