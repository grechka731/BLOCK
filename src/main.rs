use std::fs;
use std::io::{self, Write};
use std::process::Command; 
use rand::Rng;
use std::path::Path; 
use std::thread;
use std::time::Duration;
use std::time::Instant;

fn read_file(file_path: &str) -> Result<String, io::Error> {
    fs::read_to_string(file_path)
}

fn write_file(file_path: &str, contents: &str) -> Result<(), io::Error> {
    fs::write(file_path, contents)
}

fn shutdown_pc() {
    println!("\n🚨🔑 Время вышло! Система блокируется! 🔒");
    
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("shutdown").args(&["/s", "/t", "0"]).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("shutdown").arg("now").spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("osascript").arg("-e").arg("tell app \"System Events\" to shut down").spawn();
    }
    std::process::exit(1); 
}

const FLASH_DRIVE_NAME: &str = "IJNLUQN&"; 

fn find_key_path(filename: &str) -> Option<String> {
    let mut possible_paths: Vec<String> = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for letter in "CDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
            let path = format!("{}:\\{}", letter, filename);
            possible_paths.push(path);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(user) = std::env::var("USER") {
            let path_1 = format!("/run/media/{}/{}/{}", user, FLASH_DRIVE_NAME, filename);
            possible_paths.push(path_1);
        }
        let path_2 = format!("/media/{}/{}", FLASH_DRIVE_NAME, filename);
        possible_paths.push(path_2);
    }

    #[cfg(target_os = "macos")]
    {
        let path = format!("/Volumes/{}/{}", FLASH_DRIVE_NAME, filename);
        possible_paths.push(path);
    }

    for p in possible_paths {
        if Path::new(&p).exists() {
            return Some(p);
        }
    }
    None
}

fn generate_new_key<R: Rng>(rng: &mut R) -> String {
    let symbol_for_key: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_".chars().collect();
    let mut random_key = String::new();
    let alphabet_len = symbol_for_key.len();

    for _ in 0..10000 { 
        random_key.push(symbol_for_key[rng.gen_range(0..alphabet_len)]);
    }
    random_key
}

fn update_keys(block_path: &str, old_key_local_file: &str, random_key: &str) -> bool {
    let mut success = true;

    match write_file(block_path, random_key) {
        Ok(_) => println!("✅ USB: OK!"),
        Err(e) => {
            eprintln!("❌ USB: Ошибка записи: {}", e);
            success = false;
        }
    }    

    match write_file(old_key_local_file, random_key) {
        Ok(_) => println!("✅ PC: OK!"),
        Err(e) => {
            eprintln!("❌ PC: Ошибка записи: {}", e);
            success = false;
        }
    }
    success
}

fn main() {

    let mut rng = rand::thread_rng();
    println!("--- 🛡️ KEY CHECK V1.0 ---");

    let old_key_local_file = "OLD.key"; 
    let block_key_file = "BLOCK.key";
    
    let block_path_option = find_key_path(block_key_file);

    if block_path_option.is_none() {
        eprintln!("❌ USB: Флешка '{}' не найдена.", FLASH_DRIVE_NAME);
        shutdown_pc();
    }
    let block_path = block_path_option.unwrap();
    
    let (mut current_key, mut old_key) = (String::new(), String::new());

    if let Ok(contents) = read_file(&block_path) {
        current_key = contents.trim().to_string(); 
    }
    if let Ok(contents) = read_file(old_key_local_file) {
        old_key = contents.trim().to_string(); 
    }
    
    if current_key == old_key && !current_key.is_empty() {
        
        println!("✅ Ключи совпали. Обновляю...");
        let random_key = generate_new_key(&mut rng);
        update_keys(&block_path, old_key_local_file, &random_key);
        
    } else {
        
        let duration = Duration::from_secs(7);
        let start_time = Instant::now();
        println!("\n⛔ Ключи не совпадают! PC: [{}] vs USB: [{}].", old_key, current_key);
        println!("⏳ 7 секунд на подключение верной флешки...");

        while start_time.elapsed() < duration {
            
            let elapsed = start_time.elapsed();
            let remaining = duration.saturating_sub(elapsed);
            let secs_remaining = remaining.as_secs(); 
            
            print!("\r⏳ Осталось: {} сек...", secs_remaining + 1);
            let _ = io::stdout().flush(); 

            let mut new_current_key = String::new();
            let mut new_old_key = String::new();
            
            if let Ok(contents) = read_file(&block_path) {
                new_current_key = contents.trim().to_string();
            } 
            if let Ok(contents) = read_file(old_key_local_file) {
                new_old_key = contents.trim().to_string();
            }

            if new_current_key == new_old_key && !new_current_key.is_empty() {
                println!("\n🎉 Ключ принят! Обновляю...");
                let random_key = generate_new_key(&mut rng);
                update_keys(&block_path, old_key_local_file, &random_key);
                return;
            }
            
            thread::sleep(Duration::from_millis(990));
        }

        println!("\n❌ Время вышло. Блокировка...");
        shutdown_pc();
    }
}