use std::{path::{Path, PathBuf, Component}, fs::{File, self}, io::Read, sync::Arc, collections::HashMap};

use google_drive::{Client, AccessToken, traits::FileOps};
use tokio::sync::{Semaphore, Mutex};
use vfs::{VfsPath, MemoryFS};

pub(crate) async fn gd_get_sync(
    folder_id: &str,
    client: &Client,
    tokens: &AccessToken,
) -> String {

    let mut query = format!("name contains '.sync'");
    query = format!("{} and '{}' in parents and trashed = false", query, folder_id);

    let files = client.files().list_all(
        "allDrives",  // corpora
        "", // drive id
        true,     // include_items_from_all_drives
        "",       // include_permissions_for_view
        false,    // include_team_drive_items
        "",       // order_by
        &query,   // query
        "",       // spaces
        true,     // supports_all_drives
        false,    // supports_team_drives
        "",       // team_drive_id
    )
    .await;

    let filesvec = files.unwrap().body;

    if filesvec.is_empty() {
        return "".to_string();
    }else{
        let fl = filesvec.get(0).unwrap();

        return fl.name.clone();
    }
}

pub(crate) async fn upload_files_to_google_drive(
    files: Vec<Box<Path>>,
    folder_path: &str,
    folder_id: &str,
    client: &Client,
    tokens: &AccessToken,
    override_name: Option<String>
) {
    let drive_id = &client.files().get(folder_id, false, "published", true, true).await.unwrap().body.drive_id;
    let mut folder_id_tmp: String = folder_id.to_string();
    for file in files {
        let file_path = file.clone();
        let relative_path = file_path.strip_prefix(Path::new(folder_path)).unwrap_or(&(*file_path));
        let full_destination_path = PathBuf::from(folder_id_tmp.clone()).join(relative_path);
        let mut current_parent_id = folder_id_tmp.to_string();

        if let Some(file_name) = file_path.file_name() {
            let file_name_str = file_name.to_str().unwrap();

            for component in relative_path.parent().unwrap_or(Path::new("")).components() {
                let component_str = component.as_os_str().to_str().unwrap();
                //let result = client.files().create_folder(drive_id, &current_parent_id, component_str).await.unwrap();
                
                let folder_mime_type = "application/vnd.google-apps.folder";
                let mut file: google_drive::types::File = Default::default();
                // Set the name,
                file.name = component_str.to_string();
                file.mime_type = folder_mime_type.to_string();
                file.parents = vec![current_parent_id.clone()];

                let mut query = format!(
                    "name = '{}' and mimeType = 'application/vnd.google-apps.folder'",
                    component_str
                );
                query = format!("{} and '{}' in parents and trashed = false", query, current_parent_id.clone());
        
                // Check if the folder exists.
                let resp = client.files()
                    .list_all(
                        "allDrives",  // corpora
                        "", // drive id
                        true,     // include_items_from_all_drives
                        "",       // include_permissions_for_view
                        false,    // include_team_drive_items
                        "",       // order_by
                        &query,   // query
                        "",       // spaces
                        true,     // supports_all_drives
                        false,    // supports_team_drives
                        "",       // team_drive_id
                    )
                    .await.unwrap();
        
                if !resp.body.is_empty() {
                    current_parent_id = resp.body.get(0).unwrap().id.clone();
                }else{
                    let res = client.files().create(false, "published", false, "en", true, true, false, &file).await.unwrap();
                    current_parent_id = res.body.id;
                }

                println!("RE: {:?}", resp.body);

                // Use the created or existing folder ID for the next iterations
                //current_parent_id = result.body.id;
            }

            // Check if the file is a folder
            if file_path.is_dir() {
                // Create the folder in Google Drive
                
            } else {
                
            }

            // Read the file contents
            let mut file_contents = Vec::new();
            let _ = File::open(file.clone()).unwrap()
                .read_to_end(&mut file_contents);

            let name_final = override_name.clone().unwrap_or(file_name_str.to_owned());
            
            let mut query = format!("name = '{}'", name_final);
            query = format!("{} and '{}' in parents and trashed = false", query, current_parent_id);

            let files = client.files().list_all(
                "allDrives",  // corpora
                drive_id, // drive id
                true,     // include_items_from_all_drives
                "",       // include_permissions_for_view
                false,    // include_team_drive_items
                "",       // order_by
                &query,   // query
                "",       // spaces
                true,     // supports_all_drives
                false,    // supports_team_drives
                "",       // team_drive_id
            )
            .await;

            let file = files.unwrap();

            println!("TEST {} {:?}", current_parent_id, file.body);

            if file.body.len() != 0 {
                for fl in file.body.clone() {
                    println!("{:?}", fl.parents);
                }
                let file_bd = file.body.get(0).unwrap();

                let client = reqwest::Client::new();

                let mut req: reqwest::RequestBuilder = client.patch(format!("https://www.googleapis.com/upload/drive/v3/files/{}", &file_bd.id));
                req = req.header("Authorization", format!("Bearer {}", tokens.access_token));
                req = req.header("Content-Length", file_contents.len().to_string());
                req = req.header("Content-Type", "application/octet-stream");
                req = req.query(&[("uploadType", "media")]);
                req = req.body(file_contents);
                let fullreq = req.build().unwrap();
                let response = client.execute(fullreq).await.unwrap();
                println!("{}", response.text().await.unwrap());
            }else{
                client.files().create_or_update(
                    "",
                    &current_parent_id,
                    &name_final,
                    "application/octet-stream", // Replace with the actual mime type
                    &file_contents,
                ).await.unwrap();
            }
        }
    }
}

