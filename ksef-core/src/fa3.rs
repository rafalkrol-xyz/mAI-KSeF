//! FA(3) XML generation.
//!
//! IMPORTANT — correctness caveat:
//! The element names and structure below follow the published FA(3) schema
//! (mandatory since Feb 2026) for the subset of fields these invoices use.
//! Before issuing a real invoice, the output MUST be validated against the
//! official FA(3) XSD (XML Schema Definition) on the KSeF TEST environment.
//! Treat production as gated on a successful TEST round-trip (submit + UPO).
//! See README.
//!
//! This renderer is intentionally hand-written (no XML lib) to keep the output
//! auditable and dependency-light for this narrow, fixed invoice shape.

use rust_decimal::Decimal;
use time::macros::format_description;
use time::Date;

use crate::model::{Address, Invoice, Party};

/// Render an [`Invoice`] to FA(3) XML.
pub fn render(invoice: &Invoice) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');
    out.push_str(r#"<Faktura xmlns="http://crd.gov.pl/wzor/2025/06/25/13775/">"#);
    out.push('\n');

    render_header(&mut out, invoice);
    render_party(&mut out, "Podmiot1", &invoice.seller);
    render_party(&mut out, "Podmiot2", &invoice.buyer);
    render_fa(&mut out, invoice);

    out.push_str("</Faktura>\n");
    out
}

fn render_header(out: &mut String, invoice: &Invoice) {
    out.push_str("  <Naglowek>\n");
    out.push_str(
        "    <KodFormularza kodSystemowy=\"FA (3)\" wersjaSchemy=\"1-0E\">FA</KodFormularza>\n",
    );
    out.push_str("    <WariantFormularza>3</WariantFormularza>\n");
    out.push_str(&format!(
        "    <DataWytworzeniaFa>{}</DataWytworzeniaFa>\n",
        datetime_at_issue(invoice.issue_date)
    ));
    if let Some(place) = &invoice.issue_place {
        out.push_str(&format!(
            "    <MiejsceWystawienia>{}</MiejsceWystawienia>\n",
            escape(place)
        ));
    }
    out.push_str(&format!("    <SystemInfo>{}</SystemInfo>\n", "mAI-KSeF"));
    out.push_str("  </Naglowek>\n");
}

fn render_party(out: &mut String, tag: &str, party: &Party) {
    out.push_str(&format!("  <{tag}>\n"));
    out.push_str("    <DaneIdentyfikacyjne>\n");
    out.push_str(&format!("      <NIP>{}</NIP>\n", escape(&party.nip)));
    out.push_str(&format!("      <Nazwa>{}</Nazwa>\n", escape(&party.name)));
    out.push_str("    </DaneIdentyfikacyjne>\n");
    render_address(out, &party.address);
    out.push_str(&format!("  </{tag}>\n"));
}

fn render_address(out: &mut String, addr: &Address) {
    out.push_str("    <Adres>\n");
    out.push_str(&format!(
        "      <KodKraju>{}</KodKraju>\n",
        escape(&addr.country)
    ));
    out.push_str(&format!(
        "      <AdresL1>{}</AdresL1>\n",
        escape(&addr.street)
    ));
    out.push_str(&format!(
        "      <AdresL2>{} {}</AdresL2>\n",
        escape(&addr.postal_code),
        escape(&addr.city)
    ));
    out.push_str("    </Adres>\n");
}

fn render_fa(out: &mut String, invoice: &Invoice) {
    out.push_str("  <Fa>\n");
    out.push_str(&format!(
        "    <KodWaluty>{}</KodWaluty>\n",
        escape(&invoice.currency)
    ));
    out.push_str(&format!(
        "    <P_1>{}</P_1>\n",
        date_iso(invoice.issue_date)
    ));
    out.push_str(&format!("    <P_2>{}</P_2>\n", escape(&invoice.number)));
    out.push_str(&format!("    <P_6>{}</P_6>\n", date_iso(invoice.sale_date)));

    // Tax summary for the single 23% rate.
    out.push_str(&format!(
        "    <P_13_1>{}</P_13_1>\n",
        money(invoice.total_net())
    ));
    out.push_str(&format!(
        "    <P_14_1>{}</P_14_1>\n",
        money(invoice.total_vat())
    ));
    out.push_str(&format!(
        "    <P_15>{}</P_15>\n",
        money(invoice.total_gross())
    ));

    out.push_str("    <RodzajFaktury>VAT</RodzajFaktury>\n");

    // Line items.
    for (idx, item) in invoice.items.iter().enumerate() {
        out.push_str("    <FaWiersz>\n");
        out.push_str(&format!("      <NrWierszaFa>{}</NrWierszaFa>\n", idx + 1));
        out.push_str(&format!("      <P_7>{}</P_7>\n", escape(&item.description)));
        out.push_str(&format!("      <P_8A>{}</P_8A>\n", escape(&item.measure)));
        out.push_str(&format!("      <P_8B>{}</P_8B>\n", qty(item.quantity)));
        out.push_str(&format!("      <P_9A>{}</P_9A>\n", money(item.unit_net)));
        out.push_str(&format!("      <P_11>{}</P_11>\n", money(item.net_value())));
        out.push_str(&format!("      <P_12>{}</P_12>\n", item.vat_rate.0));
        out.push_str("    </FaWiersz>\n");
    }

    render_payment(out, invoice);

    out.push_str("  </Fa>\n");
}

fn render_payment(out: &mut String, invoice: &Invoice) {
    out.push_str("    <Platnosc>\n");
    // FormaPlatnosci 6 = przelew (bank transfer) in the FA(3) dictionary.
    out.push_str("      <FormaPlatnosci>6</FormaPlatnosci>\n");
    out.push_str(&format!(
        "      <TerminPlatnosci>\n        <Termin>{}</Termin>\n      </TerminPlatnosci>\n",
        date_iso(invoice.payment.due_date)
    ));
    out.push_str("      <RachunekBankowy>\n");
    out.push_str(&format!(
        "        <NrRB>{}</NrRB>\n",
        account_digits(&invoice.payment.account)
    ));
    out.push_str("      </RachunekBankowy>\n");
    out.push_str("    </Platnosc>\n");
}

// --- formatting helpers ---

/// Money with exactly 2 decimal places and a dot separator (FA(3) uses `.`).
fn money(d: Decimal) -> String {
    format!("{:.2}", d.round_dp(2))
}

/// Quantity, trimmed of trailing zeros but keeping at least the integer form.
fn qty(d: Decimal) -> String {
    d.normalize().to_string()
}

fn date_iso(date: Date) -> String {
    let fmt = format_description!("[year]-[month]-[day]");
    date.format(&fmt).expect("date formatting cannot fail")
}

/// FA(3) `DataWytworzeniaFa` is a datetime; we use midnight local on issue date.
fn datetime_at_issue(date: Date) -> String {
    let fmt = format_description!("[year]-[month]-[day]T00:00:00Z");
    date.format(&fmt).expect("datetime formatting cannot fail")
}

/// Strip spaces from a bank account entered as "20 1610 ...".
fn account_digits(account: &str) -> String {
    account.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Minimal XML escaping for text content.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}
