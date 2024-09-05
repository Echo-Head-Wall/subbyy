use std::fs::{self, File};
use std::io::{Read, Write};
use toml::Value;

fn main() -> std::io::Result<()> {
    // Read configuration
    let config = read_config("config.toml")?;
    let source_dir = config["source_dir"].as_str().unwrap_or("/var/www");
    let output_file = config["output_file"].as_str().unwrap_or("Caddyfile");
    let domain = config["domain"].as_str().unwrap_or("example.com");
    let email = config["email"].as_str().unwrap_or("your-email@example.com");
    
    let entries = fs::read_dir(source_dir)?;
    let mut caddyfile_content = format!("# Global options\n{{\n    email {}\n}}\n\n", email);
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let subdomain = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            if let Ok(subdir_entries) = fs::read_dir(&path) {
                for subentry in subdir_entries {
                    if let Ok(subentry) = subentry {
                        if subentry.path().is_file() {
                            let file_name = subentry.file_name();
                            let file_name_str = file_name.to_str().unwrap_or("unknown");
                            let config = format!(
                                "# Subdomain for {}/{}\n{}.{} {{\n    encode gzip\n    tls {{\n        protocols tls1.2 tls1.3\n    }}\n    root * {}/{}\n    file_server\n    try_files /{}\n}}\n\n",
                                subdomain, file_name_str, subdomain, domain, source_dir, subdomain, file_name_str
                            );
                            caddyfile_content.push_str(&config);
                            break;
                        }
                    }
                }
            }
        }
    }


    caddyfile_content.push_str(&format!(
        "# Catch-all for other subdomains\n*.{} {{\n    tls {{\n        protocols tls1.2 tls1.3\n    }}\n    respond \"Subdomain not found\" 404\n}}\n",
        domain
    ));


    let mut file = File::create(output_file)?;
    file.write_all(caddyfile_content.as_bytes())?;

    println!("Caddyfile has been generated successfully.");

    Ok(())
}

fn read_config(file_path: &str) -> std::io::Result<Value> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Value = toml::from_str(&contents).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(config)
}