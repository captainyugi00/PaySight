//! Animated startup banner. Renders the PaySight logotype with a horizontal
//! gradient using truecolor ANSI escapes.

use colorgrad::Gradient;
use console::Term;

const LOGO: &[&str] = &[
    "██████╗  █████╗ ██╗   ██╗███████╗██╗ ██████╗ ██╗  ██╗████████╗",
    "██╔══██╗██╔══██╗╚██╗ ██╔╝██╔════╝██║██╔════╝ ██║  ██║╚══██╔══╝",
    "██████╔╝███████║ ╚████╔╝ ███████╗██║██║  ███╗███████║   ██║   ",
    "██╔═══╝ ██╔══██║  ╚██╔╝  ╚════██║██║██║   ██║██╔══██║   ██║   ",
    "██║     ██║  ██║   ██║   ███████║██║╚██████╔╝██║  ██║   ██║   ",
    "╚═╝     ╚═╝  ╚═╝   ╚═╝   ╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ",
];

const TAGLINE: &str = "Payment + protection stack reconnaissance";

/// Print the banner. If `colorful` is false, use plain monochrome.
pub fn print(colorful: bool) {
    if !Term::stdout().is_term() {
        return;
    }
    if colorful {
        print_colorful();
    } else {
        for line in LOGO {
            println!("{line}");
        }
        println!("    {TAGLINE}");
        println!();
    }
}

fn print_colorful() {
    // Build gradient from electric magenta -> cyan -> mint, evoking a "scope"
    // / sci-fi recon vibe. Anchored on three named colors so the visual is
    // stable across runs.
    let grad = colorgrad::GradientBuilder::new()
        .colors(&[
            colorgrad::Color::from_rgba8(0xC8, 0x40, 0xFF, 0xFF),
            colorgrad::Color::from_rgba8(0x36, 0xC2, 0xFF, 0xFF),
            colorgrad::Color::from_rgba8(0x3D, 0xF5, 0xB1, 0xFF),
        ])
        .build::<colorgrad::CatmullRomGradient>()
        .expect("static gradient builds");

    let width = LOGO[0].chars().count() as f32;
    for line in LOGO {
        let mut out = String::new();
        for (i, ch) in line.chars().enumerate() {
            let t = i as f32 / (width - 1.0).max(1.0);
            let c = grad.at(t);
            let [r, g, b, _] = c.to_rgba8();
            out.push_str(&format!("\x1b[38;2;{r};{g};{b}m{ch}"));
        }
        out.push_str("\x1b[0m");
        println!("{out}");
    }

    // Tagline in dim white with a leading bullet that picks up the gradient end.
    let end = grad.at(1.0).to_rgba8();
    let [r, g, b, _] = end;
    println!(
        "\x1b[38;2;{r};{g};{b}m  ◆\x1b[0m \x1b[2m{TAGLINE}\x1b[0m"
    );
    println!();
}
