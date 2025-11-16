# Spiris Bokföring och Fakturering - Terminal UI

A powerful and comprehensive Terminal User Interface (TUI) for managing all aspects of your accounting with the Spiris Bokföring och Fakturering API (formerly Visma eAccounting).

![Spiris TUI Demo](https://img.shields.io/badge/status-production--ready-green)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

## Features

### Core Functionality
- 📊 **Statistics Dashboard** - View real-time statistics for customers, invoices, and articles
- 👥 **Customer Management** - Full CRUD operations: List, view, create, edit, delete, and search customers
- 🧾 **Invoice Management** - Complete invoice handling: Browse, view, create, delete invoices with line items
- 🏷️ **Article/Product Management** - Manage your product catalog: Create, view, delete articles with prices
- 🔍 **Smart Search** - Search across customers and invoices with real-time filtering
- 💾 **Data Export** - Export all data to JSON files with timestamps
- ✅ **Input Validation** - Real-time validation for all form inputs with helpful error messages

### User Experience
- 🔐 **OAuth2 Authentication** - Secure authentication with automatic token handling
- ⌨️ **Full Keyboard Navigation** - Efficient keyboard-driven interface with shortcuts
- 💾 **Persistent Sessions** - Automatic token storage for seamless usage
- 🔄 **Real-time Refresh** - Instantly refresh data with 'r' key
- 📄 **Pagination Support** - Navigate through multiple pages with ← → keys
- ⏱️ **Auto-clearing Messages** - Status and error messages auto-dismiss after timeout
- 🔍 **Live Search Input** - Type to search with real-time query updates
- ✅ **Delete Confirmation** - Safety dialog prevents accidental deletions
- 📊 **Form Progress Indicators** - Clear field progress tracking (Field 2/4)
- 🎯 **Context-Aware Footer** - Dynamic shortcuts based on current screen
- 🎨 **Beautiful UI** - Clean and intuitive terminal interface with color-coded states
- ⚡ **Fast Performance** - Optimized async operations with Tokio
- 💡 **Enhanced Loading States** - Clear visual feedback during data loading

## Screenshots

### Main Menu
```
┌──────────────────────────────────────────────────────┐
│      Spiris Bokföring och Fakturering - TUI          │
└──────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────┐
│ Main Menu                                            │
│                                                      │
│  >> Dashboard - View statistics and quick access     │
│     Customers - Browse and manage customers          │
│     Invoices - Browse and manage invoices            │
│     Articles - Browse and manage products/articles   │
│     Search - Search across all entities              │
│     Export - Export data to JSON                     │
│     Help - View keyboard shortcuts                   │
└──────────────────────────────────────────────────────┘
```

## Installation

### Prerequisites

- Rust 1.70 or later
- Spiris Bokföring och Fakturering API credentials (Client ID & Secret)
- Terminal emulator with Unicode support

### Build from Source

```bash
# Clone the repository
git clone https://github.com/jimmystridh/claude_jungle_bamboo
cd claude_jungle_bamboo/spiris-tui

# Build the application
cargo build --release

# Run the TUI
cargo run --release
```

## Configuration

### OAuth2 Credentials

The TUI requires OAuth2 credentials to authenticate with the Spiris API. Set the following environment variables:

```bash
export SPIRIS_CLIENT_ID="your_client_id"
export SPIRIS_CLIENT_SECRET="your_client_secret"
```

You can obtain these credentials from the [Visma Developer Portal](https://developer.visma.com/).

### Token Storage

After successful authentication, your access token is automatically saved to `.spiris_token.json` in the current directory. This allows you to resume your session without re-authenticating.

**Security Note:** Keep this file secure and never commit it to version control!

## Usage

### Launching the TUI

```bash
# From the spiris-tui directory
cargo run --release

# Or if you've built the binary
./target/release/spiris-tui
```

### Keyboard Shortcuts

#### Global Navigation

| Key | Action |
|-----|--------|
| `Tab` | Next screen |
| `Shift+Tab` | Previous screen |
| `↑` / `↓` | Navigate lists |
| `←` / `→` | Previous / Next page |
| `Enter` | Select / Confirm |
| `Esc` | Go back / Cancel / Stop typing |
| `q` | Quit (from main screens) |

#### Context-Specific Actions

| Key | Action | Available In |
|-----|--------|--------------|
| `n` | Create new | Customers, Invoices, Articles |
| `e` | Edit selected item | Customer Detail |
| `x` | Delete selected item | Customer/Invoice/Article Detail |
| `r` | Refresh current view | Customers, Invoices, Articles, Dashboard |
| `d` | Go to Dashboard | Any screen |
| `s` | Open Search | Any screen |
| `h` or `?` | Show help | Any screen |

### Screens

#### 1. Authentication Screen

If no valid token is found, you'll be presented with the authentication screen:

1. Press `Enter` to start the OAuth2 flow
2. Copy the authorization URL displayed
3. Open it in your browser
4. Authorize the application
5. The token will be automatically saved

#### 2. Home Screen

The main menu provides quick access to:
- **Dashboard** - View statistics and quick actions
- **Customers** - Browse and manage customers
- **Invoices** - Browse and manage invoices
- **Articles** - Browse and manage products/articles
- **Search** - Search across all entities
- **Export** - Export data to JSON files
- **Help** - View keyboard shortcuts and documentation

Use `↑`/`↓` to navigate and `Enter` to select.

#### 3. Dashboard Screen

The dashboard displays real-time statistics and provides quick access:
- Total number of customers
- Total number of invoices
- Total number of articles
- Quick export action

Press `Enter` on any item to navigate to that section or export all data.

#### 3. Customers Screen

- Browse all customers with pagination
- View customer details (number, name, email, phone)
- Press `Enter` to view full customer details
- Press `n` to create a new customer
- Press `r` to refresh the customer list

#### 4. Customer Creation

Fill in the form fields:
1. Name (required)
2. Email (required)
3. Phone (required)
4. Website (optional)

Press `Enter` after each field. The customer is created automatically after the last field.

#### 5. Invoices Screen

- Browse all invoices with pagination
- View invoice number, customer, and total amount
- Press `Enter` to view full invoice details
- Press `r` to refresh the invoice list

#### 6. Invoice Detail View

View complete invoice information:
- Invoice number
- Customer ID
- Invoice date
- Total amount
- VAT amount
- Total including VAT
- Remarks

Press `Esc` to return to the invoice list.

#### 7. Customer Editing

From the Customer Detail view, press `e` to edit:
1. Pre-populated form with existing data
2. Modify any fields (name, email, phone, website)
3. Press `Enter` to save changes
4. Returns to Customer Detail view after successful update

#### 8. Articles/Products Screen

- Browse all articles/products with pagination
- View article name, number, and sales price
- Press `Enter` to view full article details
- Press `n` to create a new article
- Press `r` to refresh the articles list

#### 9. Article Creation

Fill in the form fields:
1. Name (required)
2. Sales Price in SEK (required)

Press `Enter` after each field. The article is created automatically after the last field.

#### 10. Article Detail View

View complete article information:
- Article ID
- Article number
- Name
- Unit type
- Sales price
- Purchase price
- Active status

Press `Esc` to return to the articles list.

#### 11. Invoice Creation

Fill in the form fields:
1. Customer ID (required)
2. Description/Remarks (required)
3. Amount in SEK (required)

Press `Enter` after each field. A simple invoice with one line item is created automatically.

#### 12. Search Screen

Real-time search across customers and invoices:
- Start typing to enter search mode (query updates live)
- Press `Enter` to execute search
- Press `ESC` to stop typing and navigate results
- Results show matching customers and invoices
- Search is performed on names, emails, customer IDs, and remarks
- Client-side filtering for fast results

#### 13. Export Screen

Export all loaded data to JSON files:
- Press `Enter` to export
- Creates timestamped files:
  - `customers_export_YYYYMMDD_HHMMSS.json`
  - `invoices_export_YYYYMMDD_HHMMSS.json`
  - `articles_export_YYYYMMDD_HHMMSS.json`
- Files are saved in the current directory
- Status message shows export results

#### 14. Help Screen

Press `h` or `?` from any screen to view the help page with all keyboard shortcuts and available screens.

## Features in Detail

### Automatic Token Refresh

The TUI checks if your access token is expired before each API request. If the token is expired and you have a refresh token, you'll need to re-authenticate through the OAuth2 flow.

### Error Handling

- **Network errors**: Displayed at the bottom of the screen
- **Authentication errors**: Redirects to the auth screen
- **API errors**: Shown with context-specific messages

### Loading States

When fetching data from the API, a loading indicator is displayed:
```
Loading customers...
```

### Empty States

When no data is available, helpful messages guide you:
```
No customers found. Press 'n' to create a new customer.
```

## Development

### Project Structure

```
spiris-tui/
├── src/
│   ├── main.rs          # Application entry point
│   ├── app.rs           # Application state and logic
│   ├── ui.rs            # UI rendering
│   ├── auth.rs          # OAuth2 authentication helpers
│   └── screens/         # Screen-specific modules (future)
├── Cargo.toml           # Dependencies
└── README.md           # This file
```

### Running in Development

```bash
cargo run
```

### Building for Release

```bash
cargo build --release
```

The optimized binary will be at `target/release/spiris-tui`.

## Troubleshooting

### Token Expired Errors

If you see authentication errors:

1. Delete `.spiris_token.json`
2. Restart the TUI
3. Complete the OAuth2 flow again

### Display Issues

If you experience display issues:

1. Ensure your terminal supports Unicode
2. Try resizing the terminal window
3. Check that your terminal emulator is up to date

### API Connection Errors

If you can't connect to the API:

1. Check your internet connection
2. Verify your OAuth2 credentials are correct
3. Ensure the Spiris API is accessible

### Missing Customers/Invoices

If data doesn't appear:

1. Press `r` to refresh
2. Check that your account has data
3. Verify API permissions in the developer portal

## Known Limitations

- **Advanced Filtering**: Basic search implemented, advanced filters coming soon
- **Invoice Editing**: Invoices cannot be edited after creation (can only delete)
- **CSV Export**: Only JSON export currently supported (CSV/PDF coming soon)
- **Article Editing**: Articles cannot be edited after creation (can only delete)
- **Total Page Count**: Page count estimation is approximate (API doesn't return total count)

## Roadmap

- [x] ✅ Complete invoice creation form
- [x] ✅ Implement customer editing
- [x] ✅ Add search and filtering
- [x] ✅ Add articles/products management
- [x] ✅ Export functionality (JSON)
- [x] ✅ Statistics dashboard
- [x] ✅ Pagination support (navigate through pages)
- [x] ✅ Auto-clearing status/error messages
- [x] ✅ Live search input with real-time updates
- [x] ✅ Improved refresh mechanism
- [x] ✅ Input validation for forms (email, numbers, required fields)
- [x] ✅ Delete functionality for customers, invoices, and articles
- [x] ✅ Delete confirmation dialog for safety
- [x] ✅ Context-aware footer with relevant shortcuts
- [x] ✅ Form progress indicators (Field X/Y)
- [x] ✅ Enhanced loading indicators with better visuals
- [x] ✅ Color-coded UI states (success, error, warning, loading)
- [ ] 🚧 Advanced filtering with multiple criteria
- [ ] 🚧 Invoice and article editing
- [ ] 🚧 CSV and PDF export formats
- [ ] 🚧 Multi-account support
- [ ] 🚧 Keyboard shortcut customization
- [ ] 🚧 Batch operations (bulk delete, bulk edit)
- [ ] 🚧 Report generation

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

### Areas for Contribution

- UI/UX improvements
- Additional features (editing, search, etc.)
- Bug fixes
- Documentation
- Testing

## Dependencies

- **ratatui** - Terminal UI framework
- **crossterm** - Terminal manipulation library
- **tokio** - Async runtime
- **spiris_bokforing** - Spiris API client library
- **anyhow** - Error handling
- **serde/serde_json** - Serialization
- **chrono** - Date/time handling

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Support

For issues, questions, or feature requests, please open an issue on GitHub.

## Acknowledgments

- Built with [Ratatui](https://ratatui.rs/) - A Rust library for building rich terminal user interfaces
- Uses the [Spiris Bokföring och Fakturering API](https://developer.visma.com/api/eaccounting)
- Inspired by modern TUI applications like [lazygit](https://github.com/jesseduffield/lazygit) and [k9s](https://k9scli.io/)
