// Removes the placeholder texts added by "Fill empty texts" (the ones that start with "Lorem ipsum"), leaving real content alone.

fn run(design) {
    let cleared = 0;
    for i in 0..design.elements.len() {
        if design.elements[i].text.starts_with("Lorem ipsum") || design.elements[i].text.starts_with("Lorem") {
            design.elements[i].text = "";
            cleared += 1;
        }
    }
    log(cleared.to_string() + " placeholder texts removed");
    design
}
