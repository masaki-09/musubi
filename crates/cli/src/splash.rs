//! Splash screen shown when `musubi` is invoked with no subcommand.
//!
//! - If stdout is a TTY: full ANSI-coloured banner (Claude Code / Gemini CLI style).
//! - If stdout is piped: plain-text fallback (no escape codes).

use std::io::IsTerminal as _;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ── ANSI helpers ──────────────────────────────────────────────────────────────

macro_rules! c {
    ($code:literal, $s:expr) => {
        format!("\x1b[{}m{}\x1b[0m", $code, $s)
    };
}

fn bold(s: &str) -> String {
    c!("1", s)
}
fn dim(s: &str) -> String {
    c!("2", s)
}
fn red(s: &str) -> String {
    c!("31", s)
}
fn bright_red(s: &str) -> String {
    c!("91", s)
}
fn white(s: &str) -> String {
    c!("97", s)
}
fn cyan(s: &str) -> String {
    c!("36", s)
}
fn yellow(s: &str) -> String {
    c!("33", s)
}

// ── Layout constant ───────────────────────────────────────────────────────────

const W: usize = 58; // inner width (between borders)

fn rule() -> String {
    dim(&format!("╭{}╮", "─".repeat(W)))
}
fn rule_bottom() -> String {
    dim(&format!("╰{}╯", "─".repeat(W)))
}
fn rule_mid() -> String {
    dim(&format!("├{}┤", "─".repeat(W)))
}

/// Render one row: `│ <content padded to W-2> │`
/// `content_len` is the *visible* length (without ANSI codes).
fn row(content: &str, content_len: usize) -> String {
    let pad = W.saturating_sub(2 + content_len); // 2 = leading space + trailing space
    format!("{} {}{} {}", dim("│"), content, " ".repeat(pad), dim("│"))
}

fn blank_row() -> String {
    row("", 0)
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn print() {
    if std::io::stdout().is_terminal() {
        print_ansi();
    } else {
        print_plain();
    }
}

// ── ANSI banner ───────────────────────────────────────────────────────────────

fn print_ansi() {
    let lines = build_lines();
    for l in &lines {
        println!("{l}");
    }
}

fn build_lines() -> Vec<String> {
    let mut v = Vec::new();

    // ── Top border ────────────────────────────────────────────────────────────
    v.push(rule());

    // ── Title bar: "musubi 結び  v0.x.x" ──────────────────────────────────
    v.push(blank_row());
    {
        // "  musubi" bold white + "  結び" bright red + "  v0.x.x" dim
        let title = format!(
            "{}  {}  {}",
            bold(&white("musubi")),
            bright_red("結び"),
            dim(&format!("v{VERSION}"))
        );
        // visible: "  musubi  結び  v0.x.x"
        // "musubi" = 6, "  " = 2, "結び" = 2 chars (4 cols wide), "  " = 2, "vX.Y.Z" depends
        let vis = 6 + 2 + 4 + 2 + 1 + VERSION.len(); // 1 for 'v'
        v.push(row(&title, vis));
    }
    {
        let tagline = dim("関係性暗号 — Relational classical cipher");
        // "関係性暗号" = 5 chars × 2 = 10 cols, " — Relational classical cipher" = 30
        let vis = 10 + 3 + 29; // "—" is 3 bytes but 1 col, space around = 3 cols total
        v.push(row(&tagline, vis));
    }
    v.push(blank_row());

    // ── Divider ───────────────────────────────────────────────────────────────
    v.push(rule_mid());

    // ── Quick start ───────────────────────────────────────────────────────────
    v.push(blank_row());
    {
        let label = format!("{}  Quick start", bold(&cyan("◆")));
        v.push(row(&label, 2 + 11)); // "◆" 1 col + "  Quick start" 13 = 14 ... tweak
    }
    v.push(blank_row());

    let steps: &[(&str, &str, &str)] = &[
        ("①", "鍵を生成", "musubi keygen -o my.key"),
        (
            "②",
            "暗号化  ",
            "musubi encrypt -k my.key -i plain.txt -o cipher.json",
        ),
        ("③", "復号    ", "musubi decrypt -k my.key -i cipher.json"),
    ];

    for (num, label, cmd) in steps {
        // step number + japanese label
        let step_line = format!("  {}  {}", bright_red(num), dim(label));
        let step_vis = 2 + 1 + 2 + label.chars().count();
        v.push(row(&step_line, step_vis));

        // command (indented)
        let cmd_line = format!("     {}", red(cmd));
        let cmd_vis = 5 + cmd.len();
        v.push(row(&cmd_line, cmd_vis));

        v.push(blank_row());
    }

    // ── Divider ───────────────────────────────────────────────────────────────
    v.push(rule_mid());

    // ── Links ─────────────────────────────────────────────────────────────────
    v.push(blank_row());
    {
        let link_line = format!("  {}  {}", dim("Docs"), cyan("https://musubi.masak1.com"));
        v.push(row(&link_line, 2 + 4 + 2 + 25));
    }
    {
        let link_line = format!(
            "  {}  {}",
            dim("Src "),
            cyan("https://github.com/masaki-09/musubi"),
        );
        v.push(row(&link_line, 2 + 4 + 2 + 35));
    }
    v.push(blank_row());

    // ── Divider ───────────────────────────────────────────────────────────────
    v.push(rule_mid());

    // ── Disclaimer ────────────────────────────────────────────────────────────
    v.push(blank_row());
    {
        let warn_line = format!(
            "  {}  {}",
            yellow("⚠"),
            dim("玩具暗号です。機密情報の保護には使用しないでください。"),
        );
        // visible: "  ⚠  玩具暗号です。機密情報の保護には使用しないでください。"
        // "⚠" = 1, "  " = 2, space before = 2, text = 26 chars × 2 + 4 punct ≈ 52
        let vis = 2 + 1 + 2 + 52;
        v.push(row(&warn_line, vis));
    }
    v.push(blank_row());

    // ── Bottom border ─────────────────────────────────────────────────────────
    v.push(rule_bottom());

    // ── Tip after box ─────────────────────────────────────────────────────────
    v.push(String::new());
    v.push(format!(
        "  {} {}",
        dim("サブコマンドのヘルプ:"),
        white("musubi <SUBCOMMAND> --help"),
    ));
    v.push(String::new());

    v
}

// ── Plain-text fallback ───────────────────────────────────────────────────────

fn print_plain() {
    println!("musubi (結び) v{VERSION}");
    println!("関係性暗号 — Relational classical cipher");
    println!();
    println!("Quick start:");
    println!("  1. musubi keygen -o my.key");
    println!("  2. musubi encrypt -k my.key -i plain.txt -o cipher.json");
    println!("  3. musubi decrypt -k my.key -i cipher.json");
    println!();
    println!("Docs:   https://musubi.masak1.com");
    println!("Source: https://github.com/masaki-09/musubi");
    println!();
    println!("WARNING: 玩具暗号 (toy cipher). Do NOT use for sensitive data.");
    println!();
    println!("Usage: musubi <SUBCOMMAND> --help");
}
