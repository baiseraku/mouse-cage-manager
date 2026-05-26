use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
use chrono::Local;
use tauri::{Manager, WebviewWindow};

#[derive(serde::Deserialize)]
struct ExportByteFile {
    name: String,
    bytes: Vec<u8>,
}

#[derive(serde::Serialize)]
struct BackupFileInfo {
    name: String,
    path: String,
    size: u64,
    modified: u64,
}

#[tauri::command]
fn print_current_window(window: WebviewWindow) -> Result<(), String> {
    window.print().map_err(|err| err.to_string())
}

#[tauri::command]
fn save_export_file(
    default_name: String,
    extension: String,
    filter_name: String,
    content: String,
) -> Result<Option<String>, String> {
    let clean_extension = extension.trim_start_matches('.').to_string();
    let mut default_path = PathBuf::from(default_name);
    default_path.set_extension(&clean_extension);

    let Some(path) = rfd::FileDialog::new()
        .set_file_name(default_path.to_string_lossy().as_ref())
        .add_filter(&filter_name, &[clean_extension.as_str()])
        .save_file()
    else {
        return Ok(None);
    };

    fs::write(&path, content).map_err(|err| err.to_string())?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
fn save_export_bytes(
    default_name: String,
    extension: String,
    filter_name: String,
    bytes: Vec<u8>,
) -> Result<Option<String>, String> {
    let clean_extension = extension.trim_start_matches('.').to_string();
    let mut default_path = PathBuf::from(default_name);
    default_path.set_extension(&clean_extension);

    let Some(path) = rfd::FileDialog::new()
        .set_file_name(default_path.to_string_lossy().as_ref())
        .add_filter(&filter_name, &[clean_extension.as_str()])
        .save_file()
    else {
        return Ok(None);
    };

    fs::write(&path, bytes).map_err(|err| err.to_string())?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
fn save_export_byte_files(
    default_dir_name: String,
    files: Vec<ExportByteFile>,
) -> Result<Option<String>, String> {
    if files.is_empty() {
        return Ok(None);
    }
    let Some(base_dir) = rfd::FileDialog::new()
        .set_directory(default_dir_name)
        .pick_folder()
    else {
        return Ok(None);
    };

    fs::create_dir_all(&base_dir).map_err(|err| err.to_string())?;
    for file in files {
        fs::write(base_dir.join(file.name), file.bytes).map_err(|err| err.to_string())?;
    }
    Ok(Some(base_dir.to_string_lossy().into_owned()))
}

fn app_data_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|err| err.to_string())?;
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    Ok(dir.join("data.json"))
}

fn today_backup_name() -> String {
    format!("data-{}.json", Local::now().format("%Y-%m-%d"))
}

fn timestamp_backup_name(prefix: &str) -> String {
    format!("{prefix}-{}.json", Local::now().format("%Y-%m-%d-%H%M%S"))
}

