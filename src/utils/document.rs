use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn read_file_content(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))
}

pub fn detect_file_type(path: &Path) -> &str {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "txt" | "md" | "markdown" | "rst" => "text",
        "json" => "json",
        "csv" => "csv",
        _ => "text",
    }
}

pub fn parse_json_lines(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub fn parse_csv(content: &str) -> Vec<String> {
    content
        .lines()
        .skip(1) // Skip header
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub fn chunk_by_paragraphs(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub fn chunk_by_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if ".!?".contains(ch) {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
            current = String::new();
        }
    }

    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }

    sentences
}

pub fn chunk_document(content: &str, chunk_size: usize) -> Vec<String> {
    chunk_document_with_overlap(content, chunk_size, 2)
}

pub fn chunk_document_with_overlap(
    content: &str,
    chunk_size: usize,
    overlap_paragraphs: usize,
) -> Vec<String> {
    if content.len() < chunk_size {
        return vec![content.to_string()];
    }

    let paragraphs = chunk_by_paragraphs(content);
    if paragraphs.len() <= overlap_paragraphs {
        return vec![content.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut overlap_buffer: Vec<String> = Vec::new();

    for (i, para) in paragraphs.iter().enumerate() {
        let para_with_separator = if i > 0 && !current_chunk.is_empty() {
            format!("\n\n{}", para)
        } else {
            para.clone()
        };

        if current_chunk.len() + para_with_separator.len() > chunk_size && !current_chunk.is_empty()
        {
            chunks.push(current_chunk.trim().to_string());

            let last_n = std::cmp::min(overlap_paragraphs, overlap_buffer.len());
            let overlap: Vec<String> = overlap_buffer.iter().rev().take(last_n).cloned().collect();

            current_chunk = overlap
                .into_iter()
                .rev()
                .collect::<Vec<String>>()
                .join("\n\n");
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(para);

            overlap_buffer.clear();
            for j in (i.saturating_sub(overlap_paragraphs)..i).rev() {
                if j < paragraphs.len() {
                    overlap_buffer.push(paragraphs[j].clone());
                }
            }
        } else {
            current_chunk.push_str(&para_with_separator);

            if overlap_buffer.len() >= overlap_paragraphs {
                overlap_buffer.remove(0);
            }
            overlap_buffer.push(para.clone());
        }
    }

    if !current_chunk.trim().is_empty() {
        chunks.push(current_chunk.trim().to_string());
    }

    chunks
}
