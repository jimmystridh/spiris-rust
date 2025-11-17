use crate::app::{App, InputMode, Screen, SortOrder, CustomerSortField, InvoiceSortField, ArticleSortField};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

fn get_sort_indicator(order: &SortOrder) -> &str {
    match order {
        SortOrder::Ascending => "↑",
        SortOrder::Descending => "↓",
    }
}

fn get_customer_sort_info(app: &App) -> String {
    let field = match app.customer_sort_field {
        CustomerSortField::Name => "Name",
        CustomerSortField::Email => "Email",
        CustomerSortField::CustomerNumber => "Number",
    };
    format!("{} {}", field, get_sort_indicator(&app.customer_sort_order))
}

fn get_invoice_sort_info(app: &App) -> String {
    let field = match app.invoice_sort_field {
        InvoiceSortField::InvoiceNumber => "Number",
        InvoiceSortField::CustomerID => "Customer",
        InvoiceSortField::Date => "Date",
        InvoiceSortField::Amount => "Amount",
    };
    format!("{} {}", field, get_sort_indicator(&app.invoice_sort_order))
}

fn get_article_sort_info(app: &App) -> String {
    let field = match app.article_sort_field {
        ArticleSortField::Name => "Name",
        ArticleSortField::Price => "Price",
        ArticleSortField::ArticleNumber => "Number",
    };
    format!("{} {}", field, get_sort_indicator(&app.article_sort_order))
}

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    draw_header(f, chunks[0], app);

    // Main content
    match &app.screen {
        Screen::Home => draw_home(f, chunks[1], app),
        Screen::Dashboard => draw_dashboard(f, chunks[1], app),
        Screen::Auth => draw_auth(f, chunks[1], app),
        Screen::Customers => draw_customers(f, chunks[1], app),
        Screen::CustomerCreate => draw_customer_form(f, chunks[1], app),
        Screen::CustomerEdit(id) => draw_customer_edit_form(f, chunks[1], app, id),
        Screen::CustomerDetail(id) => draw_customer_detail(f, chunks[1], app, id),
        Screen::Invoices => draw_invoices(f, chunks[1], app),
        Screen::InvoiceCreate => draw_invoice_form(f, chunks[1], app),
        Screen::InvoiceEdit(id) => draw_invoice_edit_form(f, chunks[1], app, id),
        Screen::InvoiceDetail(id) => draw_invoice_detail(f, chunks[1], app, id),
        Screen::Articles => draw_articles(f, chunks[1], app),
        Screen::ArticleCreate => draw_article_form(f, chunks[1], app),
        Screen::ArticleEdit(id) => draw_article_edit_form(f, chunks[1], app, id),
        Screen::ArticleDetail(id) => draw_article_detail(f, chunks[1], app, id),
        Screen::Search => draw_search(f, chunks[1], app),
        Screen::Export => draw_export(f, chunks[1], app),
        Screen::Help => draw_help(f, chunks[1]),
    }

    // Footer
    draw_footer(f, chunks[2], app);

    // Draw confirmation dialog on top if needed
    if app.confirm_delete.is_some() {
        draw_confirmation_dialog(f, app);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let (title, color) = match app.client {
        Some(_) => ("Spiris Bokföring och Fakturering - TUI ✓", Color::Green),
        None => ("Spiris Bokföring och Fakturering - TUI (Not Authenticated)", Color::Red),
    };

    let mut header_lines = vec![Line::from(Span::styled(
        title,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))];

    // Show status/error messages in header
    if let Some(msg) = &app.status_message {
        header_lines.push(Line::from(Span::styled(
            format!("✓ {}", msg),
            Style::default().fg(Color::Green),
        )));
    } else if let Some(err) = &app.error_message {
        header_lines.push(Line::from(Span::styled(
            format!("✗ {}", err),
            Style::default().fg(Color::Red),
        )));
    }

    let header = Paragraph::new(header_lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(header, area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let keys = if app.confirm_delete.is_some() {
        // Confirmation dialog is active
        "Y: Confirm deletion | N/ESC: Cancel"
    } else if app.search_input_mode {
        // Search input mode
        "Type to search | Enter: Execute search | ESC: Stop typing"
    } else {
        match app.input_mode {
            InputMode::Editing => {
                // Form editing mode
                match &app.screen {
                    Screen::CustomerCreate | Screen::CustomerEdit(_) => {
                        match app.input_field {
                            0 => "Name (required) | Enter: Next field | ESC: Cancel",
                            1 => "Email (required) | Enter: Next field | ESC: Cancel",
                            2 => "Phone (required) | Enter: Next field | ESC: Cancel",
                            3 => "Website (optional) | Enter: Submit | ESC: Cancel",
                            _ => "Enter: Submit | ESC: Cancel",
                        }
                    }
                    Screen::ArticleCreate | Screen::ArticleEdit(_) => {
                        match app.input_field {
                            0 => "Name (required) | Enter: Next field | ESC: Cancel",
                            1 => "Sales Price (required) | Enter: Submit | ESC: Cancel",
                            _ => "Enter: Submit | ESC: Cancel",
                        }
                    }
                    Screen::InvoiceCreate | Screen::InvoiceEdit(_) => {
                        match app.input_field {
                            0 => "Customer ID (required) | Enter: Next field | ESC: Cancel",
                            1 => "Description (required) | Enter: Next field | ESC: Cancel",
                            2 => "Amount (required) | Enter: Submit | ESC: Cancel",
                            _ => "Enter: Submit | ESC: Cancel",
                        }
                    }
                    _ => "Enter: Next field | ESC: Cancel",
                }
            }
            InputMode::Normal => {
                // Context-specific shortcuts
                match &app.screen {
                    Screen::Home => "↑↓: Navigate | Enter: Select | c/i/a: Quick jump | q: Quit | h: Help",
                    Screen::Dashboard => "↑↓: Navigate | Enter: Select | c/i/a: Quick jump | r: Refresh | h: Help",
                    Screen::Customers => "↑↓: Select | ←→: Page | o: Sort | Enter: View | n: New | r: Refresh | s: Search | q: Quit",
                    Screen::CustomerDetail(_) => "e: Edit | x: Delete | ESC: Back | s: Search | d: Dashboard",
                    Screen::Invoices => "↑↓: Select | ←→: Page | o: Sort | Enter: View | n: New | r: Refresh | s: Search | q: Quit",
                    Screen::InvoiceDetail(_) => "e: Edit | x: Delete | ESC: Back | s: Search | d: Dashboard",
                    Screen::Articles => "↑↓: Select | ←→: Page | o: Sort | Enter: View | n: New | r: Refresh | s: Search | q: Quit",
                    Screen::ArticleDetail(_) => "e: Edit | x: Delete | ESC: Back | s: Search | d: Dashboard",
                    Screen::Search => "Start typing to search | Enter: Execute | ESC: Back | d: Dashboard",
                    Screen::Export => "↑↓: Navigate | Enter: Select/Toggle | ESC: Back | d: Dashboard",
                    Screen::Help => "ESC: Back | d: Dashboard | s: Search",
                    Screen::Auth => "Enter: Start OAuth | q: Quit",
                    _ => "ESC: Back | s: Search | d: Dashboard | h: Help",
                }
            }
        }
    };

    let footer = Paragraph::new(keys)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

fn draw_home(f: &mut Frame, area: Rect, app: &App) {
    let items = vec![
        ListItem::new("Dashboard - View statistics and quick access"),
        ListItem::new("Customers - Browse and manage customers"),
        ListItem::new("Invoices - Browse and manage invoices"),
        ListItem::new("Articles - Browse and manage products/articles"),
        ListItem::new("Search - Search across all entities"),
        ListItem::new("Export - Export data to JSON"),
        ListItem::new("Help - View keyboard shortcuts"),
    ];

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(
        list,
        area,
        &mut ratatui::widgets::ListState::default().with_selected(Some(app.selected_customer)),
    );
}

fn draw_auth(f: &mut Frame, area: Rect, app: &App) {
    let mut text = vec![
        Line::from("OAuth2 Authentication Required"),
        Line::from(""),
        Line::from("Press Enter to start OAuth2 flow"),
        Line::from(""),
    ];

    if let Some(url) = &app.oauth_url {
        text.push(Line::from(""));
        text.push(Line::from("Open this URL in your browser:"));
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            url.clone(),
            Style::default().fg(Color::Yellow),
        )));
        text.push(Line::from(""));
        text.push(Line::from(
            "After authorization, you'll receive a code. Use the CLI to complete the flow.",
        ));
    }

    if let Some(msg) = &app.status_message {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            msg.clone(),
            Style::default().fg(Color::Green),
        )));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Authentication"))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn draw_customers(f: &mut Frame, area: Rect, app: &App) {
    if app.loading {
        let loading_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "⏳ Loading customers...",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Please wait",
                Style::default().fg(Color::Gray),
            )),
        ];
        let loading = Paragraph::new(loading_text)
            .block(Block::default().borders(Borders::ALL).title("Customers"))
            .alignment(Alignment::Center);
        f.render_widget(loading, area);
        return;
    }

    if app.customers.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "📋 No customers found",
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'n' to create a new customer",
                Style::default().fg(Color::Gray),
            )),
        ];
        let empty = Paragraph::new(empty_text)
            .block(Block::default().borders(Borders::ALL).title("Customers"))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .customers
        .iter()
        .map(|c| {
            let name = c.name.as_deref().unwrap_or("N/A");
            let email = c.email.as_deref().unwrap_or("N/A");
            let customer_number = c
                .customer_number
                .as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "N/A".to_string());

            ListItem::new(format!("[{}] {} - {}", customer_number, name, email))
        })
        .collect();

    let title = format!(
        "Customers (Page {} | Sort: {} | o: change sort | ↑↓: select, ←→: page)",
        app.current_page,
        get_customer_sort_info(app)
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(
        list,
        area,
        &mut ratatui::widgets::ListState::default().with_selected(Some(app.selected_customer)),
    );
}

