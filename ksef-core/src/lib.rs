//! Core domain logic for mAI-KSeF: invoice model, config loading, FA(3) XML
//! generation, and the KSeF client.
//!
//! Both the CLI (`ksef-cli`) and future front-ends (MCP server, web) are thin
//! layers over this crate.

pub mod client;
pub mod config;
pub mod fa3;
pub mod model;

pub use client::{Client, Environment, Token};
pub use config::load_invoice;
pub use model::Invoice;

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use time::macros::date;

    fn sample_invoice() -> Invoice {
        use model::*;
        Invoice {
            number: "1/09/2026".to_string(),
            issue_date: date!(2026 - 09 - 27),
            sale_date: date!(2026 - 09 - 27),
            issue_place: Some("Warszawa".to_string()),
            currency: "PLN".to_string(),
            seller: Party {
                nip: "1234567890".to_string(),
                name: "John Doe LLC".to_string(),
                address: Address {
                    street: "Agrykola 1".to_string(),
                    postal_code: "00-460".to_string(),
                    city: "Warszawa".to_string(),
                    country: "PL".to_string(),
                },
            },
            buyer: Party {
                nip: "0987654321".to_string(),
                name: "Jane Doe Inc.".to_string(),
                address: Address {
                    street: "Rondo Ignacego Daszyńskiego 1".to_string(),
                    postal_code: "00-843".to_string(),
                    city: "Warszawa".to_string(),
                    country: "PL".to_string(),
                },
            },
            items: vec![LineItem {
                description: "Konsultacje informatyczne we wrześniu 2026 zgodnie z umową z dnia 31 grudnia 2025"
                    .to_string(),
                unit_net: dec!(11111.11),
                quantity: dec!(1),
                measure: "kpl.".to_string(),
                vat_rate: VatRate(23),
            }],
            payment: Payment {
                form: "Przelew".to_string(),
                account: "20 1610 1902 7790 0335 6801 9355".to_string(),
                due_date: date!(2026 - 10 - 14),
            },
        }
    }

    #[test]
    fn totals_match_known_invoice() {
        let inv = sample_invoice();
        // From the real September 2026 invoice.
        assert_eq!(inv.total_net(), dec!(11111.11));
        assert_eq!(inv.total_vat(), dec!(2555.56));
        assert_eq!(inv.total_gross(), dec!(13666.67));
    }

    #[test]
    fn fa3_renders_and_contains_key_fields() {
        let inv = sample_invoice();
        let xml = fa3::render(&inv);
        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<NIP>1234567890</NIP>"));
        assert!(xml.contains("<NIP>0987654321</NIP>"));
        assert!(xml.contains("<P_2>1/09/2026</P_2>"));
        assert!(xml.contains("<P_15>13666.67</P_15>"));
        // Bank account spaces are stripped.
        assert!(xml.contains("<NrRB>20161019027790033568019355</NrRB>"));
    }

    #[test]
    fn environment_defaults_to_test() {
        assert_eq!(Environment::default(), Environment::Test);
        assert!(!Environment::default().is_production());
    }
}
