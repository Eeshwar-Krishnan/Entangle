// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod auth;
pub mod gdrive;

use std::{sync::{Arc}, path::{Path, PathBuf}, fs::{self, File}, env, process::Command, io::{Write, Read}, collections::HashMap};

use futures::executor;
use futures_util::lock::Mutex;
use gdrive::{upload_files_to_google_drive, gd_get_file, gd_delete_file};
use auth::GDStruct;
use git2::{Repository, Signature, StatusOptions, RepositoryOpenFlags, RepositoryInitOptions};
use serde::{Serialize, Deserialize};
use tokio::task::spawn_blocking;
use walkdir::WalkDir;
use hex_literal::hex;
use sha2::{Sha256, Sha512, Digest};
struct MutexState(Mutex<State>);

struct State{
    signature_email: Option<String>,
    signature_name: Option<String>,
    repo_path: Option<String>,
    gdstruct: Option<GDStruct>,
}
#[derive(Debug, Serialize, Deserialize)]
struct SyncFile {
    name: String,
    path: String,
    sha256: String
}
#[derive(Debug, Serialize, Deserialize)]
struct SyncInfo {
    files: Vec<SyncFile>,
    msg: String,
    author: String
}

#[tokio::main]
async fn main() {

    let state = MutexState(Mutex::new(State {
        signature_email: None,
        signature_name: None,
        repo_path: None,
        gdstruct: None,
    }));
    tauri::Builder::default()
        .manage(Arc::new(state))
        .invoke_handler(tauri::generate_handler![open_repo, list_files, login, commit, validate_gsfile, push, initialize, gd_auth, gd_initialize, list_files_gd, gd_commit, gd_pull])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug, Serialize, Deserialize)]
struct FileData {
    name: String,
    select: bool,
    path: String,
    status: u8
}

#[tauri::command]
async fn login(state: tauri::State<'_, Arc<MutexState>>, email: String, name: String) -> Result<bool, bool> {
    let mut lclstate = state.inner().0.lock().await;
    lclstate.signature_email = Some(email);
    lclstate.signature_name = Some(name);

    return Ok(true);
}

#[tauri::command]
fn initialize(state: tauri::State<'_, Arc<MutexState>>, path: String, projectname: String) -> bool {
    let folder_path = Path::new(&path);

    // Create a SyncInfo struct
    let sync_info = SyncInfo {
        files: Vec::new(),
        msg: String::new(), // initialize with a blank message
        author: String::new(), // initialize with a blank author
    };

    // Serialize SyncInfo to JSON
    let sync_info_json = serde_json::to_string_pretty(&sync_info).unwrap();

    let sync_file_path = folder_path.join(format!("{}.sync", projectname));
    let mut file = File::create(sync_file_path).expect("Failed to create sync file");
    file.write_all(sync_info_json.as_bytes()).expect("Failed to write to sync file");

    true
}

#[tauri::command]
async fn gd_initialize(state: tauri::State<'_, Arc<MutexState>>, path: String, id: String, projectname: String) -> Result<bool, bool> {
    let mut lclstate: futures_util::lock::MutexGuard<'_, State> = state.inner().0.lock().await;

    let folder_path = Path::new(&path);
    let client = match &lclstate.gdstruct {
        Some(v) => {&v.drive},
        None => {return Ok(false)},
    };

    let tokens = &lclstate.gdstruct.as_ref().unwrap().token;

    // Create a SyncInfo struct
    let sync_info = SyncInfo {
        files: Vec::new(),
        msg: String::new(), // initialize with a blank message
        author: String::new(), // initialize with a blank author
    };

    update_hashes(state, path.clone(), projectname.clone());

    let sync_file_path = folder_path.join(format!("{}.sync", projectname));

    upload_files_to_google_drive(vec![Box::from(sync_file_path.clone())], &path, &id, client, tokens, Some(format!("{}.sync", projectname))).await;

    let sync = read_sync_file(sync_file_path).unwrap();

    let files: Vec<Box<Path>> = sync.files.into_iter().map(|entry| {
        Box::from(folder_path.join(entry.path))
    }).collect();

    upload_files_to_google_drive(files, &path, &id, client, tokens, None).await;

    Ok(true)
}

#[tauri::command]
fn gd_auth(state: tauri::State<'_, Arc<MutexState>>) -> Result<bool, bool> {
    let gds = executor::block_on(auth::auth());

    let mut lclstate = executor::block_on(state.inner().0.lock());

    lclstate.gdstruct = Some(gds);

    Ok(true)
}