fn draw_customer_form(f: &mut Frame, area: Rect, app: &App) {
    let fields = vec!["Name", "Email", "Phone", "Website (optional)"];
    let current_field = app.input_field;

    let mut text = vec![
        Line::from("Create New Customer"),
        Line::from(Span::styled(
            format!("Field {}/{}", current_field + 1, fields.len()),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
    ];

    for (i, field) in fields.iter().enumerate() {
        let value = app.form_data.get(i).map(|s| s.as_str()).unwrap_or("");
        let line = if i == current_field && app.input_mode == InputMode::Editing {
            Line::from(vec![
                Span::styled(format!("{}: ", field), Style::default().fg(Color::Yellow)),
                Span::raw(&app.input),
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ])
        } else {
            Line::from(format!("{}: {}", field, value))
        };
        text.push(line);
    }

    if let Some(err) = &app.validation_error {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            format!("⚠ {}", err),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(err) = &app.error_message {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Create Customer"),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_customer_detail(f: &mut Frame, area: Rect, app: &App, id: &str) {
    let customer = app.customers.iter().find(|c| c.id.as_deref() == Some(id));

    let text = if let Some(c) = customer {
        vec![
            Line::from(format!(
                "ID: {}",
                c.id.as_deref().unwrap_or("N/A")
            )),
            Line::from(format!(
                "Customer Number: {}",
                c.customer_number
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "N/A".to_string())
            )),
            Line::from(format!("Name: {}", c.name.as_deref().unwrap_or("N/A"))),
            Line::from(format!("Email: {}", c.email.as_deref().unwrap_or("N/A"))),
            Line::from(format!("Phone: {}", c.phone.as_deref().unwrap_or("N/A"))),
            Line::from(format!(
                "Website: {}",
                c.website.as_deref().unwrap_or("N/A")
            )),
            Line::from(format!(
                "Active: {}",
                c.is_active.map(|a| a.to_string()).unwrap_or_else(|| "N/A".to_string())
            )),
        ]
    } else {
        vec![Line::from("Customer not found")]
    };

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Customer Detail (e: edit | x: delete | ESC: back)"),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_invoices(f: &mut Frame, area: Rect, app: &App) {
    if app.loading {
        let loading_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "⏳ Loading invoices...",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Please wait",
                Style::default().fg(Color::Gray),
            )),
        ];
        let loading = Paragraph::new(loading_text)
            .block(Block::default().borders(Borders::ALL).title("Invoices"))
            .alignment(Alignment::Center);
        f.render_widget(loading, area);
        return;
    }

    if app.invoices.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "🧾 No invoices found",
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'n' to create a new invoice",
                Style::default().fg(Color::Gray),
            )),
        ];
        let empty = Paragraph::new(empty_text)
            .block(Block::default().borders(Borders::ALL).title("Invoices"))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .invoices
        .iter()
        .map(|inv| {
            let number = inv
                .invoice_number
                .as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "N/A".to_string());
            let total = inv
                .total_amount_including_vat
                .map(|t| format!("{:.2}", t))
                .unwrap_or_else(|| "N/A".to_string());
            let customer_id = inv.customer_id.as_deref().unwrap_or("N/A");

            ListItem::new(format!("[{}] Customer: {} - {} SEK", number, customer_id, total))
        })
        .collect();

    let title = format!(
        "Invoices (Page {} | Sort: {} | o: change sort | ↑↓: select, ←→: page)",
        app.current_page,
        get_invoice_sort_info(app)
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(
        list,
        area,
        &mut ratatui::widgets::ListState::default().with_selected(Some(app.selected_invoice)),
    );
}

