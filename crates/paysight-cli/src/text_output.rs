//! Pretty terminal output for `ScanReport`s.
//!
//! Uses `comfy-table` for the tables and `console::Style` for color-coded
//! confidence pills, gateway accents, and section headers.

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};
use console::Style;
use paysight_core::{
    AuthGateStatus, Confidence, PaymentCategory, PaymentHit, ScanReport,
};

pub fn render(report: &ScanReport, colorful: bool) {
    let head = if colorful {
        Style::new().bold().bright().cyan()
    } else {
        Style::new().bold()
    };
    let dim = Style::new().dim();

    println!();
    println!(
        "{} {} {}",
        head.apply_to("▎"),
        head.apply_to(&report.target),
        dim.apply_to(format!("→ {}", report.final_url)),
    );
    println!(
        "  {}  {}  {}",
        dim.apply_to(format!("{} ms", report.elapsed_ms)),
        dim.apply_to(format!("{} probes", report.probes.len())),
        gate_pill(report.auth_gate, colorful),
    );

    if let Some(top) = report.primary_gateway() {
        let label = if colorful {
            Style::new().bold().green().apply_to("PRIMARY").to_string()
        } else {
            "PRIMARY".to_string()
        };
        println!(
            "  {} {} ({} · score {})",
            label,
            if colorful {
                Style::new().bold().apply_to(&top.vendor).to_string()
            } else {
                top.vendor.clone()
            },
            confidence_pill(top.confidence, colorful),
            top.score,
        );
    }

    println!();
    println!("  {}", section("Payment stack", colorful));
    if report.payment_hits.is_empty() {
        println!("    {}", dim.apply_to("(no signatures matched)"));
    } else {
        let mut rows: Vec<&PaymentHit> = report.payment_hits.iter().collect();
        rows.sort_by(|a, b| {
            a.category
                .label()
                .cmp(b.category.label())
                .then(b.score.cmp(&a.score))
        });
        let mut table = build_table();
        table.set_header(vec![
            cell_h("Category", colorful),
            cell_h("Vendor", colorful),
            cell_h("Score", colorful),
            cell_h("Confidence", colorful),
            cell_h("Patterns", colorful),
        ]);
        for h in rows {
            table.add_row(vec![
                Cell::new(h.category.label()).fg(category_color(h.category, colorful)),
                Cell::new(&h.vendor).add_attribute(comfy_table::Attribute::Bold),
                Cell::new(h.score).set_alignment(CellAlignment::Right),
                Cell::new(h.confidence.label()).fg(confidence_color(h.confidence, colorful)),
                Cell::new(truncate(&h.matched_patterns.join(", "), 60)),
            ]);
        }
        println!("{table}");
    }

    println!();
    println!("  {}", section("Bot mitigation / WAF / CDN", colorful));
    if report.antibot_hits.is_empty() {
        println!("    {}", dim.apply_to("(no signatures matched)"));
    } else {
        let mut table = build_table();
        table.set_header(vec![
            cell_h("Kind", colorful),
            cell_h("Vendor", colorful),
            cell_h("Score", colorful),
            cell_h("Confidence", colorful),
            cell_h("Patterns", colorful),
        ]);
        for h in &report.antibot_hits {
            table.add_row(vec![
                Cell::new(h.kind.label()).fg(if colorful { Color::Yellow } else { Color::Reset }),
                Cell::new(&h.vendor).add_attribute(comfy_table::Attribute::Bold),
                Cell::new(h.score).set_alignment(CellAlignment::Right),
                Cell::new(h.confidence.label()).fg(confidence_color(h.confidence, colorful)),
                Cell::new(truncate(&h.matched_patterns.join(", "), 60)),
            ]);
        }
        println!("{table}");
    }

    if !report.cookies.is_empty() {
        println!();
        println!("  {}", section("Cookies set during probes", colorful));
        let names: Vec<String> = report.cookies.iter().map(|c| c.name.clone()).collect();
        let cookie_style = if colorful {
            Style::new().bg(console::Color::Color256(238)).white()
        } else {
            Style::new()
        };
        let mut line = String::from("    ");
        for (i, n) in names.iter().enumerate() {
            if i > 0 {
                line.push(' ');
            }
            line.push_str(&format!("{}", cookie_style.apply_to(format!(" {n} "))));
        }
        println!("{line}");
    }

    println!();
}

fn build_table() -> Table {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth);
    t
}

fn cell_h(s: &str, colorful: bool) -> Cell {
    let mut c = Cell::new(s).add_attribute(comfy_table::Attribute::Bold);
    if colorful {
        c = c.fg(Color::Cyan);
    }
    c
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}

fn section(title: &str, colorful: bool) -> String {
    if colorful {
        Style::new()
            .bold()
            .magenta()
            .apply_to(format!("◆ {title}"))
            .to_string()
    } else {
        format!("[{title}]")
    }
}

fn confidence_color(c: Confidence, colorful: bool) -> Color {
    if !colorful {
        return Color::Reset;
    }
    match c {
        Confidence::None | Confidence::Weak => Color::DarkGrey,
        Confidence::Moderate => Color::Yellow,
        Confidence::Strong => Color::Green,
        Confidence::VeryStrong => Color::Magenta,
    }
}

fn category_color(c: PaymentCategory, colorful: bool) -> Color {
    if !colorful {
        return Color::Reset;
    }
    match c {
        PaymentCategory::PrimaryGateway => Color::Green,
        PaymentCategory::Wallet => Color::Cyan,
        PaymentCategory::BuyNowPayLater => Color::Magenta,
        PaymentCategory::BankRedirect => Color::Blue,
        PaymentCategory::ThreeDSecure => Color::Yellow,
        PaymentCategory::SiteBuilder => Color::DarkGrey,
    }
}

fn confidence_pill(c: Confidence, colorful: bool) -> String {
    let label = format!(" {} ", c.label());
    if !colorful {
        return format!("[{}]", label.trim());
    }
    let style = match c {
        Confidence::None | Confidence::Weak => Style::new().bg(console::Color::Color256(238)).dim(),
        Confidence::Moderate => Style::new().bg(console::Color::Color256(94)).white(),
        Confidence::Strong => Style::new().bg(console::Color::Color256(28)).white().bold(),
        Confidence::VeryStrong => Style::new().bg(console::Color::Color256(127)).white().bold(),
    };
    style.apply_to(label).to_string()
}

fn gate_pill(g: AuthGateStatus, colorful: bool) -> String {
    let (label, color256) = match g {
        AuthGateStatus::Open => ("checkout open", 28u8),
        AuthGateStatus::Gated => ("auth-gated", 196u8),
        AuthGateStatus::Unknown => ("gate unknown", 238u8),
    };
    if !colorful {
        return format!("[{label}]");
    }
    Style::new()
        .bg(console::Color::Color256(color256))
        .white()
        .apply_to(format!(" {label} "))
        .to_string()
}
