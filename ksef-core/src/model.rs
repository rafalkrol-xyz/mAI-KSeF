//! Domain model for a KSeF invoice.
//!
//! This is deliberately scoped to the recurring B2B service invoice shape
//! (one seller, one buyer, service line items at a single VAT rate). It is not
//! a general FA(3) model — it covers the subset actually used, and can grow.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::Date;

/// A party on the invoice (seller or buyer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    /// Polish tax identifier (NIP), digits only, e.g. "1234567890".
    pub nip: String,
    /// Legal name.
    pub name: String,
    pub address: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    /// Street line, e.g. "Agrykola 1".
    pub street: String,
    /// Postal code, e.g. "00-460".
    pub postal_code: String,
    /// City, e.g. "Warszawa".
    pub city: String,
    /// ISO country code, e.g. "PL".
    #[serde(default = "default_country")]
    pub country: String,
}

fn default_country() -> String {
    "PL".to_string()
}

/// VAT rate as a percentage
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VatRate(pub u8);

impl VatRate {
    /// The FA(3) `StawkaPodatku` code. 23% maps to the standard rate.
    pub fn as_fraction(self) -> Decimal {
        Decimal::from(self.0) / Decimal::from(100)
    }
}

/// A single invoice line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    /// Description, e.g. "Konsultacje informatyczne we wrześniu 2026 ...".
    pub description: String,
    /// Net unit price in the invoice currency.
    pub unit_net: Decimal,
    /// Quantity.
    pub quantity: Decimal,
    /// Unit of measure, e.g. "kpl.".
    pub measure: String,
    pub vat_rate: VatRate,
}

impl LineItem {
    /// Net value = unit_net * quantity.
    pub fn net_value(&self) -> Decimal {
        self.unit_net * self.quantity
    }

    /// VAT amount on this line, rounded to 2 decimals (grosze).
    pub fn vat_amount(&self) -> Decimal {
        (self.net_value() * self.vat_rate.as_fraction()).round_dp(2)
    }

    /// Gross value = net + VAT.
    pub fn gross_value(&self) -> Decimal {
        self.net_value() + self.vat_amount()
    }
}

/// Payment details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Payment form. Your invoices use "Przelew" (bank transfer).
    #[serde(default = "default_payment_form")]
    pub form: String,
    /// IBAN / bank account number (spaces allowed in config, stripped on output).
    pub account: String,
    /// Payment due date.
    pub due_date: Date,
}

fn default_payment_form() -> String {
    "Przelew".to_string()
}

/// A complete invoice ready to be rendered to FA(3) and submitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Invoice number, e.g. "1/09/2026".
    pub number: String,
    /// Issue date (data wystawienia).
    pub issue_date: Date,
    /// Date the service was completed (data dokonania/zakończenia usługi).
    pub sale_date: Date,
    /// Optional place of issue (miejsce wystawienia).
    #[serde(default)]
    pub issue_place: Option<String>,
    /// ISO currency code, e.g. "PLN".
    #[serde(default = "default_currency")]
    pub currency: String,
    pub seller: Party,
    pub buyer: Party,
    pub items: Vec<LineItem>,
    pub payment: Payment,
}

fn default_currency() -> String {
    "PLN".to_string()
}

impl Invoice {
    /// Total net across all items.
    pub fn total_net(&self) -> Decimal {
        self.items.iter().map(|i| i.net_value()).sum()
    }

    /// Total VAT across all items.
    pub fn total_vat(&self) -> Decimal {
        self.items.iter().map(|i| i.vat_amount()).sum()
    }

    /// Total gross (kwota należności ogółem).
    pub fn total_gross(&self) -> Decimal {
        self.total_net() + self.total_vat()
    }
}