fn update_hashes(state: tauri::State<'_, Arc<MutexState>>, path: String, projectname: String) -> bool {
    let folder_path = Path::new(&path);

    // Open the folder path, listing all of the files recursively
    let files: Vec<SyncFile> = WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_type().is_dir())
        .filter(|entry| entry.file_name() != ".DS_Store") // Ignore .DS_Store files
        .filter(|entry| entry.file_name().to_str().unwrap() != format!("{projectname}.sync"))
        .map(|entry| {
            let relative_path = entry.path().strip_prefix(folder_path).unwrap();
            let md5 = compute_sha256(entry.path()).unwrap();

            SyncFile {
                name: entry.file_name().to_str().unwrap().to_owned(),
                path: relative_path.to_string_lossy().into_owned(),
                sha256: md5,
            }
        })
        .collect();

    // Create a SyncInfo struct
    let sync_info = SyncInfo {
        files,
        msg: String::new(), // initialize with a blank message
        author: String::new(), // initialize with a blank author
    };

    // Serialize SyncInfo to JSON
    let sync_info_json = serde_json::to_string_pretty(&sync_info).unwrap();

    let sync_file_path = folder_path.join(format!("{}.sync", projectname));
    let mut file = File::create(sync_file_path).expect("Failed to create sync file");
    file.write_all(sync_info_json.as_bytes()).expect("Failed to write to sync file");

    true
}

fn compute_sha256(file_path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[tauri::command]
async fn open_repo(state: tauri::State<'_, Arc<MutexState>>, path: String) -> Result<bool, bool> {
    let repo = match Repository::open(path.clone()) {
        Ok(v) => {v},
        Err(e) => {println!("{}", e); return Ok(true);}
    };

    let mut lclstate = state.inner().0.lock().await;
    lclstate.repo_path = Some(path.clone());
    return Ok(true);
}

#[tauri::command]
fn list_files(state: tauri::State<'_, Arc<MutexState>>, path: String, projectname: String, remotepath: String, remoteproject: String) -> Vec<FileData> {
    let folder_path = Path::new(&path);

    // Open the folder path, listing all of the files recursively
    let local_files: HashMap<String, SyncFile> = WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_type().is_dir())
        .filter(|entry| entry.file_name() != ".DS_Store") // Ignore .DS_Store files
        .filter(|entry| entry.file_name().to_str().unwrap() != format!("{projectname}.sync"))
        .filter(|entry| entry.file_name().to_str().unwrap() != format!("{remoteproject}.sync"))
        .map(|entry| {
            let relative_path = entry.path().strip_prefix(folder_path).unwrap();
            let md5 = compute_sha256(entry.path()).unwrap();

            (relative_path.to_string_lossy().into_owned(), SyncFile { name: entry.file_name().to_str().unwrap().to_owned(), path: relative_path.to_string_lossy().into_owned(), sha256: md5 })
        })
        .collect();

    let files: Vec<SyncFile> = WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_type().is_dir())
        .filter(|entry| entry.file_name() != ".DS_Store") // Ignore .DS_Store files
        .filter(|entry| entry.file_name().to_str().unwrap() != format!("{projectname}.sync"))
        .filter(|entry| entry.file_name().to_str().unwrap() != format!("{remoteproject}.sync"))
        .map(|entry| {
            let relative_path = entry.path().strip_prefix(folder_path).unwrap();
            let md5 = compute_sha256(entry.path()).unwrap();

            SyncFile {
                name: entry.file_name().to_str().unwrap().to_owned(),
                path: relative_path.to_string_lossy().into_owned(),
                sha256: md5,
            }
        })
        .collect();

    // Create a SyncInfo struct
    let sync_info = SyncInfo {
        files,
        msg: String::new(), // initialize with a blank message
        author: String::new(), // initialize with a blank author
    };

    // Deserialize the content of the local .sync file
    let local_sync_file_path = folder_path.join(format!("{}.sync", projectname));
    let local_sync_info: SyncInfo = read_sync_file(local_sync_file_path).unwrap();

    // Deserialize the content of the remote .sync file
    let remote_sync_file_path = Path::new(&remotepath).join(format!("{}.sync", remoteproject));
    let remote_sync_info: SyncInfo = read_sync_file(remote_sync_file_path).unwrap();

    // Match the file names between the local, remote, and project files
    //Status is defined as the following
    //If the file exists in all three places and the sha hashes are the same, be zero
    //If a file exists in all three places, but the remote sha is different and the project and files sha are the same, be 1
    //If a file exists in all three places, but the remote and project sha are the same but the files sha is different, be 2
    //If a file exists in all three places, but the remote and files sha are the same but the project sha is different, be 3
    //If a file exists in all three places, but the shas are all different, be 4
    //If a file exists in remote, but not in project and files, be 5
    //If a file exists in remote and project, but not in files, be 6
    //If a file exists in project and files but not in remote, be 7
    //If a file exists in files but not project or remote, be 8
    let mut result: Vec<FileData> = sync_info
        .files
        .into_iter()
        .map(|local_file| {
            let remote_file = remote_sync_info.files.iter().find(|rf: &&SyncFile| rf.path == local_file.path.clone());
            let project_file = local_sync_info.files.iter().find(|rf: &&SyncFile| rf.path == local_file.path.clone());

            let selected = true; // Assuming all files are selected by default
            
            let status = match (remote_file, project_file) {
                (Some(rf), Some(pf)) if ((rf.sha256 == pf.sha256) && (local_file.sha256 == rf.sha256)) => 0,
                (Some(rf), Some(pf)) => {
                    if local_file.sha256 == pf.sha256 {
                        1
                    } else if pf.sha256 == rf.sha256 {
                        2
                    } else if local_file.sha256 == rf.sha256 {
                        3
                    } else {
                        4
                    }
                }
                (Some(_), None) => 5,
                (Some(_), Some(_)) => 6,
                (None, Some(_)) => 7,
                (None, None) => 8,
            };

            FileData {
                name: local_file.name,
                path: local_file.path,
                select: selected,
                status,
            }
        })
        .collect();

    // Include files from remote that are not in local
    result.extend(
        remote_sync_info
            .files
            .into_iter()
            .filter(|rf| !local_files.contains_key(&rf.path))
            .map(|rf| {
                let project_file = local_sync_info.files.iter().find(|rf2: &&SyncFile| rf2.path == rf.path.clone());

                if project_file.is_some(){
                    FileData {
                        name: rf.path.clone(),
                        path: rf.path.clone(),
                        select: true, // Assuming all files from remote are selected by default
                        status: 6,
                    }
                }else{
                    FileData {
                        name: rf.path.clone(),
                        path: rf.path.clone(),
                        select: true, // Assuming all files from remote are selected by default
                        status: 5,
                    }
                }
            }),
    );

    result
}

