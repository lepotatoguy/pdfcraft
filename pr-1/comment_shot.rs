//! Throwaway: before/after screenshot of commenting on selected text. Not committed.
use std::time::{Duration, Instant};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::PdfCraftApp;

const S1: &str = "There is a short pause as the reviewer reads this line.";
const S2: &str = "The second sentence stays unmarked by the comment.";

fn pdf() -> Vec<u8> {
    let content = format!("BT /F1 16 Tf 30 180 Td ({S1}) Tj 0 -30 Td ({S2}) Tj ET");
    let objs = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 520 260] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offs = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offs.push(out.len());
        out.extend(format!("{} 0 obj\n{o}\nendobj\n", i + 1).bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).bytes());
    for o in offs {
        out.extend(format!("{o:010} 00000 n \n").bytes());
    }
    out.extend(format!("trailer << /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).bytes());
    out
}

fn settle(h: &mut Harness<'static, PdfCraftApp>) {
    let start = Instant::now();
    h.run_steps(4);
    while start.elapsed() < Duration::from_secs(20) {
        h.run_steps(2);
        if !h.state().render_pending() {
            h.run_steps(3);
            if !h.state().render_pending() {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(30));
    }
}

fn main() -> Result<(), String> {
    let out = std::env::args().nth(1).ok_or("usage: comment_shot out.png")?;
    let mut h = Harness::builder().with_size(egui::vec2(1440.0, 760.0)).with_pixels_per_point(2.0).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("comment.pdf", None, pdf()).expect("opens");
        app.set_option("left", "closed").unwrap();
        app.set_option("author", "Tester").unwrap();
        app
    });
    settle(&mut h);
    h.state_mut().views[0].select_text(0, 0, S1.chars().count() - 1);
    h.run_steps(3);
    h.get_by_label("Add comment").click();
    h.run_steps(3);
    if h.state().views[0].comments.composer.is_some() {
        // Old behaviour: a sticky-note composer.
        h.query_all_by(|n| n.placeholder() == Some("Add a comment")).next().ok_or("no composer field")?.type_text("hi");
        h.run_steps(2);
        h.get_by_label("Post").click();
        h.run_steps(3);
        h.state_mut().set_option("panel", "comments").unwrap();
    } else {
        // New behaviour: the highlight's card is open for typing.
        h.get_by_role(egui::accesskit::Role::MultilineTextInput).click();
        h.run_steps(2);
        h.get_by_role(egui::accesskit::Role::MultilineTextInput).type_text("hi");
        h.run_steps(2);
        h.get_by_label("Save").click();
        h.run_steps(3);
        h.state_mut().set_option("quick", "select").unwrap();
    }
    settle(&mut h);
    let image = h.render()?;
    image.save(&out).map_err(|e| e.to_string())?;
    let s = h.state();
    let a = &s.session.get(s.views[0].id).ok_or("no doc")?.info.annotations;
    eprintln!("wrote {out}; annotations: {:?}", a.iter().map(|a| (a.subtype.clone(), a.contents.clone())).collect::<Vec<_>>());
    Ok(())
}
