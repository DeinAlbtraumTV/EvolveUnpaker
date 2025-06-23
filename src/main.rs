use std::env;
use std::env::args;
use std::fs::{copy, create_dir_all, exists, read_dir, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::time::Duration;
use arboard::Clipboard;
use enigo::{Button, Enigo, Keyboard, Mouse, Settings};
use enigo::Direction::{Click, Press, Release};
use thirtyfour::{By, DesiredCapabilities, WebDriver};
use thirtyfour::prelude::{ElementQueryable};
use tokio::time::{sleep_until, Instant};

fn find_pak_files(dir: &PathBuf, pak_files: &mut Vec<PathBuf>) {
    if let Ok(entries) = read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path().canonicalize().unwrap();
            if path.is_dir() {
                find_pak_files(&path, pak_files);
            } else if path.extension().map_or(false, |ext| ext == "pak") {
                pak_files.push(path);
            }
        }
    }
}

fn find_non_pak_files(dir: &PathBuf, non_pak_files: &mut Vec<PathBuf>) {
    if let Ok(entries) = read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path().canonicalize().unwrap();
            if path.is_dir() {
                find_non_pak_files(&path, non_pak_files);
            } else if path.extension().map_or(false, |ext| ext != "pak") {
                non_pak_files.push(path);
            }
        }
    }
}