pub(crate) async fn gd_get_file(
    files_name: &str,
    folder_id: &str,
    client: &Client,
    tokens: &AccessToken,
) -> Option<Vec<u8>> {

    let mut parentid = folder_id.to_string();

    let mut fl_name = files_name;

    if files_name.contains("/") {
        let path = Path::new(files_name);
        fl_name = path.file_name().unwrap().to_str().unwrap();

        for component in path.parent().unwrap_or(Path::new("")).components() {
            let component_str = component.as_os_str().to_str().unwrap();
            //let result = client.files().create_folder(drive_id, &current_parent_id, component_str).await.unwrap();
            
            let folder_mime_type = "application/vnd.google-apps.folder";
            let mut file: google_drive::types::File = Default::default();
            // Set the name,
            file.name = component_str.to_string();
            file.mime_type = folder_mime_type.to_string();
            file.parents = vec![parentid.to_string()];

            let mut query = format!(
                "name = '{}' and mimeType = 'application/vnd.google-apps.folder'",
                component_str
            );
            query = format!("{} and '{}' in parents and trashed = false", query, parentid.clone());
    
            // Check if the folder exists.
            let resp = client.files()
                .list_all(
                    "allDrives",  // corpora
                    "", // drive id
                    true,     // include_items_from_all_drives
                    "",       // include_permissions_for_view
                    false,    // include_team_drive_items
                    "",       // order_by
                    &query,   // query
                    "",       // spaces
                    true,     // supports_all_drives
                    false,    // supports_team_drives
                    "",       // team_drive_id
                )
                .await.unwrap();
    
            if !resp.body.is_empty() {
                parentid = resp.body.get(0).unwrap().id.clone();
            }else{
                let res = client.files().create(false, "published", false, "en", true, true, false, &file).await.unwrap();
                parentid = res.body.id;
            }
            println!("RE: {:?}", resp.body);
        }
    }

    let mut query = format!("name = '{}'", fl_name);
    query = format!("{} and '{}' in parents and trashed = false", query, parentid);

    let files = client.files().list_all(
        "allDrives",  // corpora
        "", // drive id
        true,     // include_items_from_all_drives
        "",       // include_permissions_for_view
        false,    // include_team_drive_items
        "",       // order_by
        &query,   // query
        "",       // spaces
        true,     // supports_all_drives
        false,    // supports_team_drives
        "",       // team_drive_id
    )
    .await;

    let filesvec = files.unwrap().body;

    if filesvec.is_empty() {
        return Some(Vec::new());
    }else{
        let fl = filesvec.get(0).unwrap();

        println!("{}", fl.mime_type);

        if fl.mime_type == "application/vnd.google-apps.folder" {
            return None;
        }

        let mut link = format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", fl.id);

        let client = reqwest::Client::new();
        let req = client.get(link.clone()).header("Authorization", format!("Bearer {}", tokens.access_token)).send().await.unwrap();

        let body = req.bytes().await;
        return Some(body.unwrap().to_vec());
    }
}