fn draw_invoice_form(f: &mut Frame, area: Rect, app: &App) {
    let fields = vec!["Customer ID", "Description/Remarks", "Amount (SEK)"];
    let current_field = app.input_field;

    let mut text = vec![
        Line::from("Create New Invoice"),
        Line::from(Span::styled(
            format!("Field {}/{}", current_field + 1, fields.len()),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
    ];

    for (i, field) in fields.iter().enumerate() {
        let value = app.form_data.get(i).map(|s| s.as_str()).unwrap_or("");
        let line = if i == current_field && app.input_mode == InputMode::Editing {
            Line::from(vec![
                Span::styled(format!("{}: ", field), Style::default().fg(Color::Yellow)),
                Span::raw(&app.input),
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ])
        } else {
            Line::from(format!("{}: {}", field, value))
        };
        text.push(line);
    }

    if let Some(err) = &app.validation_error {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            format!("⚠ {}", err),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(err) = &app.error_message {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Create Invoice"),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_invoice_detail(f: &mut Frame, area: Rect, app: &App, id: &str) {
    let invoice = app.invoices.iter().find(|inv| inv.id.as_deref() == Some(id));

    let text = if let Some(inv) = invoice {
        vec![
            Line::from(format!(
                "Invoice Number: {}",
                inv.invoice_number
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "N/A".to_string())
            )),
            Line::from(format!(
                "Customer ID: {}",
                inv.customer_id.as_deref().unwrap_or("N/A")
            )),
            Line::from(format!(
                "Date: {}",
                inv.invoice_date
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "N/A".to_string())
            )),
            Line::from(format!(
                "Total Amount: {} SEK",
                inv.total_amount
                    .map(|t| format!("{:.2}", t))
                    .unwrap_or_else(|| "N/A".to_string())
            )),
            Line::from(format!(
                "VAT Amount: {} SEK",
                inv.total_vat_amount
                    .map(|t| format!("{:.2}", t))
                    .unwrap_or_else(|| "N/A".to_string())
            )),
            Line::from(format!(
                "Total Including VAT: {} SEK",
                inv.total_amount_including_vat
                    .map(|t| format!("{:.2}", t))
                    .unwrap_or_else(|| "N/A".to_string())
            )),
            Line::from(format!(
                "Remarks: {}",
                inv.remarks.as_deref().unwrap_or("N/A")
            )),
        ]
    } else {
        vec![Line::from("Invoice not found")]
    };

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Invoice Detail (e: edit | x: delete | ESC: back)"),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_invoice_edit_form(f: &mut Frame, area: Rect, app: &App, _id: &str) {
    let fields = vec!["Customer ID", "Description/Remarks", "Amount (SEK)"];
    let current_field = app.input_field;

    let mut text = vec![
        Line::from("Edit Invoice"),
        Line::from(Span::styled(
            format!("Field {}/{}", current_field + 1, fields.len()),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
    ];

    for (i, field) in fields.iter().enumerate() {
        let value = app.form_data.get(i).map(|s| s.as_str()).unwrap_or("");
        let line = if i == current_field && app.input_mode == InputMode::Editing {
            Line::from(vec![
                Span::styled(format!("{}: ", field), Style::default().fg(Color::Yellow)),
                Span::raw(&app.input),
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ])
        } else {
            Line::from(format!("{}: {}", field, value))
        };
        text.push(line);
    }

    if let Some(err) = &app.validation_error {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            format!("⚠ {}", err),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(err) = &app.error_message {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Edit Invoice"))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  Tab/Shift+Tab  - Switch between screens"),
        Line::from("  ↑/↓            - Navigate lists"),
        Line::from("  Enter          - Select/confirm"),
        Line::from("  ESC            - Go back/cancel"),
        Line::from("  q              - Quit (from main screens)"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  n              - Create new (customer/invoice/article)"),
        Line::from("  e              - Edit selected item"),
        Line::from("  x              - Delete selected item"),
        Line::from("  o              - Cycle sort options (in list views)"),
        Line::from("  r              - Refresh current view"),
        Line::from(""),
        Line::from("Quick Navigation:"),
        Line::from("  d              - Go to Dashboard"),
        Line::from("  c              - Go to Customers"),
        Line::from("  i              - Go to Invoices"),
        Line::from("  a              - Go to Articles"),
        Line::from("  s or /         - Search"),
        Line::from("  h or ?         - Show this help"),
        Line::from(""),
        Line::from("Screens:"),
        Line::from("  Home           - Main menu"),
        Line::from("  Dashboard      - Statistics and quick access"),
        Line::from("  Customers      - View and manage customers"),
        Line::from("  Invoices       - View and manage invoices"),
        Line::from("  Articles       - View and manage articles/products"),
        Line::from("  Search         - Search across all entities"),
        Line::from("  Export         - Export data to JSON files"),
        Line::from("  Help           - This screen"),
        Line::from(""),
        Line::from(Span::styled(
            "Press ESC to return to the previous screen",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
fn draw_dashboard(f: &mut Frame, area: Rect, app: &App) {
    // Split into two sections: stats display and quick actions
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(7)])
        .split(area);

    // Stats display (top)
    let stats_text = vec![
        Line::from(Span::styled(
            "Business Overview",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Customers: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} total", app.stats_total_customers)),
            Span::styled(" | ", Style::default().fg(Color::Gray)),
            Span::styled("Active: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{}", app.stats_active_customers)),
        ]),
        Line::from(vec![
            Span::styled("Invoices: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} total", app.stats_total_invoices)),
            Span::styled(" | ", Style::default().fg(Color::Gray)),
            Span::styled("Last 7 days: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", app.stats_recent_invoices_7d)),
            Span::styled(" | ", Style::default().fg(Color::Gray)),
            Span::styled("Last 30 days: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", app.stats_recent_invoices_30d)),
        ]),
        Line::from(vec![
            Span::styled("Articles: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", app.stats_total_articles)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Revenue Statistics",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Total Revenue: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:.2} SEK", app.stats_total_revenue),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Average Invoice: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:.2} SEK", app.stats_average_invoice)),
        ]),
    ];

    let stats = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Dashboard - Statistics"),
        )
        .wrap(Wrap { trim: false });

    // Quick actions (bottom)
    let items = vec![
        ListItem::new("View Customers"),
        ListItem::new("View Invoices"),
        ListItem::new("View Articles"),
        ListItem::new("Export All Data"),
    ];

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Quick Actions (↑↓: select, Enter: go)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_widget(stats, chunks[0]);
    f.render_stateful_widget(
        list,
        chunks[1],
        &mut ratatui::widgets::ListState::default().with_selected(Some(app.selected_customer)),
    );
}