fn find_lua_files(dir: &PathBuf, lua_files: &mut Vec<PathBuf>) {
    if let Ok(entries) = read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path().canonicalize().unwrap();
            if path.is_dir() {
                find_lua_files(&path, lua_files);
            } else if path.extension().map_or(false, |ext| ext == "lua") {
                lua_files.push(path);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut copy_non_paks = false;
    
    for arg in args() {
        if arg == "--copy-non-paks" {
            copy_non_paks = true;
        }
    }
    
    if exists("../EvolveGame").expect("Failed to check if EvolveGame folder exists!") {
        let evolve_folder = Path::new("../EvolveGame").canonicalize().unwrap();
        let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
        let output_base = Path::new("../EvolveUnpacked/");
        let backup_dir = Path::new("../EvolveUnpacked_backup/");

        if copy_non_paks {
            // just copy over all files that are not .pak files
            let mut non_pak_files = Vec::new();
            find_non_pak_files(&evolve_folder, &mut non_pak_files);
            
            for file in non_pak_files {
                if !output_base.exists() {
                    eprintln!("Output directory does not exist. Please run the program without --copy-non-paks first.");
                    exit(1);
                }
                
                let path_rel_to_evolve_folder = file.strip_prefix(&evolve_folder).unwrap();
                let path_in_output_folder = output_base.join(path_rel_to_evolve_folder);
                
                println!("{}, {}", file.display(), path_in_output_folder.display());
                
                create_dir_all(&path_in_output_folder.parent().unwrap()).expect("Failed to create directory");
                copy(&file, &path_in_output_folder).expect("Failed to copy file");
            }
        } else {
            if !output_base.exists() {
                std::fs::create_dir(output_base).expect("Failed to create output directory");
            } else {
                if backup_dir.exists() {
                    std::fs::remove_dir_all(backup_dir).expect("Failed to remove EvolveUnpacked_backup folder");
                }
                std::fs::rename(output_base, backup_dir).expect("Failed to rename EvolveUnpacked folder");
                std::fs::create_dir(output_base).expect("Failed to create output directory");
            }

            let mut pak_files = Vec::new();
            find_pak_files(&evolve_folder, &mut pak_files);

            let mut clipboard = Clipboard::new().unwrap();

            let mut enigo = Enigo::new(&Settings::default()).unwrap();

            let caps = DesiredCapabilities::firefox();
            let driver = WebDriver::new("http://localhost:4444", caps).await.unwrap();

            driver.goto("https://luadec.metaworm.site/").await.unwrap();
            let changelog = driver.find(By::XPath("/html/body/div[1]/div[2]/div/div/header/button")).await.unwrap();
            changelog.click().await.unwrap();

            let uploader = driver.find(By::XPath("/html/body/div[1]/div[1]/div[1]/div/div[3]/div/div[2]/a")).await.unwrap();

            let mut init_uploader = true;

            for pak_path in pak_files {
                let file_name = pak_path.file_name()
                    .expect("Failed to get filename")
                    .to_string_lossy()
                    .to_string();

                // Copy the file to current directory
                copy(&pak_path, &current_dir.join(&file_name)).expect("Failed to copy pak file");

                let absolute_path = current_dir.join(&file_name).canonicalize().unwrap();

                println!("Processing: {}", file_name);

                // Run PakDecrypter.exe with the absolute path
                Command::new("PakDecrypt.exe")
                    .arg(&absolute_path)
                    .status()
                    .expect(&format!("Failed to execute PakDecrypt.exe for {}", file_name));

                let path_rel_to_evolve_folder = pak_path.strip_prefix(&evolve_folder).unwrap();
                let path_in_output_folder = output_base.join(path_rel_to_evolve_folder);

                let file = File::open(format!("{}.zip", &absolute_path.display())).unwrap();

                //Unpack the zip file
                let mut zip = zip::ZipArchive::new(&file).unwrap();

                //Continue even if a file is corrupted
                zip.extract(output_base.join(&path_in_output_folder)).unwrap();

                //Go through all files in the unzipped folder and run them through luac-parser-rs if they end in .lua, after they were unpacked
                let mut lua_files = Vec::new();
                find_lua_files(&path_in_output_folder, &mut lua_files);

                if lua_files.len() > 0 && init_uploader {
                    init_uploader = false;
                    for _i in 0..3 {
                        uploader.click().await.unwrap();

                        sleep_until(Instant::now() + Duration::from_secs(3)).await;

                        enigo.text(lua_files.first().unwrap().to_str().unwrap().replace("\\\\?\\", "").as_str()).unwrap();
                        enigo.key(enigo::Key::Return, Click).unwrap();

                        sleep_until(Instant::now() + Duration::from_secs(3)).await;
                    }
                }

                for mut lua_file in lua_files {
                    // remove the hidden class from uploader
                    uploader.click().await.unwrap();

                    sleep_until(Instant::now() + Duration::from_secs(3)).await;

                    enigo.text(lua_file.to_str().unwrap().replace("\\\\?\\", "").as_str()).unwrap();
                    enigo.key(enigo::Key::Return, Click).unwrap();

                    driver.query(By::Css("svg.circular")).wait(Duration::from_secs(10), Duration::from_secs(1)).exists().await.unwrap();

                    driver.query(By::Css("svg.circular")).wait(Duration::from_secs(10), Duration::from_secs(1)).not_exists().await.unwrap();

                    enigo.button(Button::Left, Click).unwrap();
                    enigo.key(enigo::Key::Control, Press).unwrap();
                    enigo.key(enigo::Key::A, Click).unwrap();
                    enigo.key(enigo::Key::C, Click).unwrap();
                    enigo.key(enigo::Key::Control, Release).unwrap();

                    // extract text from clipboard
                    let text = clipboard.get_text().unwrap();

                    lua_file.set_extension("decomp.lua");
                    let mut file = File::options().write(true).create(true).open(lua_file).unwrap();
                    file.write(text.as_bytes()).unwrap();
                }

                //Delete the zip file
                std::fs::remove_file(format!("{}.zip", &absolute_path.display())).unwrap();

                //Delete the pak file
                std::fs::remove_file(absolute_path).unwrap();
            }
        }
    } else {
        eprintln!("{}{}{}{}", "Failed to locate EvolveGame folder. Make sure that the folder structure looks like this: \n",
        "/steamapps/common\n",
        "-> /EvolveGame\n",
        "-> /EvolveTools\n"
        );
        exit(1);
    }
}
