mod wit;

use crate::wit::exports::wavs::ipfs::ipfs::Guest;

use anyhow::Result as AnyhowResult;
use serde::Deserialize;
use std::{
    fs::File,
    io::{Read, Write},
};
use wstd::http::{IntoBody, Request};
use wstd::io::AsyncRead;
use wstd::runtime::block_on;

pub struct Component;

impl Guest for Component {
    /// Uploads JSON data directly to IPFS and returns the CID
    fn upload_json_to_ipfs(json_data: String, ipfs_url: String) -> Result<String, String> {
        block_on(async {
            // Create a temporary file to store the JSON data
            let filename = "nft_metadata.json".to_string();
            let temp_path = format!("/tmp/{}", filename);

            eprint!("Temp path {}", temp_path);

            // Ensure the /tmp directory exists
            match std::fs::create_dir_all("/tmp") {
                Ok(_) => {}
                Err(e) => return Err(format!("Failed to create /tmp directory: {}", e)),
            };

            // Write JSON to temporary file
            let mut file = match File::create(&temp_path) {
                Ok(f) => f,
                Err(e) => return Err(format!("Failed to create file: {}", e)),
            };

            if let Err(e) = file.write_all(json_data.as_bytes()) {
                return Err(format!("Failed to write to file: {}", e));
            }

            // Upload the file
            let hash = match upload_to_ipfs(&temp_path, &ipfs_url).await {
                Ok(h) => h,
                Err(e) => return Err(format!("Failed to upload: {}", e)),
            };

            // Clean up the temporary file
            if let Err(e) = Self::delete_file(temp_path.clone()) {
                eprintln!("Warning: Failed to delete temp file: {}", e);
                // Continue anyway, this is not critical
            }

            // Return the IPFS URI
            Ok(Self::get_ipfs_url(hash, Some(filename)))
        })
    }

    /// Uploads an image to IPFS and returns the CID
    fn upload_image_to_ipfs(
        image_data: Vec<u8>,
        filename: String,
        ipfs_url: String,
    ) -> Result<String, String> {
        block_on(async {
            // Create a temporary file to store the image data
            let temp_path = format!("/tmp/{}", filename);

            // Ensure the /tmp directory exists
            match std::fs::create_dir_all("/tmp") {
                Ok(_) => {}
                Err(e) => return Err(format!("Failed to create /tmp directory: {}", e)),
            };

            // Write image data to temporary file
            let mut file = match File::create(&temp_path) {
                Ok(f) => f,
                Err(e) => return Err(format!("Failed to create file: {}", e)),
            };

            if let Err(e) = file.write_all(&image_data) {
                return Err(format!("Failed to write to file: {}", e));
            }

            // Upload the file
            let hash = match upload_to_ipfs(&temp_path, &ipfs_url).await {
                Ok(h) => h,
                Err(e) => return Err(format!("Failed to upload: {}", e)),
            };

            // Clean up the temporary file
            if let Err(e) = Self::delete_file(temp_path.clone()) {
                eprintln!("Warning: Failed to delete temp file: {}", e);
                // Continue anyway, this is not critical
            }

            // Return the IPFS URI
            Ok(Self::get_ipfs_url(hash, Some(filename)))
        })
    }

    /// Delete a file from the filesystem
    fn delete_file(file_path: String) -> Result<bool, String> {
        match std::fs::remove_file(&file_path) {
            Ok(_) => {
                println!("File deleted successfully: {}", file_path);
                Ok(true)
            }
            Err(e) => Err(format!("Failed to delete file: {}", e)),
        }
    }

    /// Get IPFS URL from CID
    /// If filename is provided, constructs a URL that points to a file within a directory
    fn get_ipfs_url(cid: String, filename: Option<String>) -> String {
        match filename {
            Some(name) => format!("ipfs://{}/{}", cid, name),
            None => format!("ipfs://{}", cid),
        }
    }

