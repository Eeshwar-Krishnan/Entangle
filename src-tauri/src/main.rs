// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{sync::{Mutex, Arc}, path::{Path, PathBuf}, fs::{self, File}, env, process::Command, io::{Write, Read}, collections::HashMap};

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
    git_path: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct SyncFile {
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
        git_path: git_executable_path.to_string_lossy().to_string()
    }));
    tauri::Builder::default()
        .manage(Arc::new(state))
        .invoke_handler(tauri::generate_handler![open_repo, list_files, login, commit, auth, validate_gsfile, push, initialize])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug, Serialize)]
struct FileData {
    name: String,
    select: bool,
    path: String,
    status: u8
}

#[tauri::command]
fn login(state: tauri::State<'_, Arc<MutexState>>, email: String, name: String) -> bool {
    let mut lclstate = state.inner().0.lock().unwrap();
    lclstate.signature_email = Some(email);
    lclstate.signature_name = Some(name);

    return true;
}

#[tauri::command]
fn initialize(state: tauri::State<'_, Arc<MutexState>>, path: String, projectname: String) -> bool {
    let folder_path = Path::new(&path);

    // Open the folder path, listing all of the files recursively
    let files: Vec<SyncFile> = WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_type().is_dir())
        .filter(|entry| entry.file_name() != ".DS_Store") // Ignore .DS_Store files
        .map(|entry| {
            let relative_path = entry.path().strip_prefix(folder_path).unwrap();
            let md5 = compute_sha256(entry.path()).unwrap();

            SyncFile {
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
fn open_repo(state: tauri::State<'_, Arc<MutexState>>, path: String) -> bool {
    let repo = match Repository::open(path.clone()) {
        Ok(v) => {v},
        Err(e) => {println!("{}", e); return false;}
    };

    let mut lclstate = state.inner().0.lock().unwrap();
    lclstate.repo_path = Some(path.clone());
    return true;
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

            (relative_path.to_string_lossy().into_owned(), SyncFile { path: relative_path.to_string_lossy().into_owned(), sha256: md5 })
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
                    println!("{}: {} {} {}", local_file.path, local_file.sha256, rf.sha256, pf.sha256);
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
                name: local_file.path.clone(),
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
            .map(|rf| FileData {
                name: rf.path.clone(),
                path: rf.path.clone(),
                select: true, // Assuming all files from remote are selected by default
                status: 5,      // Status for files only in remote
            }),
    );

    result
}

fn read_sync_file(file_path: PathBuf) -> Result<SyncInfo, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let sync_info: SyncInfo = serde_json::from_reader(reader)?;
    Ok(sync_info)
}

#[tauri::command]
fn commit(state: tauri::State<'_, Arc<MutexState>>, files: Vec<String>, commit_message: String) -> bool{
    let mut lclstate = state.inner().0.lock().unwrap();

    let repo_path = match &lclstate.repo_path {
        Some(v) => v,
        None => return false,
    };

    let email = match &lclstate.signature_email {
        Some(v) => v,
        None => return false,
    };

    let name = match &lclstate.signature_name {
        Some(v) => v,
        None => return false,
    };

    let signature = match Signature::now(email, name) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let repo = match Repository::open(repo_path.clone()) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Stage changes
    let mut index = match repo.index() {
        Ok(v) => v,
        Err(_) => return false,
    };

    for file in files.iter() {
        match index.add_path(&Path::new(file)) {
            Ok(_) => (),
            Err(_) => return false,
        };
    }

    index.write().unwrap();

    // Commit changes
    let tree_id = match index.write_tree() {
        Ok(v) => v,
        Err(_) => return false,
    };

    let tree = match repo.find_tree(tree_id) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Get the current HEAD commit as the parent
    let head_commit = match repo.head() {
        Ok(reference) => match reference.peel_to_commit() {
            Ok(commit) => commit,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    match repo.commit(
        Some("HEAD"), // point HEAD to our new commit
        &signature,
        &signature,
        &commit_message,
        &tree,
        &[&head_commit], // set the current HEAD commit as the parent
    ) {
        Ok(_) => true,
        Err(e) => {
            println!("{}", e);
            false
        }
    }
}

#[tauri::command]
fn auth(state: tauri::State<'_, Arc<MutexState>>) -> Result<bool, ()> {
    return Ok(false);
}

#[tauri::command]
fn validate_gsfile(state: tauri::State<'_, Arc<MutexState>>, id: String) -> bool {
    return false;
}

#[tauri::command]
fn push(state: tauri::State<'_, Arc<MutexState>>, projectname: String, remoteid: String) -> bool {
    return false;
}
