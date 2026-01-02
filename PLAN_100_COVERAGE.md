# Plan: 100% API Coverage for Spiris/Visma eAccounting

Based on research from the [Visma Developer Portal](https://developer.visma.com/api/eaccounting), [OpenAPI spec](https://github.com/pwitab/visma/blob/master/swagger.json), and [Python SDK docs](https://visma.readthedocs.io/en/latest/models.html).

## Current State

| Endpoint | Status |
|----------|--------|
| Customers | ✓ Complete |
| Customer Invoices | ✓ Complete |
| Articles | ✓ Complete |

**Coverage: 3 of ~35 endpoints (9%)**

---

## Phase 1: Sales & Customer Extensions

High-value extensions to existing sales functionality.

### 1.1 Customer Invoice Drafts
**Path:** `/v2/customerinvoicedrafts`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}, POST/{id}/convert
**Types:** `CustomerInvoiceDraft`, `CustomerInvoiceDraftRow`
**Priority:** HIGH - Enables draft workflow before finalizing invoices

### 1.2 Customer Ledger Items
**Path:** `/v2/customerledgeritems`
**Operations:** GET, POST, GET/{id}, POST/customerledgeritemswithvoucher
**Types:** `CustomerLedgerItem`
**Priority:** HIGH - Payment tracking and AR management

### 1.3 Customer Labels
**Path:** `/v2/customerlabels`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `CustomerLabel`
**Priority:** MEDIUM - Customer categorization

### 1.4 Invoice Payments
**Path:** `/v2/customerinvoices/{invoiceId}/payments`
**Operations:** POST
**Types:** `InvoicePayment`
**Priority:** HIGH - Already have invoices, payment is natural extension

### 1.5 Invoice PDF
**Path:** `/v2/customerinvoices/{invoiceId}/pdf`
**Operations:** GET (returns binary)
**Priority:** HIGH - Common use case for invoice export

### 1.6 E-Invoice
**Path:** `/v2/customerinvoices/{invoiceId}/einvoice`
**Operations:** POST
**Priority:** MEDIUM - Electronic invoice sending

---

## Phase 2: Supplier/Purchasing

Complete purchasing side to mirror sales.

### 2.1 Suppliers
**Path:** `/v2/suppliers`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `Supplier` (similar to `Customer`)
**Priority:** HIGH - Core entity

### 2.2 Supplier Invoices
**Path:** `/v2/supplierinvoices`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `SupplierInvoice`, `SupplierInvoiceRow`
**Priority:** HIGH - AP management

### 2.3 Supplier Invoice Drafts
**Path:** `/v2/supplierinvoicedrafts`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}, POST/{id}/convert
**Types:** `SupplierInvoiceDraft`
**Priority:** MEDIUM - Draft workflow

### 2.4 Supplier Ledger Items
**Path:** `/v2/supplierledgeritems`
**Operations:** GET, POST, GET/{id}
**Types:** `SupplierLedgerItem`
**Priority:** MEDIUM - AP tracking

### 2.5 Supplier Labels
**Path:** `/v2/supplierlabels`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `SupplierLabel`
**Priority:** LOW - Categorization

---

## Phase 3: Accounting & Finance

Core accounting functionality.

### 3.1 Accounts
**Path:** `/v2/accounts`
**Operations:** GET, POST, GET/{fiscalyearId}, GET/{fiscalyearId}/{accountNumber}, PUT/{fiscalyearId}/{accountNumber}
**Additional:** `/v2/accounts/standardaccounts`
**Types:** `Account`
**Priority:** HIGH - Chart of accounts

### 3.2 Account Balances
**Path:** `/v2/accountbalances/{date}`, `/v2/accountbalances/{accountNumber}/{date}`
**Operations:** GET
**Types:** `AccountBalance`
**Priority:** HIGH - Financial reporting

### 3.3 Account Types
**Path:** `/v2/accountTypes`
**Operations:** GET
**Types:** `AccountType`
**Priority:** MEDIUM - Reference data

### 3.4 Fiscal Years
**Path:** `/v2/fiscalyears`
**Operations:** GET, POST, GET/{id}
**Additional:** `/v2/fiscalyears/openingbalances` (GET, PUT)
**Types:** `FiscalYear`, `OpeningBalance`
**Priority:** HIGH - Period management

### 3.5 VAT Codes
**Path:** `/v2/vatcodes`
**Operations:** GET, GET/{id}
**Types:** `VatCode`
**Priority:** HIGH - Tax handling