#[tauri::command]
async fn list_files_gd(state: tauri::State<'_, Arc<MutexState>>, path: String, projectname: String, remote_drive: String) -> Result<Vec<FileData>, ()> {
    let folder_path = Path::new(&path);
    let mut lclstate: futures_util::lock::MutexGuard<'_, State> = state.inner().0.lock().await;

    // Open the folder path, listing all of the files recursively
    let local_files: HashMap<String, SyncFile> = WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_type().is_dir())
        .filter(|entry| entry.file_name() != ".DS_Store") // Ignore .DS_Store files
        .filter(|entry| entry.file_name().to_str().unwrap() != format!("{projectname}.sync"))
        .filter(|entry| entry.file_name().to_str().unwrap() != format!("{projectname}.rmsync"))
        .map(|entry| {
            let relative_path = entry.path().strip_prefix(folder_path).unwrap();
            let md5 = compute_sha256(entry.path()).unwrap();

            (relative_path.to_string_lossy().into_owned(), SyncFile { name: entry.file_name().to_str().unwrap().to_owned(), path: relative_path.to_string_lossy().into_owned(), sha256: md5 })
        })
        .collect();

    let files: Vec<SyncFile> = WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_type().is_dir())
        .filter(|entry| entry.file_name() != ".DS_Store") // Ignore .DS_Store files
        .filter(|entry| entry.file_name().to_str().unwrap() != format!("{projectname}.sync"))
        .filter(|entry| entry.file_name().to_str().unwrap() != format!("{projectname}.rmsync"))
        .map(|entry| {
            let relative_path = entry.path().strip_prefix(folder_path).unwrap();
            let md5 = compute_sha256(entry.path()).unwrap();

            SyncFile {
                name: entry.file_name().to_str().unwrap().to_owned(),
                path: relative_path.to_string_lossy().into_owned(),
                sha256: md5,
            }
        })
        .collect();

    // Create a SyncInfo struct
    let sync_info = SyncInfo {
        files,
        msg: String::new(), // initialize with a blank message
        author: String::new(), // initialize with a blank author
    };

    // Deserialize the content of the local .sync file
    let local_sync_file_path = folder_path.join(format!("{}.sync", projectname));
    let local_sync_info: SyncInfo = read_sync_file(local_sync_file_path).unwrap();

    // Deserialize the content of the remote .sync file
    let bytes = gd_get_file(&format!("{}.sync", projectname), &remote_drive, &lclstate.gdstruct.as_ref().unwrap().drive, &lclstate.gdstruct.as_ref().unwrap().token).await;
    let rmtext = String::from_utf8(bytes).unwrap();
    let remote_sync_info: SyncInfo = serde_json::from_str(&rmtext).unwrap();

    // Match the file names between the local, remote, and project files
    //Status is defined as the following
    //If the file exists in all three places and the sha hashes are the same, be zero
    //If a file exists in all three places, but the remote sha is different and the project and files sha are the same, be 1
    //If a file exists in all three places, but the remote and project sha are the same but the files sha is different, be 2
    //If a file exists in all three places, but the remote and files sha are the same but the project sha is different, be 3
    //If a file exists in all three places, but the shas are all different, be 4
    //If a file exists in remote, but not in project and files, be 5
    //If a file exists in remote and project, but not in files, be 6
    //If a file exists in project and files but not in remote, be 7
    //If a file exists in files but not project or remote, be 8
    let mut result: Vec<FileData> = sync_info
        .files
        .into_iter()
        .map(|local_file| {
            let remote_file = remote_sync_info.files.iter().find(|rf: &&SyncFile| rf.path == local_file.path.clone());
            let project_file = local_sync_info.files.iter().find(|rf: &&SyncFile| rf.path == local_file.path.clone());

            let selected = true; // Assuming all files are selected by default
            
            let status = match (remote_file, project_file) {
                (Some(rf), Some(pf)) if ((rf.sha256 == pf.sha256) && (local_file.sha256 == rf.sha256)) => 0,
                (Some(rf), Some(pf)) => {
                    if local_file.sha256 == pf.sha256 {
                        1
                    } else if pf.sha256 == rf.sha256 {
                        2
                    } else if local_file.sha256 == rf.sha256 {
                        3
                    } else {
                        4
                    }
                }
                (Some(_), None) => 5,
                (Some(_), Some(_)) => 6,
                (None, Some(_)) => 7,
                (None, None) => 8,
            };

            FileData {
                name: local_file.name,
                path: local_file.path,
                select: selected,
                status,
            }
        })
        .collect();

    // Include files from remote that are not in local
    result.extend(
        remote_sync_info
            .files
            .into_iter()
            .filter(|rf| !local_files.contains_key(&rf.path))
            .map(|rf| {
                let project_file = local_sync_info.files.iter().find(|rf2: &&SyncFile| rf2.path == rf.path.clone());

                if project_file.is_some(){
                    FileData {
                        name: rf.path.clone(),
                        path: rf.path.clone(),
                        select: true, // Assuming all files from remote are selected by default
                        status: 6,
                    }
                }else{
                    FileData {
                        name: rf.path.clone(),
                        path: rf.path.clone(),
                        select: true, // Assuming all files from remote are selected by default
                        status: 5,
                    }
                }
            }),
    );

    Ok(result)
}