pub(crate) async fn gd_delete_file(
    files_name: &str,
    folder_id: &str,
    client: &Client,
    tokens: &AccessToken,
) {

    let mut parentid = folder_id.to_string();

    let mut fl_name = files_name;

    if files_name.contains("/") {
        let path = Path::new(files_name);
        fl_name = path.file_name().unwrap().to_str().unwrap();

        for component in path.parent().unwrap_or(Path::new("")).components() {
            let component_str = component.as_os_str().to_str().unwrap();
            //let result = client.files().create_folder(drive_id, &current_parent_id, component_str).await.unwrap();
            
            let folder_mime_type = "application/vnd.google-apps.folder";
            let mut file: google_drive::types::File = Default::default();
            // Set the name,
            file.name = component_str.to_string();
            file.mime_type = folder_mime_type.to_string();
            file.parents = vec![parentid.to_string()];

            let mut query = format!(
                "name = '{}' and mimeType = 'application/vnd.google-apps.folder'",
                component_str
            );
            query = format!("{} and '{}' in parents and trashed = false", query, parentid.clone());
    
            // Check if the folder exists.
            let resp = client.files()
                .list_all(
                    "allDrives",  // corpora
                    "", // drive id
                    true,     // include_items_from_all_drives
                    "",       // include_permissions_for_view
                    false,    // include_team_drive_items
                    "",       // order_by
                    &query,   // query
                    "",       // spaces
                    true,     // supports_all_drives
                    false,    // supports_team_drives
                    "",       // team_drive_id
                )
                .await.unwrap();
    
            if !resp.body.is_empty() {
                parentid = resp.body.get(0).unwrap().id.clone();
            }else{
                let res = client.files().create(false, "published", false, "en", true, true, false, &file).await.unwrap();
                parentid = res.body.id;
            }
            println!("RE: {:?}", resp.body);
        }
    }

    let mut query = format!("name = '{}'", fl_name);
    query = format!("{} and '{}' in parents and trashed = false", query, parentid);

    let files = client.files().list_all(
        "allDrives",  // corpora
        "", // drive id
        true,     // include_items_from_all_drives
        "",       // include_permissions_for_view
        false,    // include_team_drive_items
        "",       // order_by
        &query,   // query
        "",       // spaces
        true,     // supports_all_drives
        false,    // supports_team_drives
        "",       // team_drive_id
    )
    .await;

    let filesvec = files.unwrap().body;

    if filesvec.is_empty() {
        return;
    }else{
        let fl = filesvec.get(0).unwrap();

        client.files().delete(&fl.id, true, true).await.unwrap();
    }
}

pub(crate) async fn create_folder_gd(
    folders: Vec<String>,
    folder_id_glb: String,
    client_glb: &Client,
    tokens_glb: &AccessToken,
) {
    for file in &folders {
        let client = client_glb.clone();
        let folder_id = folder_id_glb.clone();
        let mut folder_id_tmp: String = folder_id.to_string();
        let file_path = file.clone();
        let relative_path = Path::new(file);
        let mut current_parent_id = folder_id_tmp.to_string();

        for component in relative_path.components() {
            if component == Component::RootDir {
                continue;
            }

            let component_str = component.as_os_str().to_str().unwrap();

            let folder_mime_type = "application/vnd.google-apps.folder";
            let mut file: google_drive::types::File = Default::default();
            // Set the name,
            file.name = component_str.to_string();
            file.mime_type = folder_mime_type.to_string();
            file.parents = vec![current_parent_id.clone()];
            let res = client.files().create(false, "published", false, "en", true, true, false, &file).await.unwrap();
            current_parent_id = res.body.id;
        }
    }
}

pub(crate) async fn delete_folder_gd(
    name: String,
    folder_id: String,
    client_glb: &Client,
    tokens_glb: &AccessToken,
) {
    let client: Client = client_glb.clone();
    let mut folder_id_tmp: String = folder_id.to_string();
    let file_path = name.clone();
    let relative_path = Path::new(&name);
    let mut current_parent_id = folder_id_tmp.to_string();

    for component in relative_path.parent().unwrap_or(Path::new("")).components() {
        if component == Component::RootDir {
            continue;
        }

        let component_str = component.as_os_str().to_str().unwrap();

        let folder_mime_type = "application/vnd.google-apps.folder";
        let mut file: google_drive::types::File = Default::default();
        // Set the name,
        file.name = component_str.to_string();
        file.mime_type = folder_mime_type.to_string();
        file.parents = vec![current_parent_id.clone()];
    
        let mut query = format!(
            "name = '{}' and mimeType = 'application/vnd.google-apps.folder'",
            component_str
        );
        query = format!("{} and '{}' in parents and trashed = false", query, current_parent_id.clone());
    
        // Check if the folder exists.
        let resp = client.files()
            .list_all(
                "allDrives",  // corpora
                "", // drive id
                true,     // include_items_from_all_drives
                "",       // include_permissions_for_view
                false,    // include_team_drive_items
                "",       // order_by
                &query,   // query
                "",       // spaces
                true,     // supports_all_drives
                false,    // supports_team_drives
                "",       // team_drive_id
            )
            .await.unwrap();
    
        if !resp.body.is_empty() {
            current_parent_id = resp.body.get(0).unwrap().id.clone();
        }else{
            return;
        }
    }
    client_glb.files().delete_by_name(&folder_id, &current_parent_id, relative_path.file_name().unwrap().to_str().unwrap());
}