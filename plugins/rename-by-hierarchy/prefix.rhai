// Renames every element as Parent_Child (the whole chain of ancestors), so names stay unique and readable in Unity.
// Running it twice does not repeat the prefixes.

fn strip_prefix(name, parent_name) {
    let p = parent_name + "_";
    if name.len() > p.len() && name.starts_with(p) { name.sub_string(p.len()) } else { name }
}

fn run(design) {
    let n = design.elements.len();
    let idx = #{};
    let orig = [];
    let par = [];
    for i in 0..n {
        idx[design.elements[i].id.to_string()] = i;
        orig.push(design.elements[i].name);
    }
    for i in 0..n {
        let p = design.elements[i].parent;
        if p != () && idx.contains(p.to_string()) { par.push(idx[p.to_string()]); } else { par.push(-1); }
    }
    let changed = 0;
    for i in 0..n {
        // chain from the element up to its root (guarded against loops)
        let chain = [i];
        let cur = par[i];
        while cur >= 0 && chain.len() < 40 {
            chain.push(cur);
            cur = par[cur];
        }
        chain.reverse();
        let name = orig[chain[0]];
        for k in 1..chain.len() {
            name += "_" + strip_prefix(orig[chain[k]], orig[chain[k - 1]]);
        }
        if name.len() > 120 { name = name.sub_string(0, 120); }
        if name != orig[i] {
            design.elements[i].name = name;
            changed += 1;
        }
    }
    log(changed.to_string() + " elements renamed");
    design
}