fn read_sync_file(file_path: PathBuf) -> Result<SyncInfo, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let sync_info: SyncInfo = serde_json::from_reader(reader)?;
    Ok(sync_info)
}

#[tauri::command]
fn commit(state: tauri::State<'_, Arc<MutexState>>, files: Vec<FileData>, commitmessage: String, remoteproject: String, remotepath: String, projectpath: String, projectname: String) -> bool {
    let remote_sync_file_path = Path::new(&remotepath).join(format!("{}.sync", remoteproject));
    let sync_file_path = Path::new(&projectpath).join(format!("{}.sync", projectname));

    let existing_files: HashMap<String, ()> = WalkDir::new(&remotepath)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_type().is_dir())
        .map(|entry| {
            let relative_path = entry.path().strip_prefix(&remotepath).unwrap();
            (relative_path.to_string_lossy().into_owned(), ())
        })
        .collect();

    // Copy each file from the local project to the remote path
    for file_data in files.iter() {
        if file_data.select {
            let local_file_path = Path::new(&projectpath).join(&file_data.path);
            let remote_file_path = Path::new(&remotepath).join(&file_data.path);

            if(local_file_path.exists()){
                let parres = remote_file_path.parent();
                match parres {
                    Some(v) => {
                        fs::create_dir_all(v).unwrap();
                    },
                    None => {},
                }
                if let Err(e) = fs::copy(&local_file_path, &remote_file_path) {
                    eprintln!("Failed to copy file: {} {}", parres.unwrap().to_str().unwrap().to_string(), e);
                    return false;
                }
            }else {
                if let Err(e) = fs::remove_file(&remote_file_path) {
                    eprintln!("Failed to delete file: {}", e);
                    return false;
                }
            }
        }
    }

    // Open and deserialize the remote .sync file
    let mut remote_sync_info: SyncInfo = read_sync_file(remote_sync_file_path.clone()).unwrap();

    // Set the msg field in SyncInfo to commitmessage
    remote_sync_info.msg = commitmessage;

    // Serialize SyncInfo to JSON
    let remote_sync_info_json = serde_json::to_string_pretty(&remote_sync_info).unwrap();

    // Save it back to the remote .sync file
    if let Err(e) = write_sync_file(&remote_sync_file_path.clone(), &remote_sync_info_json) {
        eprintln!("Failed to write to remote .sync file: {}", e);
        return false;
    }

    update_hashes(state.clone(), remotepath, remoteproject);

    let resync = read_sync_file(remote_sync_file_path).unwrap();
    if let Err(e) = write_sync_file(&sync_file_path.clone(), &serde_json::to_string_pretty(&resync).unwrap()) {
        eprintln!("Failed to write to local .sync file: {}", e);
        return false;
    }

    true
}