fn backup_existing_data(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    let backup_dir = parent.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
    let backup_path = backup_dir.join(today_backup_name());
    if !backup_path.exists() {
        fs::copy(path, backup_path).map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn app_backup_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let path = app_data_file(app)?;
    let Some(parent) = path.parent() else {
        return Err("无法定位备份目录".to_string());
    };
    Ok(parent.join("backups"))
}

#[tauri::command]
fn load_app_data(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = app_data_file(&app)?;
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(path).map(Some).map_err(|err| err.to_string())
}

#[tauri::command]
fn save_app_data(app: tauri::AppHandle, content: String) -> Result<String, String> {
    let path = app_data_file(&app)?;
    backup_existing_data(&path)?;
    fs::write(&path, content).map_err(|err| err.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
fn get_app_data_path(app: tauri::AppHandle) -> Result<String, String> {
    app_data_file(&app).map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn open_app_data_dir(app: tauri::AppHandle) -> Result<(), String> {
    let path = app_data_file(&app)?;
    let Some(dir) = path.parent() else {
        return Err("无法定位数据目录".to_string());
    };
    fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    tauri_plugin_opener::open_path(dir, None::<&str>).map_err(|err| err.to_string())
}

#[tauri::command]
fn list_backup_files(app: tauri::AppHandle) -> Result<Vec<BackupFileInfo>, String> {
    let backup_dir = app_backup_dir(&app)?;
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(&backup_dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_json = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("json"))
            .unwrap_or(false);
        if !is_json {
            continue;
        }
        let metadata = entry.metadata().map_err(|err| err.to_string())?;
        let file_modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let modified = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .and_then(|value| value.get("lastModifiedAt").and_then(|ts| ts.as_u64()))
            .map(|timestamp_ms| timestamp_ms / 1000)
            .unwrap_or(file_modified);
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("backup.json")
            .to_string();
        files.push(BackupFileInfo {
            name,
            path: path.to_string_lossy().into_owned(),
            size: metadata.len(),
            modified,
        });
    }
    files.sort_by(|a, b| b.modified.cmp(&a.modified).then_with(|| b.name.cmp(&a.name)));
    Ok(files)
}

#[tauri::command]
fn open_backup_dir(app: tauri::AppHandle) -> Result<(), String> {
    let backup_dir = app_backup_dir(&app)?;
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
    tauri_plugin_opener::open_path(backup_dir, None::<&str>).map_err(|err| err.to_string())
}

#[tauri::command]
fn create_manual_backup(app: tauri::AppHandle, content: String) -> Result<BackupFileInfo, String> {
    let backup_dir = app_backup_dir(&app)?;
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;

    let mut backup_path = backup_dir.join(timestamp_backup_name("manual"));
    let mut suffix = 1;
    while backup_path.exists() {
        backup_path = backup_dir.join(format!(
            "manual-{}-{}.json",
            timestamp_backup_name("backup").trim_start_matches("backup-").trim_end_matches(".json"),
            suffix
        ));
        suffix += 1;
    }

    fs::write(&backup_path, content).map_err(|err| err.to_string())?;
    let metadata = fs::metadata(&backup_path).map_err(|err| err.to_string())?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let name = backup_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("manual-backup.json")
        .to_string();
    Ok(BackupFileInfo {
        name,
        path: backup_path.to_string_lossy().into_owned(),
        size: metadata.len(),
        modified,
    })
}

fn backup_file_path(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    let clean_name = Path::new(name)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "备份文件名无效".to_string())?;
    if !clean_name.ends_with(".json") {
        return Err("只能操作 JSON 备份文件".to_string());
    }
    Ok(app_backup_dir(app)?.join(clean_name))
}

#[tauri::command]
fn delete_backup_file(app: tauri::AppHandle, name: String) -> Result<(), String> {
    let backup_path = backup_file_path(&app, &name)?;
    if !backup_path.exists() {
        return Err("备份文件不存在".to_string());
    }
    if !backup_path.is_file() {
        return Err("目标不是备份文件".to_string());
    }
    fs::remove_file(backup_path).map_err(|err| err.to_string())
}

#[tauri::command]
fn restore_backup_file(app: tauri::AppHandle, name: String) -> Result<String, String> {
    let backup_path = backup_file_path(&app, &name)?;
    if !backup_path.exists() {
        return Err("备份文件不存在".to_string());
    }
    if !backup_path.is_file() {
        return Err("目标不是备份文件".to_string());
    }
    let content = fs::read_to_string(&backup_path).map_err(|err| err.to_string())?;

    let data_path = app_data_file(&app)?;
    if data_path.exists() {
        let backup_dir = app_backup_dir(&app)?;
        fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
        let mut snapshot_path = backup_dir.join(timestamp_backup_name("before-restore"));
        let mut suffix = 1;
        while snapshot_path.exists() {
            snapshot_path = backup_dir.join(format!(
                "before-restore-{}-{}.json",
                timestamp_backup_name("backup").trim_start_matches("backup-").trim_end_matches(".json"),
                suffix
            ));
            suffix += 1;
        }
        fs::copy(&data_path, snapshot_path).map_err(|err| err.to_string())?;
    }

    fs::write(&data_path, content).map_err(|err| err.to_string())?;
    Ok(data_path.to_string_lossy().into_owned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            print_current_window,
            save_export_file,
            save_export_bytes,
            save_export_byte_files,
            load_app_data,
            save_app_data,
            get_app_data_path,
            open_app_data_dir,
            list_backup_files,
            open_backup_dir,
            create_manual_backup,
            delete_backup_file,
            restore_backup_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
