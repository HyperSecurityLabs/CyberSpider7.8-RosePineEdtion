use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

use super::{MediaCorruptionFinding, CorruptionType, MediaSeverity};

pub struct MediaCorruptionDetector {
    client: Client,
}

impl MediaCorruptionDetector {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("CyberSpider/7.8.0pro")
            .build()
            .expect("Failed to create HTTP client");
        Self { client }
    }

    pub async fn check_url(&self, url: &str) -> Result<Vec<MediaCorruptionFinding>> {
        let mut findings = Vec::new();

        let response = match self.client.head(url).send().await {
            Ok(r) => r,
            Err(_) => {
                findings.push(MediaCorruptionFinding {
                    url: url.to_string(),
                    file_type: self.guess_file_type(url),
                    corruption_type: CorruptionType::ZeroLength,
                    severity: MediaSeverity::Medium,
                    description: format!("Media URL unreachable: {}", url),
                    evidence: "Connection failed".to_string(),
                    recommendation: Some("Verify the URL is accessible and the server is responding".to_string()),
                });
                return Ok(findings);
            }
        };

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let content_length = response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        if let Some(len) = content_length {
            if len == 0 {
                findings.push(MediaCorruptionFinding {
                    url: url.to_string(),
                    file_type: self.guess_file_type(url),
                    corruption_type: CorruptionType::ZeroLength,
                    severity: MediaSeverity::High,
                    description: format!("Zero-length media file detected: {}", url),
                    evidence: format!("Content-Length: 0"),
                    recommendation: Some("Investigate why the media file is empty".to_string()),
                });
            }
        }

        let expected_type = self.expected_mime_type(url);
        if let Some(exp) = &expected_type {
            if !content_type.contains(exp) && !content_type.is_empty() {
                findings.push(MediaCorruptionFinding {
                    url: url.to_string(),
                    file_type: self.guess_file_type(url),
                    corruption_type: CorruptionType::MimeMismatch,
                    severity: MediaSeverity::Medium,
                    description: format!("MIME type mismatch for {}. Expected {} but got {}", url, exp, content_type),
                    evidence: format!("Expected: {}, Got: {}", exp, content_type),
                    recommendation: Some("Check if the server is configured correctly or the file is corrupted".to_string()),
                });
            }
        }

        if self.is_media_extension(url) {
            let get_response = match self.client.get(url).send().await {
                Ok(r) => r,
                Err(e) => {
                    findings.push(MediaCorruptionFinding {
                        url: url.to_string(),
                        file_type: self.guess_file_type(url),
                        corruption_type: CorruptionType::InvalidHeader,
                        severity: MediaSeverity::High,
                        description: format!("Failed to fetch media content: {}", e),
                        evidence: e.to_string(),
                        recommendation: Some("Verify network connectivity and server status".to_string()),
                    });
                    return Ok(findings);
                }
            };

            let status = get_response.status().as_u16();
            if status >= 400 {
                findings.push(MediaCorruptionFinding {
                    url: url.to_string(),
                    file_type: self.guess_file_type(url),
                    corruption_type: CorruptionType::InvalidHeader,
                    severity: MediaSeverity::High,
                    description: format!("Media URL returned HTTP {}: {}", status, url),
                    evidence: format!("HTTP Status: {}", status),
                    recommendation: Some("Check if the media file exists and is accessible".to_string()),
                });
                return Ok(findings);
            }

            let bytes = match get_response.bytes().await {
                Ok(b) => b,
                Err(_) => return Ok(findings),
            };

            if bytes.is_empty() {
                findings.push(MediaCorruptionFinding {
                    url: url.to_string(),
                    file_type: self.guess_file_type(url),
                    corruption_type: CorruptionType::ZeroLength,
                    severity: MediaSeverity::Critical,
                    description: format!("Empty response body for media file: {}", url),
                    evidence: "0 bytes received".to_string(),
                    recommendation: Some("The media file appears to be empty or corrupted".to_string()),
                });
                return Ok(findings);
            }

            if let Some(magic_finding) = self.check_magic_bytes(url, &bytes) {
                findings.push(magic_finding);
            }

            if let Some(exec_finding) = self.check_executable_embedded(&bytes, url) {
                findings.push(exec_finding);
            }
        }

        Ok(findings)
    }

    fn check_magic_bytes(&self, url: &str, bytes: &[u8]) -> Option<MediaCorruptionFinding> {
        let file_type = self.guess_file_type(url);
        let expected_magic: Option<&[u8]> = match file_type.as_str() {
            "jpg" | "jpeg" => Some(&[0xFF, 0xD8, 0xFF]),
            "png" => Some(&[0x89, 0x50, 0x4E, 0x47]),
            "gif" => Some(&[0x47, 0x49, 0x46]),
            "webp" => Some(&[0x52, 0x49, 0x46, 0x46]),
            "bmp" => Some(&[0x42, 0x4D]),
            "tiff" | "tif" => Some(&[0x49, 0x49, 0x2A, 0x00]),
            "pdf" => Some(&[0x25, 0x50, 0x44, 0x46]),
            "zip" => Some(&[0x50, 0x4B, 0x03, 0x04]),
            "mp4" => Some(&[0x00, 0x00, 0x00]),
            "mp3" => Some(&[0x49, 0x44, 0x33]),
            "wav" => Some(&[0x52, 0x49, 0x46, 0x46]),
            "ogg" => Some(&[0x4F, 0x67, 0x67, 0x53]),
            "ico" => Some(&[0x00, 0x00, 0x01, 0x00]),
            _ => None,
        };

        if let Some(magic) = expected_magic {
            if bytes.len() >= magic.len() {
                let actual = &bytes[..magic.len()];
                if actual != magic {
                    return Some(MediaCorruptionFinding {
                        url: url.to_string(),
                        file_type: file_type.clone(),
                        corruption_type: CorruptionType::BrokenMagicBytes,
                        severity: MediaSeverity::Critical,
                        description: format!("Corrupted {} file: magic bytes don't match", file_type),
                        evidence: format!("Expected {:02X?}, got {:02X?}", magic, actual),
                        recommendation: Some(format!("The {} file appears to be corrupted or has incorrect headers", file_type)),
                    });
                }
            } else {
                return Some(MediaCorruptionFinding {
                    url: url.to_string(),
                    file_type,
                    corruption_type: CorruptionType::TruncatedContent,
                    severity: MediaSeverity::High,
                    description: format!("Truncated media file: too short to contain valid magic bytes"),
                    evidence: format!("File size too small ({})", bytes.len()),
                    recommendation: Some("The file may be truncated or incomplete".to_string()),
                });
            }
        }

        None
    }

    fn check_executable_embedded(&self, bytes: &[u8], url: &str) -> Option<MediaCorruptionFinding> {
        let suspicious_patterns: &[&[u8]] = &[
            b"MZ",                // Windows executable
            b"ELF",               // Linux executable
            b"#!/bin/sh",
            b"#!/bin/bash",
            b"<script",           // HTML/JS injection
            b"<?php",             // PHP code
            b"<html",             // HTML in media
            b"PK\x03\x04",        // ZIP (potential malware)
        ];

        for pattern in suspicious_patterns {
            if bytes.len() > pattern.len() + 10 {
                if let Some(pos) = bytes.windows(pattern.len()).position(|w| w == *pattern) {
                    if pos > 0 && pos < bytes.len().saturating_sub(pattern.len()) {
                        let surrounding = std::str::from_utf8(&bytes[pos.saturating_sub(5)..(pos + pattern.len() + 5).min(bytes.len())])
                            .unwrap_or("binary");
                        return Some(MediaCorruptionFinding {
                            url: url.to_string(),
                            file_type: self.guess_file_type(url),
                            corruption_type: CorruptionType::ExecutableEmbedded,
                            severity: MediaSeverity::Critical,
                            description: format!("Executable or script pattern '{}' found in media file at offset {}", 
                                String::from_utf8_lossy(pattern), pos),
                            evidence: format!("Pattern '{}' at offset {} (context: {})", 
                                String::from_utf8_lossy(pattern), pos, surrounding),
                            recommendation: Some("Media file contains embedded executable code - possible malware or steganography".to_string()),
                        });
                    }
                }
            }
        }

        None
    }

    fn guess_file_type(&self, url: &str) -> String {
        let url_lower = url.to_lowercase();
        if let Some(ext_start) = url_lower.rfind('.') {
            let ext = &url_lower[ext_start + 1..];
            let ext = ext.split(|c: char| c == '?' || c == '#').next().unwrap_or(ext);
            ext.to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn expected_mime_type(&self, url: &str) -> Option<String> {
        match self.guess_file_type(url).as_str() {
            "jpg" | "jpeg" => Some("image/jpeg".to_string()),
            "png" => Some("image/png".to_string()),
            "gif" => Some("image/gif".to_string()),
            "webp" => Some("image/webp".to_string()),
            "bmp" => Some("image/bmp".to_string()),
            "svg" => Some("image/svg+xml".to_string()),
            "ico" => Some("image/x-icon".to_string()),
            "mp4" => Some("video/mp4".to_string()),
            "webm" => Some("video/webm".to_string()),
            "mp3" => Some("audio/mpeg".to_string()),
            "wav" => Some("audio/wav".to_string()),
            "ogg" => Some("audio/ogg".to_string()),
            "pdf" => Some("application/pdf".to_string()),
            "zip" => Some("application/zip".to_string()),
            _ => None,
        }
    }

    fn is_media_extension(&self, url: &str) -> bool {
        matches!(
            self.guess_file_type(url).as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg"
            | "ico" | "mp4" | "webm" | "mp3" | "wav" | "ogg"
            | "pdf" | "zip" | "tiff" | "tif"
        )
    }

    pub fn check_urls_batch(&self, urls: &[String]) -> Vec<Vec<MediaCorruptionFinding>> {
        let mut all_findings = Vec::new();
        for url in urls {
            if let Ok(findings) = futures::executor::block_on(self.check_url(url)) {
                all_findings.push(findings);
            }
        }
        all_findings
    }
}