fn draw_articles(f: &mut Frame, area: Rect, app: &App) {
    if app.loading {
        let loading_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "⏳ Loading articles...",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Please wait",
                Style::default().fg(Color::Gray),
            )),
        ];
        let loading = Paragraph::new(loading_text)
            .block(Block::default().borders(Borders::ALL).title("Articles"))
            .alignment(Alignment::Center);
        f.render_widget(loading, area);
        return;
    }

    if app.articles.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "🏷️ No articles found",
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'n' to create a new article",
                Style::default().fg(Color::Gray),
            )),
        ];
        let empty = Paragraph::new(empty_text)
            .block(Block::default().borders(Borders::ALL).title("Articles"))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .articles
        .iter()
        .map(|article| {
            let name = article.name.as_deref().unwrap_or("N/A");
            let price = article
                .sales_price
                .map(|p| format!("{:.2} SEK", p))
                .unwrap_or_else(|| "N/A".to_string());
            let article_number = article
                .article_number
                .as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "N/A".to_string());

            ListItem::new(format!("[{}] {} - {}", article_number, name, price))
        })
        .collect();

    let title = format!(
        "Articles (Page {} | Sort: {} | o: change sort | ↑↓: select, ←→: page)",
        app.current_page,
        get_article_sort_info(app)
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(
        list,
        area,
        &mut ratatui::widgets::ListState::default().with_selected(Some(app.selected_article)),
    );
}