### 3.6 VAT Reports
**Path:** `/v2/vatreports`
**Operations:** GET, GET/{id}
**Additional:** `/v2/approval/vatreport/{id}` (PUT)
**Types:** `VatReport`
**Priority:** MEDIUM - Tax reporting

### 3.7 Vouchers
**Path:** `/v2/vouchers`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `Voucher`, `VoucherRow`
**Priority:** HIGH - Journal entries

---

## Phase 4: Banking & Payments

Banking integration.

### 4.1 Bank Accounts
**Path:** `/v2/bankaccounts`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `BankAccount`
**Priority:** HIGH - Payment processing

### 4.2 Banks
**Path:** `/v2/banks`
**Operations:** GET
**Types:** `Bank`
**Priority:** LOW - Reference data

### 4.3 Foreign Payment Codes
**Path:** `/v2/foreignpaymentcodes`
**Operations:** GET
**Types:** `ForeignPaymentCode`
**Priority:** LOW - International payments

### 4.4 Terms of Payment
**Path:** `/v2/termsofpayments`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `TermsOfPayment`
**Priority:** MEDIUM - Payment terms

---

## Phase 5: Projects & Cost Centers

Cost allocation and project tracking.

### 5.1 Projects
**Path:** `/v2/projects`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `Project`
**Priority:** MEDIUM - Project accounting

### 5.2 Cost Centers
**Path:** `/v2/costcenters`
**Operations:** GET, PUT/{id}
**Types:** `CostCenter`
**Priority:** MEDIUM - Cost allocation

### 5.3 Cost Center Items
**Path:** `/v2/costcenteritems`
**Operations:** GET, POST, GET/{id}, PUT/{id}
**Types:** `CostCenterItem`
**Priority:** MEDIUM - Cost allocation details

### 5.4 Allocation Periods
**Path:** `/v2/allocationperiods`
**Operations:** GET, POST, GET/{id}
**Types:** `AllocationPeriod`
**Priority:** LOW - Period allocation

---

## Phase 6: Orders & Delivery

Sales order workflow.

### 6.1 Orders
**Path:** `/v2/orders`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `Order`, `OrderRow`
**Priority:** MEDIUM - Sales orders

### 6.2 Quotations
**Path:** `/v2/quotations`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `Quotation`, `QuotationRow`
**Priority:** MEDIUM - Sales quotes

### 6.3 Delivery Methods
**Path:** `/v2/deliverymethods`
**Operations:** GET, GET/{id}
**Types:** `DeliveryMethod`
**Priority:** LOW - Reference data

### 6.4 Delivery Terms
**Path:** `/v2/deliveryterms`
**Operations:** GET, GET/{id}
**Types:** `DeliveryTerm`
**Priority:** LOW - Reference data

---

## Phase 7: Articles Extensions

### 7.1 Article Labels
**Path:** `/v2/articlelabels`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `ArticleLabel`
**Priority:** MEDIUM - Article categorization

### 7.2 Article Account Codings
**Path:** `/v2/articleaccountcodings`
**Operations:** GET, GET/{id}
**Types:** `ArticleAccountCoding`
**Priority:** LOW - Account mapping

### 7.3 Units
**Path:** `/v2/units`
**Operations:** GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
**Types:** `Unit`
**Priority:** MEDIUM - Measurement units

---

## Phase 8: Documents & Attachments

### 8.1 Attachments
**Path:** `/v2/attachments`
**Operations:** GET, POST, GET/{id}, DELETE/{id}
**Types:** `Attachment`
**Priority:** MEDIUM - File handling

### 8.2 Attachment Links
**Path:** `/v2/attachmentlinks`
**Operations:** POST, DELETE/{id}
**Types:** `AttachmentLink`
**Priority:** MEDIUM - Link documents to entities

### 8.3 Documents
**Path:** `/v2/documents/{id}`
**Operations:** GET
**Types:** `Document`
**Priority:** LOW - Document retrieval

---

## Phase 9: Settings & Reference Data

### 9.1 Company Settings
**Path:** `/v2/companysettings`
**Operations:** GET, PUT
**Types:** `CompanySettings`, `CompanyTexts`
**Priority:** LOW - Company configuration

### 9.2 Countries
**Path:** `/v2/countries`
**Operations:** GET, GET/{code}
**Types:** `Country`
**Priority:** LOW - Reference data

### 9.3 Currencies
**Path:** `/v2/currencies`
**Operations:** GET
**Types:** `Currency`
**Priority:** LOW - Reference data

