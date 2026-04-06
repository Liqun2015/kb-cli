use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;
use lopdf::Document;
use regex::Regex;
use lopdf::Object;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PaperMetadata {
    pub filename: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub pages: usize,
    pub file_size: u64,
    pub doi: Option<String>,
}

pub fn execute(custom_kb: Option<&Path>, force: bool) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let papers_dir = kb_path.join("raw/papers");
    let metadata_path = kb_path.join("logs/papers_metadata.json");

    if !papers_dir.exists() {
        println!("Papers directory not found: {}", papers_dir.display());
        return Ok(());
    }

    // Check if already extracted
    if metadata_path.exists() && !force {
        println!("Metadata already exists. Use --force to re-extract.");
        return Ok(());
    }

    println!("Extracting metadata from PDFs in: {}", papers_dir.display());

    let mut metadata_list = Vec::new();

    // Find all PDF files (skip duplicates)
    let mut seen = HashMap::new();
    for entry in walkdir::WalkDir::new(&papers_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("pdf") {
            continue;
        }

        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        // Skip duplicate files (those with _N suffix)
        let base_name = if filename.contains('_') {
            let parts: Vec<&str> = filename.rsplitn(2, '_').collect();
            if parts.len() > 1 && parts[0].chars().all(|c| c.is_ascii_digit()) {
                parts[1]
            } else {
                filename
            }
        } else {
            filename
        };

        if seen.contains_key(base_name) {
            continue;
        }
        seen.insert(base_name.to_string(), true);

        if let Ok(metadata) = extract_pdf_metadata(path) {
            println!("[OK] {}: {:?}", filename, metadata.title);
            metadata_list.push(metadata);
        } else {
            println!("[SKIP] {} (could not read)", filename);
        }
    }

    // Save metadata to JSON
    fs::create_dir_all(metadata_path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(&metadata_list)?;
    fs::write(&metadata_path, json)?;

    println!("\nExtracted metadata for {} papers", metadata_list.len());
    println!("Saved to: {}", metadata_path.display());

    Ok(())
}

fn extract_pdf_metadata(path: &Path) -> Result<PaperMetadata> {
    let filename = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let file_size = fs::metadata(path)?.len();

    let doc = Document::load(path)?;
    let pages = doc.get_pages().len();

    // Try to extract info from document
    let title = extract_info_value(&doc, b"Title");
    let author = extract_info_value(&doc, b"Author");
    let subject = extract_info_value(&doc, b"Subject");
    let creator = extract_info_value(&doc, b"Creator");

    // Try to extract DOI from filename
    let doi = extract_doi_from_filename(&filename);

    Ok(PaperMetadata {
        filename,
        title,
        author,
        subject,
        creator,
        pages,
        file_size,
        doi,
    })
}

fn extract_info_value(doc: &Document, key: &[u8]) -> Option<String> {
    // Get trailer
    let trailer = &doc.trailer;
    if let Ok(info_obj) = trailer.get(b"Info") {
        if let Ok(info_id) = info_obj.as_reference() {
            if let Ok(info_obj) = doc.get_object(info_id) {
                if let Ok(info_dict) = info_obj.as_dict() {
                    if let Ok(title_obj) = info_dict.get(key) {
                        return extract_string_from_object(title_obj);
                    }
                }
            }
        }
    }
    None
}

fn extract_string_from_object(obj: &Object) -> Option<String> {
    match obj {
        Object::String(s, _) => {
            std::str::from_utf8(s)
                .ok()
                .map(|s| s.to_string())
        }
        Object::Name(n) => {
            std::str::from_utf8(n)
                .ok()
                .map(|s| s.to_string())
        }
        Object::Array(arr) => {
            // Sometimes strings are wrapped in arrays
            for item in arr.iter() {
                if let Some(s) = extract_string_from_object(item) {
                    return Some(s);
                }
            }
            None
        }
        _ => None,
    }
}

fn clean_string(s: String) -> String {
    s.trim().to_string()
}

fn extract_doi_from_filename(filename: &str) -> Option<String> {
    // Try DOI patterns
    if filename.starts_with("10.") {
        let re = Regex::new(r"^10\.[0-9]{4,}/[^\s\.]+").unwrap();
        if let Some(caps) = re.captures(filename) {
            return Some(caps[0].to_string());
        }
    }
    // Try other DOI-like patterns
    if filename.contains("10.") {
        let re = Regex::new(r"10\.[0-9]{4,}/[^\s]+").unwrap();
        if let Some(caps) = re.captures(filename) {
            return Some(caps[0].to_string());
        }
    }
    None
}