#[tauri::command]
async fn gd_commit(state: tauri::State<'_, Arc<MutexState>>, files: Vec<FileData>, commitmessage: String, remoteid: String, projectpath: String, projectname: String) -> Result<bool, ()> {
    let mut lclstate: futures_util::lock::MutexGuard<'_, State> = state.inner().0.lock().await;
    
    let filelist: Vec<Box<Path>> = (&files).into_iter()
        .filter(|entry| entry.status != 6)
        .map(|f| {
        Box::from(Path::new(&projectpath).join(&f.path))
    }).collect();

    let delfilelist: Vec<Box<Path>> = files.into_iter()
        .filter(|entry| entry.status == 6)
        .map(|f| {
        Box::from(Path::new(&f.path))
    }).collect();

    update_hashes(state, projectpath.clone(), projectname.clone());

    let sync_file_path = Path::new(&projectpath).join(format!("{}.sync", projectname));

    let client = &lclstate.gdstruct.as_ref().unwrap().drive;
    let tokens = &lclstate.gdstruct.as_ref().unwrap().token;

    upload_files_to_google_drive(vec![Box::from(sync_file_path.clone())], &projectpath, &remoteid, client, tokens, Some(format!("{}.sync", projectname))).await;

    upload_files_to_google_drive(filelist, &projectpath, &remoteid, client, tokens, None).await;

    for item in delfilelist {
        gd_delete_file(item.to_str().unwrap(), &remoteid, client, tokens).await;
    }

    Ok(true)
}


#[tauri::command]
async fn gd_pull(state: tauri::State<'_, Arc<MutexState>>, files: Vec<FileData>, remoteid: String, projectpath: String, projectname: String) -> Result<bool, ()> {
    let mut lclstate: futures_util::lock::MutexGuard<'_, State> = state.inner().0.lock().await;
    
    let sync_file_path = Path::new(&projectpath).join(format!("{}.sync", projectname));

    let client = &lclstate.gdstruct.as_ref().unwrap().drive;
    let tokens = &lclstate.gdstruct.as_ref().unwrap().token;

    for f in files {
        if(f.status == 7){
            let filepath = Path::new(&projectpath).join(&f.path);
            fs::remove_file(filepath);
        }else{
            let data = gd_get_file(&f.path, &remoteid, client, tokens).await;

            let filepath = Path::new(&projectpath).join(&f.path);
            
            let parres = filepath.parent();
            match parres {
                Some(v) => {
                    let _ = fs::create_dir_all(v);
                },
                None => {},
            }

            println!("{:?}", filepath.as_os_str());

            let mut fl = File::create(filepath).unwrap();
            fl.write_all(&data).unwrap();
        }
    }

    update_hashes(state, projectpath.clone(), projectname.clone());

    Ok(true)
}

fn write_sync_file(file_path: &Path, content: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[tauri::command]
fn validate_gsfile(state: tauri::State<'_, Arc<MutexState>>, id: String) -> bool {
    return false;
}

#[tauri::command]
fn push(state: tauri::State<'_, Arc<MutexState>>, projectname: String, remoteid: String) -> bool {
    return false;
}
