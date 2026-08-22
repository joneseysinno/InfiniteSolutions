//! The portal binary.
//!
//! Opens the store, builds the facade, runs the portal. Under 60 non-comment
//! non-blank lines (D32, `scripts/check-rules.sh`).

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| {
        std::env::temp_dir()
            .join("infinite-solutions")
            .to_string_lossy()
            .into_owned()
    });
    let _ = std::fs::create_dir_all(&root);
    let store = infinite_solutions::facade::open(&root).expect("open store");
    infinite_solutions::editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    infinite_solutions::editor::bind(&store);
    let device = infinite_solutions::portal::Device::open();
    infinite_solutions::portal::Window::open(store, device);
}
