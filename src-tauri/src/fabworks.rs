use std::{path::PathBuf, fs::File, io::Read};
use reqwest::multipart;
use serde::Deserialize;
use tokio::fs;
use walkdir::WalkDir;

#[derive(Deserialize)]
struct FWIDResponse {
    id: String
}

pub(crate) async fn list_fw_files(folderpath: String) -> Vec<String> {
    WalkDir::new(folderpath.clone())
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_type().is_dir())
        .filter(|entry| {
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            file_name.ends_with(".stp") || file_name.ends_with(".step") || file_name.ends_with(".dxf")
        })
        .map(|entry| {
            let relative_path = entry.path().strip_prefix(folderpath.clone()).unwrap();
            relative_path.to_str().unwrap().to_string()
        })
        .collect()
}

pub(crate) async fn push_to_fw(folderpath: String, files: Vec<String>) -> Result<(), reqwest::Error> {
    let create_url = "https://www.fabworks.com/api/quotes/create";
    let create_response = reqwest::get(create_url).await?;
    
    // Parse the JSON response and extract the "id" value
    let create_json: FWIDResponse = create_response.json().await?;
    let create_id = create_json.id;
    
    for file_path in files {
        let full_path = PathBuf::from(&folderpath).join(&file_path);

        let upload_url = "https://www.fabworks.com/api/quotes/upload";
        let form = reqwest::blocking::multipart::Form::new()
            .text("id", create_id.clone())
            .file("file", full_path).unwrap();

        tokio::task::spawn_blocking(move || {
            let client = reqwest::blocking::Client::new();
            let response = client
                .post(upload_url)
                .multipart(form)
                .send().unwrap();

            if response.status().is_success() {
                println!("File '{}' uploaded successfully", file_path);
            } else {
                eprintln!("Failed to upload file '{}': {}", file_path, response.status());
            }
        }).await.unwrap();
    }

    let url = format!("https://www.fabworks.com/quotes/{}", create_id.clone());
    webbrowser::open(&url).unwrap();

    Ok(())
}