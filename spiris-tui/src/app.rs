use anyhow::Result;
use spiris_bokforing::{AccessToken, Article, Client, Customer, Invoice, InvoiceRow, PaginationParams};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Home,
    Dashboard,
    Auth,
    Customers,
    CustomerCreate,
    CustomerEdit(String),
    CustomerDetail(String),
    Invoices,
    InvoiceCreate,
    InvoiceEdit(String),
    InvoiceDetail(String),
    Articles,
    ArticleCreate,
    ArticleEdit(String),
    ArticleDetail(String),
    Search,
    Export,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub screen: Screen,
    pub previous_screen: Option<Screen>,
    pub input_mode: InputMode,
    pub client: Option<Client>,
    pub token: Option<AccessToken>,

    // Screen state
    pub customers: Vec<Customer>,
    pub selected_customer: usize,
    pub invoices: Vec<Invoice>,
    pub selected_invoice: usize,
    pub articles: Vec<Article>,
    pub selected_article: usize,

    // Pagination state
    pub current_page: u32,
    pub total_pages: u32,
    pub page_size: u32,

    // Search state
    pub search_query: String,
    pub search_results_customers: Vec<Customer>,
    pub search_results_invoices: Vec<Invoice>,
    pub search_mode: SearchMode,
    pub search_input_mode: bool,

    // Export state
    pub export_format: ExportFormat,
    pub export_selection: usize,

    // Sort state
    pub customer_sort_field: CustomerSortField,
    pub customer_sort_order: SortOrder,
    pub invoice_sort_field: InvoiceSortField,
    pub invoice_sort_order: SortOrder,
    pub article_sort_field: ArticleSortField,
    pub article_sort_order: SortOrder,

    // Statistics
    pub stats_total_customers: usize,
    pub stats_total_invoices: usize,
    pub stats_total_articles: usize,
    pub stats_active_customers: usize,
    pub stats_total_revenue: f64,
    pub stats_average_invoice: f64,
    pub stats_recent_invoices_7d: usize,
    pub stats_recent_invoices_30d: usize,

    // Form inputs
    pub input: String,
    pub input_field: usize,
    pub form_data: Vec<String>,

    // Status/error messages
    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub message_timer: usize,
    pub validation_error: Option<String>,

    // Loading state
    pub loading: bool,
    pub needs_refresh: bool,

    // Confirmation state
    pub confirm_delete: Option<(String, String)>, // (entity_type, entity_id)

    // OAuth state
    pub oauth_url: Option<String>,
    pub oauth_waiting: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Customers,
    Invoices,
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomerSortField {
    Name,
    Email,
    CustomerNumber,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InvoiceSortField {
    InvoiceNumber,
    CustomerID,
    Date,
    Amount,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArticleSortField {
    Name,
    Price,
    ArticleNumber,
}

impl App {
    pub fn new() -> Self {
        // Try to load token from file
        let token = Self::load_token().ok();
        let client = token.as_ref().map(|t| Client::new(t.clone()));

        let screen = if token.is_some() {
            Screen::Home
        } else {
            Screen::Auth
        };

        Self {
            screen,
            previous_screen: None,
            input_mode: InputMode::Normal,
            client,
            token,
            customers: Vec::new(),
            selected_customer: 0,
            invoices: Vec::new(),
            selected_invoice: 0,
            articles: Vec::new(),
            selected_article: 0,
            current_page: 1,
            total_pages: 1,
            page_size: 50,
            search_query: String::new(),
            search_results_customers: Vec::new(),
            search_results_invoices: Vec::new(),
            search_mode: SearchMode::All,
            search_input_mode: false,
            export_format: ExportFormat::Csv,
            export_selection: 0,
            customer_sort_field: CustomerSortField::Name,
            customer_sort_order: SortOrder::Ascending,
            invoice_sort_field: InvoiceSortField::InvoiceNumber,
            invoice_sort_order: SortOrder::Ascending,
            article_sort_field: ArticleSortField::Name,
            article_sort_order: SortOrder::Ascending,
            stats_total_customers: 0,
            stats_total_invoices: 0,
            stats_total_articles: 0,
            stats_active_customers: 0,
            stats_total_revenue: 0.0,
            stats_average_invoice: 0.0,
            stats_recent_invoices_7d: 0,
            stats_recent_invoices_30d: 0,
            input: String::new(),
            input_field: 0,
            form_data: Vec::new(),
            status_message: None,
            error_message: None,
            message_timer: 0,
            validation_error: None,
            loading: false,
            needs_refresh: false,
            confirm_delete: None,
            oauth_url: None,
            oauth_waiting: false,
        }
    }

    pub fn can_quit(&self) -> bool {
        self.input_mode == InputMode::Normal
    }

    pub fn handle_escape(&mut self) {
        if self.confirm_delete.is_some() {
            // Cancel delete confirmation
            self.confirm_delete = None;
        } else if self.search_input_mode {
            self.search_input_mode = false;
            self.input.clear();
        } else if self.input_mode == InputMode::Editing {
            self.input_mode = InputMode::Normal;
            self.input.clear();
        } else if let Some(prev) = self.previous_screen.take() {
            self.screen = prev;
            self.error_message = None;
        } else {
            self.screen = Screen::Home;
        }
    }

    pub async fn handle_enter(&mut self) -> Result<()> {
        if self.input_mode == InputMode::Editing {
            // Validate current input before proceeding
            if !self.validate_current_input() {
                // Validation failed, error message is already set
                return Ok(());
            }

            self.form_data.push(self.input.clone());
            self.input.clear();
            self.input_field += 1;

            // Check if form is complete
            if self.should_submit_form() {
                self.submit_form().await?;
                self.input_mode = InputMode::Normal;
            }
        } else {
            match &self.screen {
                Screen::Home => self.handle_home_enter(),
                Screen::Dashboard => self.handle_dashboard_enter().await?,
                Screen::Customers => {
                    if !self.customers.is_empty() {
                        let customer = &self.customers[self.selected_customer];
                        if let Some(id) = &customer.id {
                            self.previous_screen = Some(Screen::Customers);
                            self.screen = Screen::CustomerDetail(id.clone());
                        }
                    }
                }
                Screen::Invoices => {
                    if !self.invoices.is_empty() {
                        let invoice = &self.invoices[self.selected_invoice];
                        if let Some(id) = &invoice.id {
                            self.previous_screen = Some(Screen::Invoices);
                            self.screen = Screen::InvoiceDetail(id.clone());
                        }
                    }
                }
                Screen::Articles => {
                    if !self.articles.is_empty() {
                        let article = &self.articles[self.selected_article];
                        if let Some(id) = &article.id {
                            self.previous_screen = Some(Screen::Articles);
                            self.screen = Screen::ArticleDetail(id.clone());
                        }
                    }
                }
                Screen::Auth => {
                    if !self.oauth_waiting {
                        self.start_oauth().await?;
                    }
                }
                Screen::Search => {
                    self.perform_search().await?;
                }
                Screen::Export => {
                    // Toggle format or export based on selection
                    match self.export_selection {
                        0 => {
                            // Toggle format
                            self.export_format = match self.export_format {
                                ExportFormat::Json => ExportFormat::Csv,
                                ExportFormat::Csv => ExportFormat::Json,
                            };
                        }
                        1 => {
                            // Execute export
                            self.export_data()?;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_home_enter(&mut self) {
        match self.selected_customer {
            0 => self.screen = Screen::Dashboard,
            1 => self.screen = Screen::Customers,
            2 => self.screen = Screen::Invoices,
            3 => self.screen = Screen::Articles,
            4 => self.screen = Screen::Search,
            5 => self.screen = Screen::Export,
            6 => self.screen = Screen::Help,
            _ => {}
        }
    }

    pub fn handle_up(&mut self) {
        match self.screen {
            Screen::Customers if !self.customers.is_empty() => {
                if self.selected_customer > 0 {
                    self.selected_customer -= 1;
                }
            }
            Screen::Invoices if !self.invoices.is_empty() => {
                if self.selected_invoice > 0 {
                    self.selected_invoice -= 1;
                }
            }
            Screen::Articles if !self.articles.is_empty() => {
                if self.selected_article > 0 {
                    self.selected_article -= 1;
                }
            }
            Screen::Home | Screen::Dashboard => {
                if self.selected_customer > 0 {
                    self.selected_customer -= 1;
                }
            }
            Screen::Export => {
                if self.export_selection > 0 {
                    self.export_selection -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn handle_down(&mut self) {
        match self.screen {
            Screen::Customers if !self.customers.is_empty() => {
                if self.selected_customer < self.customers.len() - 1 {
                    self.selected_customer += 1;
                }
            }
            Screen::Invoices if !self.invoices.is_empty() => {
                if self.selected_invoice < self.invoices.len() - 1 {
                    self.selected_invoice += 1;
                }
            }
            Screen::Articles if !self.articles.is_empty() => {
                if self.selected_article < self.articles.len() - 1 {
                    self.selected_article += 1;
                }
            }
            Screen::Home => {
                if self.selected_customer < 6 {
                    self.selected_customer += 1;
                }
            }
            Screen::Dashboard => {
                if self.selected_customer < 3 {
                    self.selected_customer += 1;
                }
            }
            Screen::Export => {
                if self.export_selection < 1 {
                    self.export_selection += 1;
                }
            }
            _ => {}
        }
    }

    pub fn handle_left(&mut self) {
        // Previous page
        if self.current_page > 1 {
            self.current_page -= 1;
            self.needs_refresh = true;
        }
    }

    pub fn handle_right(&mut self) {
        // Next page
        if self.current_page < self.total_pages {
            self.current_page += 1;
            self.needs_refresh = true;
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if self.input_mode == InputMode::Editing || self.search_input_mode {
            self.input.push(c);
            // Update search query in real-time
            if self.search_input_mode {
                self.search_query = self.input.clone();
            }
        } else if self.confirm_delete.is_some() {
            // Handle confirmation dialog
            match c {
                'y' | 'Y' => {
                    self.execute_delete();
                }
                'n' | 'N' => {
                    self.confirm_delete = None;
                }
                _ => {}
            }
        } else {
            match c {
                'r' => {
                    if self.client.is_some() {
                        self.needs_refresh = true;
                    }
                }
                'n' => {
                    match self.screen {
                        Screen::Customers => {
                            self.previous_screen = Some(Screen::Customers);
                            self.screen = Screen::CustomerCreate;
                            self.start_form();
                        }
                        Screen::Invoices => {
                            self.previous_screen = Some(Screen::Invoices);
                            self.screen = Screen::InvoiceCreate;
                            self.start_form();
                        }
                        Screen::Articles => {
                            self.previous_screen = Some(Screen::Articles);
                            self.screen = Screen::ArticleCreate;
                            self.start_form();
                        }
                        _ => {}
                    }
                }
                'e' => {
                    match self.screen {
                        Screen::CustomerDetail(ref id) => {
                            self.previous_screen = Some(Screen::CustomerDetail(id.clone()));
                            self.screen = Screen::CustomerEdit(id.clone());
                            self.start_edit_form();
                        }
                        Screen::InvoiceDetail(ref id) => {
                            self.previous_screen = Some(Screen::InvoiceDetail(id.clone()));
                            self.screen = Screen::InvoiceEdit(id.clone());
                            self.start_edit_invoice_form();
                        }
                        Screen::ArticleDetail(ref id) => {
                            self.previous_screen = Some(Screen::ArticleDetail(id.clone()));
                            self.screen = Screen::ArticleEdit(id.clone());
                            self.start_edit_article_form();
                        }
                        _ => {}
                    }
                }
                'x' => {
                    // Delete key - show confirmation dialog
                    match &self.screen {
                        Screen::CustomerDetail(ref id) => {
                            self.confirm_delete = Some(("customer".to_string(), id.clone()));
                        }
                        Screen::InvoiceDetail(ref id) => {
                            self.confirm_delete = Some(("invoice".to_string(), id.clone()));
                        }
                        Screen::ArticleDetail(ref id) => {
                            self.confirm_delete = Some(("article".to_string(), id.clone()));
                        }
                        _ => {}
                    }
                }
                's' => {
                    self.screen = Screen::Search;
                    self.search_input_mode = true;
                    self.input = self.search_query.clone();
                }
                'o' => {
                    // Cycle sort options based on current screen
                    match self.screen {
                        Screen::Customers => self.cycle_customer_sort(),
                        Screen::Invoices => self.cycle_invoice_sort(),
                        Screen::Articles => self.cycle_article_sort(),
                        _ => {}
                    }
                }
                'c' => {
                    // Quick jump to Customers
                    self.screen = Screen::Customers;
                    self.needs_refresh = true;
                }
                'i' => {
                    // Quick jump to Invoices (only if not in input mode)
                    if self.input_mode == InputMode::Normal {
                        self.screen = Screen::Invoices;
                        self.needs_refresh = true;
                    }
                }
                'a' => {
                    // Quick jump to Articles (only if not in input mode)
                    if self.input_mode == InputMode::Normal {
                        self.screen = Screen::Articles;
                        self.needs_refresh = true;
                    }
                }
                '/' => {
                    // Alternative shortcut for search
                    self.screen = Screen::Search;
                    self.search_input_mode = true;
                    self.input = self.search_query.clone();
                }
                'd' => self.screen = Screen::Dashboard,
                'h' | '?' => self.screen = Screen::Help,
                _ => {}
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.input_mode == InputMode::Editing || self.search_input_mode {
            self.input.pop();
            // Update search query in real-time
            if self.search_input_mode {
                self.search_query = self.input.clone();
            }
        }
    }

    pub fn next_screen(&mut self) {
        if self.client.is_some() {
            self.screen = match &self.screen {
                Screen::Home => Screen::Dashboard,
                Screen::Dashboard => Screen::Customers,
                Screen::Customers => Screen::Invoices,
                Screen::Invoices => Screen::Articles,
                Screen::Articles => Screen::Search,
                Screen::Search => Screen::Help,
                Screen::Help => Screen::Home,
                _ => return,
            };
        }
    }

    pub fn previous_screen(&mut self) {
        if self.client.is_some() {
            self.screen = match &self.screen {
                Screen::Home => Screen::Help,
                Screen::Dashboard => Screen::Home,
                Screen::Customers => Screen::Dashboard,
                Screen::Invoices => Screen::Customers,
                Screen::Articles => Screen::Invoices,
                Screen::Search => Screen::Articles,
                Screen::Help => Screen::Search,
                _ => return,
            };
        }
    }

    fn start_form(&mut self) {
        self.input_mode = InputMode::Editing;
        self.input.clear();
        self.form_data.clear();
        self.input_field = 0;
    }

    fn start_edit_form(&mut self) {
        self.input_mode = InputMode::Editing;
        self.input.clear();
        self.form_data.clear();
        self.input_field = 0;

        // Pre-populate form data with existing customer data
        if let Screen::CustomerEdit(ref id) = self.screen {
            if let Some(customer) = self.customers.iter().find(|c| c.id.as_deref() == Some(id)) {
                self.form_data.push(customer.name.clone().unwrap_or_default());
                self.form_data.push(customer.email.clone().unwrap_or_default());
                self.form_data.push(customer.phone.clone().unwrap_or_default());
                self.form_data.push(customer.website.clone().unwrap_or_default());
                self.input_field = 4; // Start at the end to submit immediately or edit
            }
        }
    }

    fn start_edit_article_form(&mut self) {
        self.input_mode = InputMode::Editing;
        self.input.clear();
        self.form_data.clear();
        self.input_field = 0;

        // Pre-populate form data with existing article data
        if let Screen::ArticleEdit(ref id) = self.screen {
            if let Some(article) = self.articles.iter().find(|a| a.id.as_deref() == Some(id)) {
                self.form_data.push(article.name.clone().unwrap_or_default());
                self.form_data.push(
                    article
                        .sales_price
                        .map(|p| p.to_string())
                        .unwrap_or_default(),
                );
                self.input_field = 2; // Start at the end to submit immediately or edit
            }
        }
    }

    fn start_edit_invoice_form(&mut self) {
        self.input_mode = InputMode::Editing;
        self.input.clear();
        self.form_data.clear();
        self.input_field = 0;

        // Pre-populate form data with existing invoice data
        if let Screen::InvoiceEdit(ref id) = self.screen {
            if let Some(invoice) = self.invoices.iter().find(|i| i.id.as_deref() == Some(id)) {
                self.form_data.push(invoice.customer_id.clone().unwrap_or_default());
                self.form_data.push(invoice.remarks.clone().unwrap_or_default());
                // Calculate amount from total
                let amount = invoice.total_amount.unwrap_or(0.0);
                self.form_data.push(amount.to_string());
                self.input_field = 3; // Start at the end to submit immediately or edit
            }
        }
    }

    fn validate_email(email: &str) -> bool {
        // Simple email validation
        email.contains('@') && email.contains('.') && email.len() > 3
    }

    fn validate_number(s: &str) -> bool {
        s.parse::<f64>().is_ok() && s.parse::<f64>().unwrap_or(0.0) >= 0.0
    }

    fn validate_current_input(&mut self) -> bool {
        self.validation_error = None;

        match &self.screen {
            Screen::CustomerCreate | Screen::CustomerEdit(_) => {
                match self.input_field {
                    0 => {
                        // Name validation
                        if self.input.trim().is_empty() {
                            self.validation_error = Some("Name cannot be empty".to_string());
                            return false;
                        }
                    }
                    1 => {
                        // Email validation
                        if !Self::validate_email(&self.input) {
                            self.validation_error = Some("Invalid email format".to_string());
                            return false;
                        }
                    }
                    2 => {
                        // Phone validation (optional but if provided should not be empty)
                        if self.input.trim().is_empty() {
                            self.validation_error = Some("Phone cannot be empty".to_string());
                            return false;
                        }
                    }
                    3 => {
                        // Website is optional, no validation
                    }
                    _ => {}
                }
            }
            Screen::ArticleCreate | Screen::ArticleEdit(_) => {
                match self.input_field {
                    0 => {
                        // Name validation
                        if self.input.trim().is_empty() {
                            self.validation_error = Some("Article name cannot be empty".to_string());
                            return false;
                        }
                    }
                    1 => {
                        // Price validation
                        if !Self::validate_number(&self.input) {
                            self.validation_error = Some("Price must be a valid positive number".to_string());
                            return false;
                        }
                    }
                    _ => {}
                }
            }
            Screen::InvoiceCreate | Screen::InvoiceEdit(_) => {
                match self.input_field {
                    0 => {
                        // Customer ID validation
                        if self.input.trim().is_empty() {
                            self.validation_error = Some("Customer ID cannot be empty".to_string());
                            return false;
                        }
                    }
                    1 => {
                        // Description validation
                        if self.input.trim().is_empty() {
                            self.validation_error = Some("Description cannot be empty".to_string());
                            return false;
                        }
                    }
                    2 => {
                        // Amount validation
                        if !Self::validate_number(&self.input) {
                            self.validation_error = Some("Amount must be a valid positive number".to_string());
                            return false;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        true
    }

    fn should_submit_form(&self) -> bool {
        match self.screen {
            Screen::CustomerCreate | Screen::CustomerEdit(_) => self.input_field >= 4, // name, email, phone, website
            Screen::InvoiceCreate | Screen::InvoiceEdit(_) => self.input_field >= 3,   // customer_id, description, amount
            Screen::ArticleCreate | Screen::ArticleEdit(_) => self.input_field >= 2,  // name, price
            _ => false,
        }
    }

    async fn submit_form(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            match &self.screen.clone() {
                Screen::CustomerCreate => {
                    let customer = Customer {
                        name: Some(self.form_data[0].clone()),
                        email: Some(self.form_data[1].clone()),
                        phone: Some(self.form_data[2].clone()),
                        website: if self.form_data[3].is_empty() {
                            None
                        } else {
                            Some(self.form_data[3].clone())
                        },
                        is_active: Some(true),
                        ..Default::default()
                    };

                    match client.customers().create(&customer).await {
                        Ok(_) => {
                            self.set_status("Customer created successfully".to_string());
                            self.screen = Screen::Customers;
                            self.load_customers().await?;
                        }
                        Err(e) => {
                            self.set_error(format!("Failed to create customer: {}", e));
                        }
                    }
                }
                Screen::CustomerEdit(id) => {
                    let customer = Customer {
                        id: Some(id.clone()),
                        name: Some(self.form_data[0].clone()),
                        email: Some(self.form_data[1].clone()),
                        phone: Some(self.form_data[2].clone()),
                        website: if self.form_data[3].is_empty() {
                            None
                        } else {
                            Some(self.form_data[3].clone())
                        },
                        is_active: Some(true),
                        ..Default::default()
                    };

                    match client.customers().update(id, &customer).await {
                        Ok(_) => {
                            self.set_status("Customer updated successfully".to_string());
                            self.screen = Screen::CustomerDetail(id.clone());
                            self.load_customers().await?;
                        }
                        Err(e) => {
                            self.set_error(format!("Failed to update customer: {}", e));
                        }
                    }
                }
                Screen::ArticleCreate => {
                    let price: f64 = self.form_data[1].parse().unwrap_or(0.0);
                    let article = Article {
                        name: Some(self.form_data[0].clone()),
                        sales_price: Some(price),
                        is_active: Some(true),
                        ..Default::default()
                    };

                    match client.articles().create(&article).await {
                        Ok(_) => {
                            self.set_status("Article created successfully".to_string());
                            self.screen = Screen::Articles;
                            self.load_articles().await?;
                        }
                        Err(e) => {
                            self.set_error(format!("Failed to create article: {}", e));
                        }
                    }
                }
                Screen::ArticleEdit(id) => {
                    let price: f64 = self.form_data[1].parse().unwrap_or(0.0);
                    let article = Article {
                        id: Some(id.clone()),
                        name: Some(self.form_data[0].clone()),
                        sales_price: Some(price),
                        is_active: Some(true),
                        ..Default::default()
                    };

                    match client.articles().update(id, &article).await {
                        Ok(_) => {
                            self.set_status("Article updated successfully".to_string());
                            self.screen = Screen::ArticleDetail(id.clone());
                            self.load_articles().await?;
                        }
                        Err(e) => {
                            self.set_error(format!("Failed to update article: {}", e));
                        }
                    }
                }
                Screen::InvoiceCreate => {
                    if self.form_data.len() >= 3 {
                        let amount: f64 = self.form_data[2].parse().unwrap_or(0.0);
                        let invoice = Invoice {
                            customer_id: Some(self.form_data[0].clone()),
                            remarks: Some(self.form_data[1].clone()),
                            rows: vec![InvoiceRow {
                                text: Some(self.form_data[1].clone()),
                                quantity: Some(1.0),
                                unit_price: Some(amount),
                                ..Default::default()
                            }],
                            ..Default::default()
                        };

                        match client.invoices().create(&invoice).await {
                            Ok(_) => {
                                self.set_status("Invoice created successfully".to_string());
                                self.screen = Screen::Invoices;
                                self.load_invoices().await?;
                            }
                            Err(e) => {
                                self.set_error(format!("Failed to create invoice: {}", e));
                            }
                        }
                    }
                }
                Screen::InvoiceEdit(id) => {
                    if self.form_data.len() >= 3 {
                        let amount: f64 = self.form_data[2].parse().unwrap_or(0.0);
                        let invoice = Invoice {
                            id: Some(id.clone()),
                            customer_id: Some(self.form_data[0].clone()),
                            remarks: Some(self.form_data[1].clone()),
                            rows: vec![InvoiceRow {
                                text: Some(self.form_data[1].clone()),
                                quantity: Some(1.0),
                                unit_price: Some(amount),
                                ..Default::default()
                            }],
                            ..Default::default()
                        };

                        match client.invoices().update(id, &invoice).await {
                            Ok(_) => {
                                self.set_status("Invoice updated successfully".to_string());
                                self.screen = Screen::InvoiceDetail(id.clone());
                                self.load_invoices().await?;
                            }
                            Err(e) => {
                                self.set_error(format!("Failed to update invoice: {}", e));
                            }
                        }
                    }
                }
                _ => {}
            }
            self.form_data.clear();
            self.input_field = 0;
        }
        Ok(())
    }

    pub fn cycle_customer_sort(&mut self) {
        use CustomerSortField::*;
        self.customer_sort_field = match self.customer_sort_field {
            Name => Email,
            Email => CustomerNumber,
            CustomerNumber => {
                // Cycle sort order instead
                self.customer_sort_order = match self.customer_sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
                Name
            }
        };
        self.sort_customers();
    }

    pub fn cycle_invoice_sort(&mut self) {
        use InvoiceSortField::*;
        self.invoice_sort_field = match self.invoice_sort_field {
            InvoiceNumber => CustomerID,
            CustomerID => Date,
            Date => Amount,
            Amount => {
                // Cycle sort order instead
                self.invoice_sort_order = match self.invoice_sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
                InvoiceNumber
            }
        };
        self.sort_invoices();
    }

    pub fn cycle_article_sort(&mut self) {
        use ArticleSortField::*;
        self.article_sort_field = match self.article_sort_field {
            Name => Price,
            Price => ArticleNumber,
            ArticleNumber => {
                // Cycle sort order instead
                self.article_sort_order = match self.article_sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
                Name
            }
        };
        self.sort_articles();
    }

    fn sort_customers(&mut self) {
        use CustomerSortField::*;
        match self.customer_sort_field {
            Name => {
                self.customers.sort_by(|a, b| {
                    let ord = a.name.cmp(&b.name);
                    match self.customer_sort_order {
                        SortOrder::Ascending => ord,
                        SortOrder::Descending => ord.reverse(),
                    }
                });
            }
            Email => {
                self.customers.sort_by(|a, b| {
                    let ord = a.email.cmp(&b.email);
                    match self.customer_sort_order {
                        SortOrder::Ascending => ord,
                        SortOrder::Descending => ord.reverse(),
                    }
                });
            }
            CustomerNumber => {
                self.customers.sort_by(|a, b| {
                    let ord = a.customer_number.cmp(&b.customer_number);
                    match self.customer_sort_order {
                        SortOrder::Ascending => ord,
                        SortOrder::Descending => ord.reverse(),
                    }
                });
            }
        }
    }

    fn sort_invoices(&mut self) {
        use InvoiceSortField::*;
        match self.invoice_sort_field {
            InvoiceNumber => {
                self.invoices.sort_by(|a, b| {
                    let ord = a.invoice_number.cmp(&b.invoice_number);
                    match self.invoice_sort_order {
                        SortOrder::Ascending => ord,
                        SortOrder::Descending => ord.reverse(),
                    }
                });
            }
            CustomerID => {
                self.invoices.sort_by(|a, b| {
                    let ord = a.customer_id.cmp(&b.customer_id);
                    match self.invoice_sort_order {
                        SortOrder::Ascending => ord,
                        SortOrder::Descending => ord.reverse(),
                    }
                });
            }
            Date => {
                self.invoices.sort_by(|a, b| {
                    let ord = a.invoice_date.cmp(&b.invoice_date);
                    match self.invoice_sort_order {
                        SortOrder::Ascending => ord,
                        SortOrder::Descending => ord.reverse(),
                    }
                });
            }
            Amount => {
                self.invoices.sort_by(|a, b| {
                    let ord = a.total_amount.partial_cmp(&b.total_amount).unwrap_or(std::cmp::Ordering::Equal);
                    match self.invoice_sort_order {
                        SortOrder::Ascending => ord,
                        SortOrder::Descending => ord.reverse(),
                    }
                });
            }
        }
    }

    fn sort_articles(&mut self) {
        use ArticleSortField::*;
        match self.article_sort_field {
            Name => {
                self.articles.sort_by(|a, b| {
                    let ord = a.name.cmp(&b.name);
                    match self.article_sort_order {
                        SortOrder::Ascending => ord,
                        SortOrder::Descending => ord.reverse(),
                    }
                });
            }
            Price => {
                self.articles.sort_by(|a, b| {
                    let ord = a.sales_price.partial_cmp(&b.sales_price).unwrap_or(std::cmp::Ordering::Equal);
                    match self.article_sort_order {
                        SortOrder::Ascending => ord,
                        SortOrder::Descending => ord.reverse(),
                    }
                });
            }
            ArticleNumber => {
                self.articles.sort_by(|a, b| {
                    let ord = a.article_number.cmp(&b.article_number);
                    match self.article_sort_order {
                        SortOrder::Ascending => ord,
                        SortOrder::Descending => ord.reverse(),
                    }
                });
            }
        }
    }

    pub async fn load_customers(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            self.loading = true;
            let params = PaginationParams::new()
                .pagesize(self.page_size)
                .page(self.current_page);
            match client.customers().list(Some(params)).await {
                Ok(response) => {
                    self.customers = response.data;
                    self.sort_customers(); // Apply current sort
                    // Update total pages based on metadata if available
                    // For now, just assume there might be more pages
                    self.total_pages = self.current_page + 1;
                    self.loading = false;
                    self.error_message = None;
                }
                Err(e) => {
                    self.set_error(format!("Failed to load customers: {}", e));
                    self.loading = false;
                }
            }
        }
        Ok(())
    }

    pub async fn load_invoices(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            self.loading = true;
            let params = PaginationParams::new()
                .pagesize(self.page_size)
                .page(self.current_page);
            match client.invoices().list(Some(params)).await {
                Ok(response) => {
                    self.invoices = response.data;
                    self.sort_invoices(); // Apply current sort
                    self.total_pages = self.current_page + 1;
                    self.loading = false;
                    self.error_message = None;
                }
                Err(e) => {
                    self.set_error(format!("Failed to load invoices: {}", e));
                    self.loading = false;
                }
            }
        }
        Ok(())
    }

    pub async fn load_articles(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            self.loading = true;
            let params = PaginationParams::new()
                .pagesize(self.page_size)
                .page(self.current_page);
            match client.articles().list(Some(params)).await {
                Ok(response) => {
                    self.articles = response.data;
                    self.sort_articles(); // Apply current sort
                    self.stats_total_articles = self.articles.len();
                    self.total_pages = self.current_page + 1;
                    self.loading = false;
                    self.error_message = None;
                }
                Err(e) => {
                    self.set_error(format!("Failed to load articles: {}", e));
                    self.loading = false;
                }
            }
        }
        Ok(())
    }

    async fn handle_dashboard_enter(&mut self) -> Result<()> {
        match self.selected_customer {
            0 => {
                self.screen = Screen::Customers;
                self.load_customers().await?;
            }
            1 => {
                self.screen = Screen::Invoices;
                self.load_invoices().await?;
            }
            2 => {
                self.screen = Screen::Articles;
                self.load_articles().await?;
            }
            3 => {
                self.screen = Screen::Export;
            }
            _ => {}
        }
        Ok(())
    }

    async fn perform_search(&mut self) -> Result<()> {
        if !self.search_query.is_empty() {
            if let Some(_client) = &self.client {
                self.loading = true;

                // For now, search is just filtering from loaded data
                // In a real implementation, you would use the API search endpoints
                if matches!(self.search_mode, SearchMode::Customers | SearchMode::All) {
                    let query = self.search_query.to_lowercase();
                    self.search_results_customers = self
                        .customers
                        .iter()
                        .filter(|c| {
                            c.name
                                .as_ref()
                                .map(|n| n.to_lowercase().contains(&query))
                                .unwrap_or(false)
                                || c.email
                                    .as_ref()
                                    .map(|e| e.to_lowercase().contains(&query))
                                    .unwrap_or(false)
                        })
                        .cloned()
                        .collect();
                }

                if matches!(self.search_mode, SearchMode::Invoices | SearchMode::All) {
                    let query = self.search_query.to_lowercase();
                    self.search_results_invoices = self
                        .invoices
                        .iter()
                        .filter(|inv| {
                            inv.customer_id
                                .as_ref()
                                .map(|id| id.to_lowercase().contains(&query))
                                .unwrap_or(false)
                                || inv
                                    .remarks
                                    .as_ref()
                                    .map(|r| r.to_lowercase().contains(&query))
                                    .unwrap_or(false)
                        })
                        .cloned()
                        .collect();
                }

                self.loading = false;
                self.set_status(format!(
                    "Found {} customers, {} invoices",
                    self.search_results_customers.len(),
                    self.search_results_invoices.len()
                ));
            }
        }
        Ok(())
    }

    fn export_data(&mut self) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let mut messages = Vec::new();

        match self.export_format {
            ExportFormat::Json => {
                // Export customers
                if !self.customers.is_empty() {
                    let filename = format!("customers_export_{}.json", timestamp);
                    let json = serde_json::to_string_pretty(&self.customers)?;
                    std::fs::write(&filename, json)?;
                    messages.push(format!("{} customers", self.customers.len()));
                }

                // Export invoices
                if !self.invoices.is_empty() {
                    let filename = format!("invoices_export_{}.json", timestamp);
                    let json = serde_json::to_string_pretty(&self.invoices)?;
                    std::fs::write(&filename, json)?;
                    messages.push(format!("{} invoices", self.invoices.len()));
                }

                // Export articles
                if !self.articles.is_empty() {
                    let filename = format!("articles_export_{}.json", timestamp);
                    let json = serde_json::to_string_pretty(&self.articles)?;
                    std::fs::write(&filename, json)?;
                    messages.push(format!("{} articles", self.articles.len()));
                }
            }
            ExportFormat::Csv => {
                // Export customers to CSV
                if !self.customers.is_empty() {
                    let filename = format!("customers_export_{}.csv", timestamp);
                    let mut wtr = csv::Writer::from_path(&filename)?;

                    // Write header
                    wtr.write_record(&[
                        "ID",
                        "Customer Number",
                        "Name",
                        "Email",
                        "Phone",
                        "Website",
                        "Is Active",
                    ])?;

                    // Write data
                    for customer in &self.customers {
                        wtr.write_record(&[
                            customer.id.as_deref().unwrap_or(""),
                            &customer.customer_number.as_ref().map(|n| n.to_string()).unwrap_or_default(),
                            customer.name.as_deref().unwrap_or(""),
                            customer.email.as_deref().unwrap_or(""),
                            customer.phone.as_deref().unwrap_or(""),
                            customer.website.as_deref().unwrap_or(""),
                            &customer.is_active.map(|a| a.to_string()).unwrap_or_default(),
                        ])?;
                    }
                    wtr.flush()?;
                    messages.push(format!("{} customers", self.customers.len()));
                }

                // Export invoices to CSV
                if !self.invoices.is_empty() {
                    let filename = format!("invoices_export_{}.csv", timestamp);
                    let mut wtr = csv::Writer::from_path(&filename)?;

                    // Write header
                    wtr.write_record(&[
                        "ID",
                        "Invoice Number",
                        "Customer ID",
                        "Date",
                        "Total Amount",
                        "VAT Amount",
                        "Total Including VAT",
                        "Remarks",
                    ])?;

                    // Write data
                    for invoice in &self.invoices {
                        wtr.write_record(&[
                            invoice.id.as_deref().unwrap_or(""),
                            &invoice.invoice_number.as_ref().map(|n| n.to_string()).unwrap_or_default(),
                            invoice.customer_id.as_deref().unwrap_or(""),
                            &invoice
                                .invoice_date
                                .as_ref()
                                .map(|d| d.format("%Y-%m-%d").to_string())
                                .unwrap_or_default(),
                            &invoice.total_amount.map(|t| t.to_string()).unwrap_or_default(),
                            &invoice.total_vat_amount.map(|t| t.to_string()).unwrap_or_default(),
                            &invoice
                                .total_amount_including_vat
                                .map(|t| t.to_string())
                                .unwrap_or_default(),
                            invoice.remarks.as_deref().unwrap_or(""),
                        ])?;
                    }
                    wtr.flush()?;
                    messages.push(format!("{} invoices", self.invoices.len()));
                }

                // Export articles to CSV
                if !self.articles.is_empty() {
                    let filename = format!("articles_export_{}.csv", timestamp);
                    let mut wtr = csv::Writer::from_path(&filename)?;

                    // Write header
                    wtr.write_record(&[
                        "ID",
                        "Article Number",
                        "Name",
                        "Unit",
                        "Sales Price",
                        "Purchase Price",
                        "Is Active",
                    ])?;

                    // Write data
                    for article in &self.articles {
                        wtr.write_record(&[
                            article.id.as_deref().unwrap_or(""),
                            &article.article_number.as_ref().map(|n| n.to_string()).unwrap_or_default(),
                            article.name.as_deref().unwrap_or(""),
                            article.unit.as_deref().unwrap_or(""),
                            &article.sales_price.map(|p| p.to_string()).unwrap_or_default(),
                            &article.purchase_price.map(|p| p.to_string()).unwrap_or_default(),
                            &article.is_active.map(|a| a.to_string()).unwrap_or_default(),
                        ])?;
                    }
                    wtr.flush()?;
                    messages.push(format!("{} articles", self.articles.len()));
                }
            }
        }

        if !messages.is_empty() {
            let format_name = match self.export_format {
                ExportFormat::Json => "JSON",
                ExportFormat::Csv => "CSV",
            };
            self.set_status(format!("Exported to {}: {}", format_name, messages.join(", ")));
        } else {
            self.set_error("No data to export".to_string());
        }

        Ok(())
    }

    pub async fn load_dashboard_stats(&mut self) -> Result<()> {
        if self.client.is_some() {
            // Load minimal data to get counts
            self.load_customers().await?;
            self.load_invoices().await?;
            self.load_articles().await?;

            // Basic counts
            self.stats_total_customers = self.customers.len();
            self.stats_total_invoices = self.invoices.len();
            self.stats_total_articles = self.articles.len();

            // Active customers
            self.stats_active_customers = self.customers.iter().filter(|c| c.is_active.unwrap_or(false)).count();

            // Revenue calculations
            let total: f64 = self.invoices.iter()
                .filter_map(|inv| inv.total_amount_including_vat)
                .sum();
            self.stats_total_revenue = total;
            self.stats_average_invoice = if self.stats_total_invoices > 0 {
                total / self.stats_total_invoices as f64
            } else {
                0.0
            };

            // Recent invoices (7 and 30 days)
            let now = chrono::Utc::now();
            let seven_days_ago = now - chrono::Duration::days(7);
            let thirty_days_ago = now - chrono::Duration::days(30);

            self.stats_recent_invoices_7d = self.invoices.iter()
                .filter(|inv| {
                    if let Some(date) = inv.invoice_date {
                        date >= seven_days_ago
                    } else {
                        false
                    }
                })
                .count();

            self.stats_recent_invoices_30d = self.invoices.iter()
                .filter(|inv| {
                    if let Some(date) = inv.invoice_date {
                        date >= thirty_days_ago
                    } else {
                        false
                    }
                })
                .count();
        }
        Ok(())
    }

    pub async fn refresh_if_needed(&mut self) -> Result<()> {
        if !self.needs_refresh {
            return Ok(());
        }

        match self.screen {
            Screen::Customers => self.load_customers().await?,
            Screen::Invoices => self.load_invoices().await?,
            Screen::Articles => self.load_articles().await?,
            Screen::Dashboard => self.load_dashboard_stats().await?,
            _ => {}
        }

        self.needs_refresh = false;
        Ok(())
    }

    pub fn tick(&mut self) {
        // Decrement message timer and clear messages when timer reaches 0
        if self.message_timer > 0 {
            self.message_timer -= 1;
            if self.message_timer == 0 {
                self.status_message = None;
                self.error_message = None;
            }
        }
    }

    fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
        self.message_timer = 30; // 3 seconds at 10 ticks per second
    }

    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.message_timer = 50; // 5 seconds at 10 ticks per second
    }

    fn execute_delete(&mut self) {
        if let Some((entity_type, id)) = self.confirm_delete.take() {
            match entity_type.as_str() {
                "customer" => self.delete_customer(id),
                "invoice" => self.delete_invoice(id),
                "article" => self.delete_article(id),
                _ => {}
            }
        }
    }

    fn delete_customer(&mut self, id: String) {
        if let Some(client) = &self.client {
            let client = client.clone();
            let id_clone = id.clone();
            tokio::spawn(async move {
                let _ = client.customers().delete(&id_clone).await;
            });
            self.set_status("Customer deleted".to_string());
            self.screen = Screen::Customers;
            self.needs_refresh = true;
        }
    }

    fn delete_invoice(&mut self, id: String) {
        if let Some(client) = &self.client {
            let client = client.clone();
            let id_clone = id.clone();
            tokio::spawn(async move {
                let _ = client.invoices().delete(&id_clone).await;
            });
            self.set_status("Invoice deleted".to_string());
            self.screen = Screen::Invoices;
            self.needs_refresh = true;
        }
    }

    fn delete_article(&mut self, id: String) {
        if let Some(client) = &self.client {
            let client = client.clone();
            let id_clone = id.clone();
            tokio::spawn(async move {
                let _ = client.articles().delete(&id_clone).await;
            });
            self.set_status("Article deleted".to_string());
            self.screen = Screen::Articles;
            self.needs_refresh = true;
        }
    }

    async fn start_oauth(&mut self) -> Result<()> {
        self.oauth_waiting = true;
        self.status_message = Some("Starting OAuth flow...".to_string());

        // Get credentials from environment
        let client_id = std::env::var("SPIRIS_CLIENT_ID")
            .unwrap_or_else(|_| "your_client_id".to_string());
        let client_secret = std::env::var("SPIRIS_CLIENT_SECRET")
            .unwrap_or_else(|_| "your_client_secret".to_string());

        let oauth_config = spiris_bokforing::auth::OAuth2Config::new(
            client_id,
            client_secret,
            "http://localhost:8080/callback".to_string(),
        );

        let handler = spiris_bokforing::auth::OAuth2Handler::new(oauth_config)?;
        let (auth_url, _csrf, _verifier) = handler.authorize_url();

        self.oauth_url = Some(auth_url);
        self.status_message = Some("Copy the URL above and open in browser".to_string());

        Ok(())
    }

    fn load_token() -> Result<AccessToken> {
        let token_path = Self::token_path();
        let contents = std::fs::read_to_string(token_path)?;
        let token: AccessToken = serde_json::from_str(&contents)?;
        Ok(token)
    }

    pub fn save_token(&self) -> Result<()> {
        if let Some(token) = &self.token {
            let token_path = Self::token_path();
            let json = serde_json::to_string_pretty(token)?;
            std::fs::write(token_path, json)?;
        }
        Ok(())
    }

    fn token_path() -> PathBuf {
        let mut path = std::env::current_dir().unwrap();
        path.push(".spiris_token.json");
        path
    }
}

impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            screen: self.screen.clone(),
            previous_screen: self.previous_screen.clone(),
            input_mode: self.input_mode.clone(),
            client: self.client.as_ref().map(|c| Client::new(c.get_access_token().clone())),
            token: self.token.clone(),
            customers: self.customers.clone(),
            selected_customer: self.selected_customer,
            invoices: self.invoices.clone(),
            selected_invoice: self.selected_invoice,
            articles: self.articles.clone(),
            selected_article: self.selected_article,
            current_page: self.current_page,
            total_pages: self.total_pages,
            page_size: self.page_size,
            search_query: self.search_query.clone(),
            search_results_customers: self.search_results_customers.clone(),
            search_results_invoices: self.search_results_invoices.clone(),
            search_mode: self.search_mode.clone(),
            search_input_mode: self.search_input_mode,
            export_format: self.export_format.clone(),
            export_selection: self.export_selection,
            customer_sort_field: self.customer_sort_field.clone(),
            customer_sort_order: self.customer_sort_order.clone(),
            invoice_sort_field: self.invoice_sort_field.clone(),
            invoice_sort_order: self.invoice_sort_order.clone(),
            article_sort_field: self.article_sort_field.clone(),
            article_sort_order: self.article_sort_order.clone(),
            stats_total_customers: self.stats_total_customers,
            stats_total_invoices: self.stats_total_invoices,
            stats_total_articles: self.stats_total_articles,
            stats_active_customers: self.stats_active_customers,
            stats_total_revenue: self.stats_total_revenue,
            stats_average_invoice: self.stats_average_invoice,
            stats_recent_invoices_7d: self.stats_recent_invoices_7d,
            stats_recent_invoices_30d: self.stats_recent_invoices_30d,
            input: self.input.clone(),
            input_field: self.input_field,
            form_data: self.form_data.clone(),
            status_message: self.status_message.clone(),
            error_message: self.error_message.clone(),
            message_timer: self.message_timer,
            validation_error: self.validation_error.clone(),
            loading: self.loading,
            needs_refresh: self.needs_refresh,
            confirm_delete: self.confirm_delete.clone(),
            oauth_url: self.oauth_url.clone(),
            oauth_waiting: self.oauth_waiting,
        }
    }
}
