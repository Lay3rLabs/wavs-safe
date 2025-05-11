# WAVS IPFS

A WebAssembly component providing IPFS functionality for the WAVS ecosystem. This component handles uploading files to IPFS, managing IPFS URLs, and provides utilities specifically for NFT-related content.

## Features

- Upload JSON data to IPFS
- Upload images to IPFS
- Get formatted IPFS URLs from CIDs
- Delete local files
- Specialized NFT content uploading (metadata and images)

## Prerequisites

To use this component, you'll need:

1. WASM Component runtime compatible with WASI Preview 2
2. A Lighthouse IPFS API key (stored in the `WAVS_ENV_LIGHTHOUSE_API_KEY` environment variable)

## Installation

Add the component to your project's dependencies:

```toml
[dependencies]
wavs-ipfs = { workspace = true }
```

## Usage

### In Rust WASM Components

Import and use the component's functions:

```rust
use wavs_ipfs::upload_json_to_ipfs;

// Upload JSON metadata to IPFS
let json_data = r#"{"name": "My NFT", "description": "An example NFT"}"#;
let ipfs_url = "https://node.lighthouse.storage/api/v0/add"; // Lighthouse API URL
match upload_json_to_ipfs(json_data.to_string(), ipfs_url.to_string()) {
    Ok(cid_url) => println!("Uploaded to IPFS: {}", cid_url), // ipfs://Qm...
    Err(err) => eprintln!("Error uploading to IPFS: {}", err),
}
```

### Available Functions

#### `upload_json_to_ipfs`

Upload JSON data to IPFS.

```rust
fn upload_json_to_ipfs(json_data: String, ipfs_url: String) -> Result<String, String>
```

Parameters:
- `json_data`: The JSON content to upload
- `ipfs_url`: The IPFS service endpoint URL

Returns:
- On success: IPFS URL in the format `ipfs://<CID>/<filename>`
- On failure: Error message

#### `upload_image_to_ipfs`

Upload an image or binary file to IPFS.

```rust
fn upload_image_to_ipfs(
    image_data: Vec<u8>, 
    filename: String, 
    ipfs_url: String
) -> Result<String, String>
```

Parameters:
- `image_data`: The binary image data
- `filename`: The filename to use when storing
- `ipfs_url`: The IPFS service endpoint URL

#### `delete_file`

Delete a file from the local filesystem.

```rust
fn delete_file(file_path: String) -> Result<bool, String>
```

#### `get_ipfs_url`

Generate an IPFS URL from a CID.

```rust
fn get_ipfs_url(cid: String, filename: Option<String>) -> String
```

#### `upload_nft_content`

Upload NFT content (metadata or image) to IPFS.

```rust
fn upload_nft_content(
    content_type: String,
    content: Vec<u8>,
    ipfs_url: String
) -> Result<String, String>
```

Parameters:
- `content_type`: MIME type of the content (e.g., "application/json", "image/png")
- `content`: The binary content to upload
- `ipfs_url`: The IPFS service endpoint URL

## Environment Variables

The component requires the following environment variable:

- `WAVS_ENV_LIGHTHOUSE_API_KEY`: Your Lighthouse API key for authentication

## Compatibility

This component is designed to work within the WAVS ecosystem and follows the WIT interface defined in `wit/world.wit`.

## License

See the repository root for license information.