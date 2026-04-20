fn main() {
    let json = include_str!("../../docs/examples.json");
    let diffs: Vec<difft_web::StructuredDiff> = serde_json::from_str(json)
        .expect("embedded examples.json should be valid structured diff JSON");
    print!("{}", difft_web::render_demo_html(&diffs));
}