    /// Uploads NFT content (metadata and/or image) to IPFS
    /// Returns the IPFS URI (ipfs://CID) for the content
    fn upload_nft_content(
        content_type: String,
        content: Vec<u8>,
        ipfs_url: String,
    ) -> Result<String, String> {
        block_on(async {
            // Determine if this is JSON metadata or an image
            let ipfs_uri = if content_type.contains("json") || content_type == "application/json" {
                // It's JSON metadata
                let json_str = match std::str::from_utf8(&content) {
                    Ok(s) => s.to_string(),
                    Err(e) => return Err(format!("Failed to convert JSON bytes to string: {}", e)),
                };

                // Upload the JSON and return the IPFS URI
                Self::upload_json_to_ipfs(json_str, ipfs_url.clone())?
            } else {
                // It's an image or other binary content
                let extension = match content_type.as_str() {
                    "image/png" => "png",
                    "image/jpeg" => "jpg",
                    "image/gif" => "gif",
                    "image/svg+xml" => "svg",
                    _ => "bin", // Default extension for unknown types
                };

                let filename = format!("nft_image.{}", extension);

                // Upload the image and return the IPFS URI
                Self::upload_image_to_ipfs(content, filename, ipfs_url.clone())?
            };

            // Log the upload
            println!("Uploaded to IPFS with URI: {}", ipfs_uri);

            // Return IPFS URI
            Ok(ipfs_uri)
        })
    }
}

/// Uploads a file using multipart request to IPFS
async fn upload_to_ipfs(file_path: &str, ipfs_url: &str) -> AnyhowResult<String> {
    let api_key = std::env::var("WAVS_ENV_LIGHTHOUSE_API_KEY")
        .map_err(|e| anyhow::anyhow!("Failed to get API key: {}", e))?;

    eprintln!("Uploading file to IPFS: {}", file_path);

    let mut file = File::open(file_path)?;
    let mut file_bytes = Vec::new();
    file.read_to_end(&mut file_bytes)?;

    // define multipart request boundary
    let boundary = "----RustBoundary";

    // construct the body
    let body = format!(
        "--{}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n\
        Content-Type: application/octet-stream\r\n\r\n",
        boundary, file_path
    );

    let mut request_body = body.into_bytes();
    request_body.extend_from_slice(&file_bytes);
    request_body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let request = Request::post(ipfs_url)
        .header("Authorization", &format!("Bearer {}", api_key))
        .header("Content-Type", &format!("multipart/form-data; boundary={}", boundary))
        .body(request_body.into_body())?;

    let mut response = wstd::http::Client::new().send(request).await?;

    if response.status().is_success() {
        let mut body_buf = Vec::new();
        response.body_mut().read_to_end(&mut body_buf).await?;

        // Log the raw response for debugging
        let response_str = std::str::from_utf8(&body_buf)
            .map_err(|e| anyhow::anyhow!("Failed to convert response to string: {}", e))?;
        eprintln!("IPFS API Response: {}", response_str);

        // Parse using Lighthouse's response format (capitalized fields)
        #[allow(non_snake_case)]
        #[derive(Debug, Deserialize)]
        struct LighthouseResponse {
            Hash: String,
        }

        let hash = match serde_json::from_slice::<LighthouseResponse>(&body_buf) {
            Ok(resp) => resp.Hash,
            Err(e) => {
                // Simple fallback - just look for the hash in the response text
                eprintln!("Failed to parse response: {}", e);

                if let Some(start) = response_str.find("\"Hash\":\"") {
                    if let Some(end) = response_str[start + 8..].find("\"") {
                        return Ok(response_str[start + 8..start + 8 + end].to_string());
                    }
                }

                // If that fails too, try lowercase
                if let Some(start) = response_str.find("\"hash\":\"") {
                    if let Some(end) = response_str[start + 8..].find("\"") {
                        return Ok(response_str[start + 8..start + 8 + end].to_string());
                    }
                }

                return Err(anyhow::anyhow!(
                    "Could not extract hash from response: {}",
                    response_str
                ));
            }
        };

        // Return the hash directly
        Ok(hash)
    } else {
        let mut body_buf = Vec::new();
        response.body_mut().read_to_end(&mut body_buf).await?;
        let error_body = std::str::from_utf8(&body_buf).unwrap_or("unable to read error body");
        Err(anyhow::anyhow!(
            "Failed to upload to IPFS. Status: {:?}, Body: {}",
            response.status(),
            error_body
        ))
    }
}

// Export the component
wit::export!(Component with_types_in wit);
