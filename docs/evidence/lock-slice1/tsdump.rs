// throwaway: print code_fields for every .ts/.tsx path given on stdin
use std::io::BufRead;
fn main() {
    for line in std::io::stdin().lock().lines() {
        let p = line.unwrap();
        let Ok(t) = std::fs::read_to_string(&p) else { continue };
        if let Some(f) = invar_falsify::surface::code_fields(&p, &t) {
            for ((s, n), ty) in f {
                println!("{p}\t{s}\t{n}\t{}", ty.unwrap_or_default());
            }
        }
    }
}
