# HeyReach WASM Client

A complete WebAssembly implementation of the HeyReach API client for `wasm32-wasip2` target using `wasi-http`.

## Project Structure

```
heyreach-api/
├── Cargo.toml
├── wit/
│   └── world.wit
└── src/
    ├── lib.rs          # Main component export
    ├── client.rs       # API implementation
    ├── http.rs         # HTTP client using wasi-http
    └── models.rs       # DTO models for API communication
```

## Prerequisites

- Rust 1.75 or later
- `wasm32-wasip2` target installed
- `cargo-component` (optional but recommended)

Install the target:
```bash
rustup target add wasm32-wasip2
```

Install cargo-component (optional):
```bash
cargo install cargo-component
```

## Building

### Using cargo-component (recommended)
```bash
cargo component build --release
```

### Using standard cargo
```bash
cargo build --target wasm32-wasip2 --release
```

The compiled WASM component will be in:
- `target/wasm32-wasip2/release/heyreach_api.wasm` (standard build)
- `target/wasm32-wasip2/release/heyreach_api.wasm` (component build)

## Features

This WebAssembly component implements the complete HeyReach API including:

### Authentication
- ✅ Check API key validity

### Campaigns
- ✅ Get all campaigns with filtering
- ✅ Get campaign by ID
- ✅ Resume/pause campaigns
- ✅ Add leads to campaigns (v1 and v2)

### Lists
- ✅ Get all lists
- ✅ Get list by ID
- ✅ Get leads from list
- ✅ Add leads to list (v1 and v2)
- ✅ Delete leads from list (by ID or profile URL)

### Leads & Tags
- ✅ Get lead details
- ✅ Get lists for a lead
- ✅ Get tags for a lead
- ✅ Replace tags for a lead

### Inbox
- ✅ Get conversations with filtering
- ✅ Send messages

### LinkedIn Accounts
- ✅ Get all LinkedIn accounts

### Webhooks
- ✅ Create webhook
- ✅ Get webhook by ID
- ✅ Get all webhooks
- ✅ Delete webhook

## Usage Example

The component exports the `wasco-dev:heyreach-api/api` interface. Here's how you might use it:

```rust
// Check API key
let result = check_api_key("your-api-key".to_string());

// Get all campaigns
let campaigns = campaigns_get_all(
    "your-api-key".to_string(),
    CampaignFilter {
        offset: 0,
        limit: 10,
        keyword: None,
        statuses: vec![],
        account_ids: vec![],
    }
)?;

// Add leads to a campaign
let result = campaigns_add_leads_v2(
    "your-api-key".to_string(),
    CampaignAddLeadsRequest {
        campaign_id: 123,
        account_lead_pairs: vec![
            AccountLeadPair {
                linked_in_account_id: Some(456),
                lead: Lead {
                    first_name: "John".to_string(),
                    last_name: "Doe".to_string(),
                    profile_url: "https://linkedin.com/in/johndoe".to_string(),
                    // ... other fields
                },
            }
        ],
    }
)?;
```

## API Error Handling

All API calls return `Result<T, ApiError>` where `ApiError` contains:
- `code`: Enum representing the error type (Unauthorized, NotFound, etc.)
- `message`: Human-readable error message

Error codes:
- `unauthorized` - Invalid API key (401)
- `not-found` - Resource not found (404)
- `too-many-requests` - Rate limited (429)
- `bad-request` - Invalid request (400)
- `validation` - Validation error (422)
- `rate-limited` - Rate limit exceeded
- `unknown` - Other errors

## Implementation Details

### HTTP Client
The `http.rs` module implements a complete HTTP client using the WASI HTTP interface:
- Supports GET, POST, and DELETE methods
- Automatic JSON serialization/deserialization
- Proper error handling with status code mapping
- API key authentication via `x-api-key` header

### Type Conversions
The client handles conversion between:
- WIT interface types (from `world.wit`)
- Rust DTO types (for JSON serialization)
- API response formats

### Base URL
The client connects to: `https://api.heyreach.io`

## Testing

To test the component, you'll need a WASI runtime that supports WASI Preview 2 and the `wasi:http` interface, such as:
- wasmtime (with `--wasi preview2`)
- wasmer
- Spin framework

Example with wasmtime:
```bash
wasmtime run --wasi preview2 target/wasm32-wasip2/release/heyreach_api.wasm
```

## License

This implementation follows the HeyReach API specifications.

## Contributing

When adding new endpoints:
1. Add the DTO types to `models.rs`
2. Implement the API call in `client.rs`
3. Export the function in `lib.rs`
4. Update the WIT file if needed

## Notes

- All timestamps are in ISO-8601 format
- Profile URLs must be valid LinkedIn profile URLs
- Campaign IDs and List IDs are 64-bit unsigned integers
- Pagination is supported via offset/limit parameters
