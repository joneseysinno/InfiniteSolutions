//! The portal binary.
//!
//! Opens the store, builds the facade, runs the portal. Under 60 non-comment
//! non-blank lines (D32, `scripts/check-rules.sh`).

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| {
        // Layout tag (O32 / E15): bump when SpaceConfig for "editor" changes — there
        // is no migration (D34). A stale `%TEMP%/infinite-solutions` from a prior
        // packing panics as ConfigConflict on register_or_get_space.
        std::env::temp_dir()
            .join("infinite-solutions-o32")
            .to_string_lossy()
            .into_owned()
    });
    let _ = std::fs::create_dir_all(&root);
    let store = infinite_solutions::facade::open(&root).unwrap_or_else(|e| {
        panic!(
            "open store at {root}: {e:?}\n\
             If this is ConfigConflict on \"editor\", the on-disk space layout does not \
             match this build (D34 — no migration). Delete that directory and retry."
        );
    });
    infinite_solutions::editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    infinite_solutions::editor::bind(&store);
    let device = infinite_solutions::portal::Device::open();
    infinite_solutions::portal::Window::open(store, device);
}
