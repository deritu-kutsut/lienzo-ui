// Makes duplicated element names unique by adding " 2", " 3"... to the repeated ones.

fn run(design) {
    let seen = #{};
    for i in 0..design.elements.len() {
        seen[design.elements[i].name] = 0;
    }
    let counts = #{};
    let changed = 0;
    for i in 0..design.elements.len() {
        let name = design.elements[i].name;
        if counts.contains(name) {
            let k = counts[name] + 1;
            let candidate = name + " " + k.to_string();
            while seen.contains(candidate) {
                k += 1;
                candidate = name + " " + k.to_string();
            }
            counts[name] = k;
            seen[candidate] = 0;
            design.elements[i].name = candidate;
            changed += 1;
        } else {
            counts[name] = 1;
        }
    }
    log(changed.to_string() + " duplicated names fixed");
    design
}
