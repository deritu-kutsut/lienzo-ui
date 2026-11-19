// Deletes every element named "Guide Safe ..." or "Guide Title ..." (the guides added by this plugin).

fn run(design) {
    let kept = [];
    let removed = 0;
    for e in design.elements {
        if e.name.starts_with("Guide Safe ") || e.name.starts_with("Guide Title ") {
            removed += 1;
        } else {
            kept.push(e);
        }
    }
    design.elements = kept;
    log(removed.to_string() + " guides removed");
    design
}