fn draw_article_detail(f: &mut Frame, area: Rect, app: &App, id: &str) {
    let article = app.articles.iter().find(|a| a.id.as_deref() == Some(id));

    let text = if let Some(art) = article {
        vec![
            Line::from(format!("ID: {}", art.id.as_deref().unwrap_or("N/A"))),
            Line::from(format!(
                "Article Number: {}",
                art.article_number
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "N/A".to_string())
            )),
            Line::from(format!("Name: {}", art.name.as_deref().unwrap_or("N/A"))),
            Line::from(format!("Unit: {}", art.unit.as_deref().unwrap_or("N/A"))),
            Line::from(format!(
                "Sales Price: {} SEK",
                art.sales_price
                    .map(|p| format!("{:.2}", p))
                    .unwrap_or_else(|| "N/A".to_string())
            )),
            Line::from(format!(
                "Purchase Price: {} SEK",
                art.purchase_price
                    .map(|p| format!("{:.2}", p))
                    .unwrap_or_else(|| "N/A".to_string())
            )),
            Line::from(format!(
                "Active: {}",
                art.is_active.map(|a| a.to_string()).unwrap_or_else(|| "N/A".to_string())
            )),
        ]
    } else {
        vec![Line::from("Article not found")]
    };

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Article Detail (e: edit | x: delete | ESC: back)"),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_article_edit_form(f: &mut Frame, area: Rect, app: &App, _id: &str) {
    let fields = vec!["Name", "Sales Price (SEK)"];
    let current_field = app.input_field;

    let mut text = vec![
        Line::from("Edit Article"),
        Line::from(Span::styled(
            format!("Field {}/{}", current_field + 1, fields.len()),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
    ];

    for (i, field) in fields.iter().enumerate() {
        let value = app.form_data.get(i).map(|s| s.as_str()).unwrap_or("");
        let line = if i == current_field && app.input_mode == InputMode::Editing {
            Line::from(vec![
                Span::styled(format!("{}: ", field), Style::default().fg(Color::Yellow)),
                Span::raw(&app.input),
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ])
        } else {
            Line::from(format!("{}: {}", field, value))
        };
        text.push(line);
    }

    if let Some(err) = &app.validation_error {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            format!("⚠ {}", err),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(err) = &app.error_message {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Edit Article"))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_article_form(f: &mut Frame, area: Rect, app: &App) {
    let fields = vec!["Name", "Sales Price (SEK)"];
    let current_field = app.input_field;

    let mut text = vec![
        Line::from("Create New Article"),
        Line::from(Span::styled(
            format!("Field {}/{}", current_field + 1, fields.len()),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
    ];

    for (i, field) in fields.iter().enumerate() {
        let value = app.form_data.get(i).map(|s| s.as_str()).unwrap_or("");
        let line = if i == current_field && app.input_mode == InputMode::Editing {
            Line::from(vec![
                Span::styled(format!("{}: ", field), Style::default().fg(Color::Yellow)),
                Span::raw(&app.input),
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ])
        } else {
            Line::from(format!("{}: {}", field, value))
        };
        text.push(line);
    }

    if let Some(err) = &app.validation_error {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            format!("⚠ {}", err),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(err) = &app.error_message {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Create Article"),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_customer_edit_form(f: &mut Frame, area: Rect, app: &App, _id: &str) {
    let fields = vec!["Name", "Email", "Phone", "Website (optional)"];
    let current_field = app.input_field;

    let mut text = vec![
        Line::from("Edit Customer"),
        Line::from(Span::styled(
            format!("Field {}/{}", current_field + 1, fields.len()),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
    ];

    for (i, field) in fields.iter().enumerate() {
        let value = app.form_data.get(i).map(|s| s.as_str()).unwrap_or("");
        let line = if i == current_field && app.input_mode == InputMode::Editing {
            Line::from(vec![
                Span::styled(format!("{}: ", field), Style::default().fg(Color::Yellow)),
                Span::raw(&app.input),
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ])
        } else {
            Line::from(format!("{}: {}", field, value))
        };
        text.push(line);
    }

    if let Some(err) = &app.validation_error {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            format!("⚠ {}", err),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(err) = &app.error_message {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Edit Customer"))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_search(f: &mut Frame, area: Rect, app: &App) {
    let mut text = vec![
        Line::from("Search Across Customers and Invoices"),
        Line::from(""),
    ];

    // Show input field
    if app.search_input_mode {
        text.push(Line::from(vec![
            Span::styled("Query: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.input),
            Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
        ]));
    } else {
        text.push(Line::from(format!("Query: {}", app.search_query)));
    }

    text.push(Line::from(""));
    text.push(Line::from(format!(
        "Results: {} customers, {} invoices",
        app.search_results_customers.len(),
        app.search_results_invoices.len()
    )));
    text.push(Line::from(""));

    if app.loading {
        text.push(Line::from("Searching..."));
    } else if app.search_input_mode {
        text.push(Line::from("Press Enter to search, ESC to stop typing"));
    } else {
        text.push(Line::from("Type to enter search query, Enter to search"));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Search"))
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn draw_export(f: &mut Frame, area: Rect, app: &App) {
    use crate::app::ExportFormat;

    let format_str = match app.export_format {
        ExportFormat::Json => "JSON",
        ExportFormat::Csv => "CSV",
    };

    let items = vec![
        ListItem::new(format!("Format: {} (press Enter to toggle)", format_str)),
        ListItem::new("Export All Data (press Enter)"),
    ];

    let mut list_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Export Data",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Ready to export {} customers", app.customers.len())),
        Line::from(format!("Ready to export {} invoices", app.invoices.len())),
        Line::from(format!("Ready to export {} articles", app.articles.len())),
        Line::from(""),
    ];

    if let Some(msg) = &app.status_message {
        list_text.push(Line::from(""));
        list_text.push(Line::from(Span::styled(
            format!("✓ {}", msg),
            Style::default().fg(Color::Green),
        )));
    }

    // Calculate layout for list and info
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let info = Paragraph::new(list_text)
        .block(Block::default().borders(Borders::ALL).title("Info"))
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center);

    f.render_stateful_widget(
        list,
        chunks[0],
        &mut ratatui::widgets::ListState::default().with_selected(Some(app.export_selection)),
    );

    f.render_widget(info, chunks[1]);
}

fn draw_confirmation_dialog(f: &mut Frame, app: &App) {
    if let Some((entity_type, _id)) = &app.confirm_delete {
        // Create a centered popup
        let area = f.area();
        let popup_width = 60;
        let popup_height = 7;

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width.min(area.width),
            height: popup_height.min(area.height),
        };

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("⚠ Delete {} confirmation", entity_type),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Are you sure you want to delete this item?"),
            Line::from("This action cannot be undone."),
            Line::from(""),
            Line::from(vec![
                Span::styled("Y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("es  "),
                Span::styled("N", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("o  "),
                Span::styled("ESC", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" to cancel"),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .title("Confirm Delete")
                    .style(Style::default().bg(Color::Black)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, popup_area);
    }
}