### 9.4 Users
**Path:** `/v2/users`
**Operations:** GET, GET/{id}
**Types:** `User`
**Priority:** LOW - User info

---

## Phase 10: Messaging & Misc

### 10.1 Message Threads
**Path:** `/v2/messagethreads/{id}`
**Operations:** GET, PUT, POST
**Types:** `MessageThread`, `Message`
**Priority:** LOW - Internal messaging

### 10.2 Approval Endpoints
**Paths:** `/v2/approval/vatreport/{id}`, `/v2/approval/supplierinvoice/{id}`
**Operations:** PUT
**Priority:** LOW - Workflow approvals

---

## Implementation Order (Priority-Based)

### Tier 1 - Core Business Operations
1. Customer Invoice Drafts + Convert
2. Invoice Payments
3. Invoice PDF export
4. Suppliers (CRUD)
5. Supplier Invoices (CRUD)
6. Bank Accounts
7. Vouchers

### Tier 2 - Financial Reporting
8. Accounts
9. Account Balances
10. Fiscal Years + Opening Balances
11. VAT Codes
12. Customer Ledger Items
13. Supplier Ledger Items

### Tier 3 - Extended Sales/Purchasing
14. Orders
15. Quotations
16. Supplier Invoice Drafts
17. Terms of Payment
18. Projects
19. Cost Centers

### Tier 4 - Reference Data & Settings
20. Account Types
21. VAT Reports
22. Article Labels
23. Customer Labels
24. Supplier Labels
25. Units
26. Delivery Methods/Terms
27. Countries/Currencies
28. Company Settings
29. Users

### Tier 5 - Documents & Misc
30. Attachments
31. Attachment Links
32. Documents
33. E-Invoice
34. Message Threads
35. Approval endpoints

---

## Estimated New Types Required

| Category | Types |
|----------|-------|
| Suppliers | Supplier, SupplierInvoice, SupplierInvoiceRow, SupplierInvoiceDraft, SupplierLedgerItem, SupplierLabel |
| Drafts | CustomerInvoiceDraft, CustomerInvoiceDraftRow |
| Ledger | CustomerLedgerItem, InvoicePayment |
| Accounting | Account, AccountBalance, AccountType, FiscalYear, OpeningBalance, VatCode, VatReport, Voucher, VoucherRow |
| Banking | BankAccount, Bank, ForeignPaymentCode, TermsOfPayment |
| Projects | Project, CostCenter, CostCenterItem, AllocationPeriod |
| Orders | Order, OrderRow, Quotation, QuotationRow |
| Delivery | DeliveryMethod, DeliveryTerm |
| Articles | ArticleLabel, ArticleAccountCoding, Unit |
| Documents | Attachment, AttachmentLink, Document |
| Settings | CompanySettings, CompanyTexts, Country, Currency, User |
| Misc | MessageThread, Message, Label |

**Total: ~45 new types**

---

## Technical Considerations

### Binary Response Handling
- PDF endpoint returns binary data, need `get_bytes()` method on Client

### Nested Endpoints
- `/v2/customerinvoices/{id}/payments` - Consider `InvoicesEndpoint::payments(invoice_id)`
- `/v2/customerinvoicedrafts/{id}/convert` - POST with no body

### Consistent Pattern
Each endpoint module should follow existing pattern:
```rust
pub struct SuppliersEndpoint<'a> {
    client: &'a Client,
}

impl<'a> SuppliersEndpoint<'a> {
    pub async fn list(&self, params: Option<PaginationParams>) -> Result<PaginatedResponse<Supplier>>
    pub async fn get(&self, id: &str) -> Result<Supplier>
    pub async fn create(&self, supplier: &Supplier) -> Result<Supplier>
    pub async fn update(&self, id: &str, supplier: &Supplier) -> Result<Supplier>
    pub async fn delete(&self, id: &str) -> Result<()>
    pub async fn search(&self, query: QueryParams, pagination: Option<PaginationParams>) -> Result<PaginatedResponse<Supplier>>
}
```

### Testing Strategy
- Mock all endpoints using existing mockito setup
- Test serialization/deserialization for all new types
- Add integration test examples in `examples/`

---

## Success Metrics

| Phase | Endpoints | Cumulative Coverage |
|-------|-----------|---------------------|
| Current | 3 | 9% |
| Phase 1 | +6 | 26% |
| Phase 2 | +5 | 40% |
| Phase 3 | +7 | 60% |
| Phase 4 | +4 | 71% |
| Phase 5 | +4 | 83% |
| Phase 6 | +4 | 94% |
| Phase 7-10 | +6 | 100% |
