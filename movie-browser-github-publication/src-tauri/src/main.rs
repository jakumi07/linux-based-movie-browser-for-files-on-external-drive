// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SUPPORTED_EXTENSIONS: [&str; 8] = ["mp4", "mkv", "webm", "mov", "avi", "m4v", "flv", "wmv"];

#[derive(Debug, Serialize, Clone)]
struct Movie {
    name: String,
    path: String,
    folder: String,
}

fn candidate_movie_dirs() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // 1. Highest priority: explicit override via MOVIES_DIR
    if let Some(dir) = env::var_os("MOVIES_DIR") {
        let dir = PathBuf::from(dir);
        if !dir.as_os_str().is_empty() {
            candidates.push(dir.clone());
            candidates.push(dir.join("Movies"));
            candidates.push(dir.join("movies"));
        }
    }

    // 2. High priority: Mounted external drives/media paths (Fedora common mounting paths)
    for root in [PathBuf::from("/run/media"), PathBuf::from("/media"), PathBuf::from("/mnt")] {
        if !root.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                // If it is a user mount subfolder (e.g., /run/media/jakumi), look inside it
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_dir() {
                            candidates.push(sub_path.join("Movies"));
                            candidates.push(sub_path.join("movies"));
                            candidates.push(sub_path.clone());
                        }
                    }
                }

                candidates.push(path.join("Movies"));
                candidates.push(path.join("movies"));
                candidates.push(path.clone());
            }
        }

        candidates.push(root.join("Movies"));
        candidates.push(root.join("movies"));
    }

    // 3. Medium priority: MEDIA_ROOT (used primarily in testing)
    if let Some(dir) = env::var_os("MEDIA_ROOT") {
        let dir = PathBuf::from(dir);
        if !dir.as_os_str().is_empty() {
            candidates.push(dir.join("Movies"));
            candidates.push(dir.join("movies"));
        }
    }

    // 4. Low priority: User's home movies folder
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join("Movies"));
        candidates.push(home.join("movies"));
    }

    // 5. Lowest priority fallback: local workspace / dev folder CWD / exe parent
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join("Movies"));
        candidates.push(cwd.join("movies"));
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("Movies"));
            candidates.push(parent.join("movies"));

            let mut current = parent;
            while let Some(ancestor) = current.parent() {
                candidates.push(ancestor.join("Movies"));
                candidates.push(ancestor.join("movies"));
                current = ancestor;
            }
        }
    }

    // Clean up duplicates without sorting so we preserve priority order!
    let mut unique_candidates = Vec::new();
    for path in candidates {
        if !unique_candidates.contains(&path) {
            unique_candidates.push(path);
        }
    }

    unique_candidates
}

fn find_movies_dir() -> PathBuf {
    let fallback_dir = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("Movies");

    let mut fallback_candidate = None;

    for candidate in candidate_movie_dirs() {
        if candidate.is_dir() {
            return candidate;
        }

        if fallback_candidate.is_none() {
            fallback_candidate = Some(candidate.clone());
        }
    }

    if let Some(candidate) = fallback_candidate {
        let _ = fs::create_dir_all(&candidate);
        candidate
    } else {
        let _ = fs::create_dir_all(&fallback_dir);
        fallback_dir
    }
}

fn scan_dir(dir: &Path, root_dir: &Path, movies: &mut Vec<Movie>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        let path = entry.path();
        if metadata.is_dir() {
            scan_dir(&path, root_dir, movies);
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };

        let ext_lower = ext.to_lowercase();
        if !SUPPORTED_EXTENSIONS
            .iter()
            .any(|supported| ext_lower == *supported)
        {
            continue;
        }

        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        let folder = path
            .parent()
            .and_then(|p| p.strip_prefix(root_dir).ok())
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        movies.push(Movie {
            name: name.to_string(),
            path: path.to_string_lossy().into_owned(),
            folder,
        });
    }
}

#[tauri::command]
fn scan_movies() -> Result<Vec<Movie>, String> {
    let movies_dir = find_movies_dir();
    if !movies_dir.exists() {
        fs::create_dir_all(&movies_dir).map_err(|e| e.to_string())?;
    }

    let mut movies = Vec::new();
    scan_dir(&movies_dir, &movies_dir, &mut movies);
    movies.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let json_str = serde_json::to_string_pretty(&movies).map_err(|e| e.to_string())?;
    let _ = fs::write(movies_dir.join("movies.json"), json_str);

    Ok(movies)
}

#[tauri::command]
fn play_movie(path: String) -> Result<(), String> {
    let status = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(&path).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(&path).spawn()
    };

    status.map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn candidate_movie_dirs_finds_movies_folder_on_mounted_drive() {
        let unique_name = format!(
            "movie-browser-drive-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_root = std::env::temp_dir().join(unique_name);
        let mount_root = temp_root.join("media").join("drive");
        let expected = mount_root.join("Movies");
        fs::create_dir_all(&expected).unwrap();

        let previous_media = env::var_os("MEDIA_ROOT");
        env::set_var("MEDIA_ROOT", &mount_root);
        let dirs = candidate_movie_dirs();

        assert!(dirs.iter().any(|path| path == &expected));

        match previous_media {
            Some(value) => env::set_var("MEDIA_ROOT", value),
            None => env::remove_var("MEDIA_ROOT"),
        }

        let _ = fs::remove_dir_all(temp_root);
    }

    #[test]
    fn scan_dir_finds_all_movies_in_folder() {
        let unique_name = format!(
            "movie-browser-scan-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_dir = std::env::temp_dir().join(unique_name);
        let action_dir = temp_dir.join("Action");
        fs::create_dir_all(&action_dir).unwrap();

        fs::write(action_dir.join("movie1.mp4"), "content").unwrap();
        fs::write(action_dir.join("movie2.mkv"), "content").unwrap();

        let mut movies = Vec::new();
        scan_dir(&temp_dir, &temp_dir, &mut movies);

        assert_eq!(movies.len(), 2, "Expected 2 movies, found: {:?}", movies);
        assert!(movies.iter().any(|m| m.name == "movie1.mp4"));
        assert!(movies.iter().any(|m| m.name == "movie2.mkv"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn candidate_movie_dirs_honors_environment_override() {
        let unique_name = format!(
            "movie-browser-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let override_dir = std::env::temp_dir().join(unique_name);
        fs::create_dir_all(&override_dir).unwrap();

        let previous = env::var_os("MOVIES_DIR");
        env::set_var("MOVIES_DIR", &override_dir);
        let dirs = candidate_movie_dirs();

        assert!(dirs.iter().any(|path| path == &override_dir));

        match previous {
            Some(value) => env::set_var("MOVIES_DIR", value),
            None => env::remove_var("MOVIES_DIR"),
        }

        let _ = fs::remove_dir_all(override_dir);
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![scan_movies, play_movie])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
