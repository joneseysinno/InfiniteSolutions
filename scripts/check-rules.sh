#!/usr/bin/env bash
# Mechanical rule checks, per layer.
#
# `bion` is the only attempt in the corpus that had mechanical enforcement, and it is
# the only one that did not drift in the layer it enforced. A rule that cannot be
# checked is a preference (RULES.md preamble); this file is where the runtime's rules
# stop being preferences.
#
# Usage:  bash scripts/check-rules.sh

set -uo pipefail
cd "$(dirname "$0")/.."

fail=0
check() { # check <rule> <description> <command...>
  local rule="$1" desc="$2"; shift 2
  if "$@" >/dev/null 2>&1; then
    printf '  \033[32mok\033[0m   %-6s %s\n' "$rule" "$desc"
  else
    printf '  \033[31mFAIL\033[0m %-6s %s\n' "$rule" "$desc"
    fail=1
  fi
}

# Comment-stripped grep over a source tree. Succeeds when the pattern is FOUND.
#
# Stripping matters: the documentation that *cites* a rule trips the grep that
# *enforces* it. `infinite-compositor/src/core/addr.rs` names `infinite-runtime` in a
# doc comment explaining why it must not depend on it, and `value.rs` names `f64` while
# explaining why there are none. A check with false positives is a check that gets
# switched off, which is F-7's mechanism pointed at CI instead of at a cache.
#
# COMPOSITOR.md §14 finding 4 recorded that the runtime's checks below still grepped
# raw source. Fixed 2026-08-21 in the change that added the presenter, whose §3.3 `f32`
# check and §11 identity check both fire on the doc comments that explain them — and
# leaving one layer of three on raw greps is how the difference gets forgotten.
srcgrep() { # srcgrep <extended-regex> <dir>
  find "$2" -name '*.rs' -print0 | xargs -0 sed 's://[/!]*.*::' | grep -Eq "$1"
}
nogrep() { ! srcgrep "$@"; }

# R16 · the core's enum count is pinned.
#
# A grep that merely reports enums says nothing. A count that must be updated turns
# "a new enum in a core crate requires a decision record" into something that has to be
# answered before the build goes green, rather than at review time.
enum_count_is() { # enum_count_is <dir> <expected>
  local n
  n=$(find "$1" -name '*.rs' -print0 | xargs -0 sed 's://[/!]*.*::' \
      | grep -cE '^[[:space:]]*(pub )?enum ')
  [ "$n" -eq "$2" ]
}

# L5 · every map in a layer is keyed by an address.
#
# "No map keyed by anything but an address" is the presenter's form of F-2 — a map
# keyed by id standing in for an edge, roughly thirty instances in one codebase in this
# corpus. `hyper-ui`'s presentation-layer version is `HashMap<(ContainerId, SizeClass),
# f32>`, persisted, synced, and with nothing that ever evicts an entry.
maps_keyed_by_addr() { # maps_keyed_by_addr <dir>
  local bad
  bad=$(find "$1" -name '*.rs' -print0 | xargs -0 sed 's://[/!]*.*::' \
        | grep -oE '(BTreeMap|HashMap)<[A-Za-z0-9_]+' \
        | grep -vE '<Addr$' || true)
  [ -z "$bad" ]
}

RT=crates/infinite-runtime
echo "infinite-runtime — rule checks"

# R3 · the pure core depends on nothing.
check R3 "core builds with no features" \
  cargo build -p infinite-runtime --offline
check R3 "[dependencies] is empty" \
  bash -c "! awk '/^\[dependencies\]/{f=1;next} /^\[/{f=0} f && NF && \$0 !~ /^#/' $RT/Cargo.toml | grep -q ."

# R8 · the runtime owns no thread pool.
check R8 "no executor or async-runtime dependency" \
  bash -c "! grep -Eq '^(tokio|async-std|smol|futures|rayon|crossbeam)' $RT/Cargo.toml"
check R8 "no thread, sleep, or block_on in the layer" \
  nogrep 'std::thread|thread::spawn|\bsleep\b|block_on' $RT/src

# R9 · the runtime owns no storage.
check R9 "no persistence dependency" \
  bash -c "! grep -Eq '^(infinite-db|rocksdb|sled|rusqlite|redb|heed)' $RT/Cargo.toml"
check R9 "no file handle in the layer" \
  nogrep 'std::fs|File::(open|create)|OpenOptions' $RT/src

# D23 · the runtime names no crate belonging to another layer.
check D23 "no other layer is named in src" \
  nogrep 'infinite_(db|compositor|presenter|physics|ux)' $RT/src

# R11 · addresses, never records.
#   Coarse form of the field-type lint the spec calls for (§4). A struct field holding
#   Vec<u8>/Vec<(Addr, Vec<u8>)> outside the artifact registry is a retained record.
check R11 "no record-shaped struct field outside the registry" \
  nogrep '^[[:space:]]+[a-z_]+: (Vec<\(Addr, Vec<u8>>|BTreeMap<Addr, Vec<u8>>)' $RT/src

# F-8 · no mod.rs.
check F-8 "no mod.rs anywhere" \
  bash -c "! find $RT -name mod.rs | grep -q ."

# R18 · an empty directory with no spec is a finding.
check R18 "every crate directory has a spec" \
  bash -c 'for d in crates/*/; do n=$(basename "$d"); layer=${n#infinite-}; \
           spec="docs/specs/$(echo "$layer" | tr "[:lower:]" "[:upper:]").md"; \
           test -f "$spec" || { echo "no spec for $n (expected $spec)"; exit 1; }; done'

# R12 · every declared artifact passes the discard test.
check R12 "discard harness passes" \
  cargo test -p infinite-runtime --features binding --offline --test discard
# R14 / D24 · backpressure never reaches the input path.
check R14 "saturation test passes" \
  cargo test -p infinite-runtime --features binding --offline --test saturation

echo
CP=crates/infinite-compositor
echo "infinite-compositor — rule checks"

# R3 · the pure core depends on nothing.
check R3 "core builds with no features" \
  cargo build -p infinite-compositor --offline
check R3 "[dependencies] is empty" \
  bash -c "! awk '/^\[dependencies\]/{f=1;next} /^\[/{f=0} f && NF && \$0 !~ /^#/' $CP/Cargo.toml | grep -q ."

# R31 / L3 · the compositor contains no math.
check R31 "no numerical or domain dependency" \
  bash -c "! grep -Eq '^(nalgebra|ndarray|num|faer|sprs|infinite-physics)' $CP/Cargo.toml"
check R31 "no float anywhere in the layer" \
  nogrep 'f32|f64' $CP/src

# R10 / D26 · the compositor has no now. There is no Clock port.
check R10 "no clock in the layer" \
  nogrep 'std::time|SystemTime|Instant' $CP/src

# L2's analogue · nothing is authored here.
check L2 "no file handle in the layer" \
  nogrep 'std::fs|File::(open|create)|OpenOptions' $CP/src

# D26 · the compositor names no crate belonging to another layer.
check D26 "no other layer is named in src" \
  nogrep 'infinite_(db|runtime|presenter|physics|ux)' $CP/src

# L4 · the compositor is a function, not a place.
check L4 "no static, no interior mutability" \
  nogrep 'static (mut )?[A-Z_]+:|RefCell|OnceCell|OnceLock|Mutex|lazy_static' $CP/src

# D13 · tags are matched, never interpreted.
check D13 "no Display for Tag, no parse in the layer" \
  nogrep 'impl .*Display.* for Tag|\.parse[:(<]' $CP/src

# R16 · the core's enum count is pinned. Currently one: `Direction` (spec §14, f.6).
check R16 "core enum count is still 1" \
  enum_count_is $CP/src/core 1

# F-8 · no mod.rs.
check F-8 "no mod.rs anywhere" \
  bash -c "! find $CP -name mod.rs | grep -q ."

# The unit tests. S7 is the equivalence harness (E9). S3–S5 land with E5 (R20).
check S2 "unit tests pass" \
  cargo test -p infinite-compositor --offline --lib
check S3 "ports and fakes" \
  cargo test -p infinite-compositor --features binding --offline --test ports
check S4 "findings corpus" \
  cargo test -p infinite-compositor --offline --test findings
check S5 "composition closes" \
  cargo test -p infinite-compositor --offline --test closure
check S6 "interpreted execution and provenance" \
  cargo test -p infinite-compositor --features binding --offline --test interpret
check S7 "tier 0 registers by passing the harness" \
  cargo test -p infinite-compositor --features binding --offline --test equivalence

echo
PR=crates/infinite-presenter
echo "infinite-presenter — rule checks"

# R3 · the pure core depends on nothing.
check R3 "core builds with no features" \
  cargo build -p infinite-presenter --offline
check R3 "[dependencies] is empty" \
  bash -c "! awk '/^\[dependencies\]/{f=1;next} /^\[/{f=0} f && NF && \$0 !~ /^#/' $PR/Cargo.toml | grep -q ."

# D29 · no graphics crate. D15 gives this layer wgpu *resource organization*; the
# facade owns the API and `Surface` is the seam. A GPU here would make the embedding
# untestable, which is `hyper-ui`'s situation and the reason its renderer has no tests.
check D29 "no graphics or windowing dependency" \
  bash -c "! grep -Eq '^(wgpu|winit|glyphon|cosmic-text|raw-window-handle|softbuffer|glam)' $PR/Cargo.toml"

# D29 · the presenter names no crate belonging to another layer.
check D29 "no other layer is named in src" \
  nogrep 'infinite_(db|runtime|compositor|physics|ux)' $PR/src

# spec §3.3 · one scalar, and it is f64. Narrowing happens in the facade's `Surface`.
check §3.3 "no f32 anywhere in the layer" \
  nogrep 'f32' $PR/src

# R10 / D29 · the presenter has no now. There is no Clock port.
check R10 "no clock in the layer" \
  nogrep 'std::time|SystemTime|Instant' $PR/src

# L6 · the presenter authors nothing. No write port, no file handle.
check L6 "no file handle in the layer" \
  nogrep 'std::fs|File::(open|create)|OpenOptions' $PR/src

# R5 · derived state never writes back. `place` and `probe` take shared references
# only, so P4 — laying out changes the thing being laid out — is a compile error.
check R5 "place and probe take no &mut" \
  bash -c "! grep -REq 'pub fn (place|probe)\(.*&mut' $PR/src"

# L5 · the presenter mints no identity.
check L5 "no identity-shaped field" \
  nogrep '^[[:space:]]+(pub )?[a-z_]*id: (u8|u16|u32|u64|usize)' $PR/src
check L5 "every map is keyed by an address" \
  maps_keyed_by_addr $PR/src

# R16 · the core's enum count is pinned. Currently **zero** — collapse is zoom, so the
# obvious candidate (`Visibility`) is a number instead (spec §5.2).
check R16 "core enum count is still 0" \
  enum_count_is $PR/src/core 0

# F-8 · no mod.rs.
check F-8 "no mod.rs anywhere" \
  bash -c "! find $PR -name mod.rs | grep -q ."

# The unit tests for S4, S5 and S6 land with those stages (R20).
check S2 "unit tests pass" \
  cargo test -p infinite-presenter --offline --lib
check S4 "agreement test" \
  cargo test -p infinite-presenter --offline --test agreement
check S5 "hysteresis sweep" \
  cargo test -p infinite-presenter --offline --test hysteresis
check E14.1 "hysteresis reaches place through a real View" \
  cargo test -p infinite-presenter --offline --test hysteresis_live
check S6 "probe is self-sufficient" \
  cargo test -p infinite-presenter --offline --test probe

echo
FA=src/facade
ED=src/editor
PO=src/portal

echo "infinite-solutions · facade — rule checks"

# R2 · the dependency direction is one-way. Only the facade names a layer.
check R2  "no layer crate named outside src/facade" \
  bash -c 'for d in src/editor src/portal; do
             ! ( find $d -name "*.rs" -print0 | xargs -0 sed "s://[/!]*.*::" \
                 | grep -Eq "infinite_(db|runtime|compositor|presenter)" ) || exit 1
           done'
check R2  "main.rs is thin" \
  bash -c '[ "$(grep -vcE "^\s*(//|$)" src/main.rs)" -le 60 ]'

# D29 · f32 narrowing lives in the Surface implementation; glyphs/text may use
# f32 because glyphon and cosmic-text are f32 APIs (E14, R-F). No other src file.
check D29 "f32 appears only in facade ports surface/glyphs/text" \
  bash -c '! ( find src -name "*.rs" \
               ! -path "src/facade/ports/surface.rs" \
               ! -path "src/facade/ports/glyphs.rs" \
               ! -path "src/facade/ports/text/*" \
               ! -path "src/facade/ports/text.rs" \
               -print0 \
               | xargs -0 sed "s://[/!]*.*::" | grep -q "f32" )'

# D30 / L5 · the app mints no identity either. Every reference is an address.
check L5  "every map in src is keyed by an address" maps_keyed_by_addr src

# D24 · there is exactly one write path, and input is not on it.
check D24 "portal/input.rs never submits a write" \
  nogrep 'submit|StoreWrite' $PO/input.rs

# R8 / L1 · the portal drives; the runtime does not.
check R8  "no thread pool in the tick path" \
  nogrep 'std::thread|thread::spawn|\bsleep\b|block_on' $PO/drive.rs

# F-8 · no mod.rs.
check F-8 "no mod.rs anywhere" bash -c '! find src -name mod.rs | grep -q .'

echo
echo "infinite-solutions · editor — rule checks"

# R2 · the app depends on the facade, and on nothing below it.
check R2  "editor names no layer crate" \
  nogrep 'infinite_(db|runtime|compositor|presenter)' $ED

# D29 · the app names no graphics crate.
check D29 "editor names no graphics or windowing crate" \
  nogrep 'wgpu|winit|glyphon|cosmic_text|raw_window_handle|softbuffer|glam' $ED

# §2.1 · appearance is authored data. There is no widget block.
# Finding 22: an anchored filename grep let increment_text.rs pass as a native.
# The live registry is the check; a one-off cannot hide behind a helper file.
check §2.1 "no widget-shaped block" \
  bash -c '! ls src/editor/blocks/ | grep -Eqi "^(rectangle|label|panel|widget|button)\.rs$"'
check §2.1 "retired one-offs are not natives" \
  bash -c '! awk "/fn native_signatures/,/^}/" src/facade/ports/blocks.rs \
           | grep -Eq "\"(increment-text|encode-selection|encode-wire|set-origin|offset|displace)\""'

# E18a · declared effect set. Live natives ⊆ {read,amend,commit,gate,probe-at,map,fold}.
# Rule 1: a primitive used by fewer than two src/ sites is a one-off.
# Rule 2: a component that requires a new primitive means the alphabet was wrong.
# Keys are the bare quoted line inside native_signatures (not port names).
check E18a.0 "each declared effect has a two-domain line" \
  bash -c 'for k in probe-at read amend commit gate map fold; do
             grep -Fq -- "$k" docs/specs/EDITOR.md || exit 1
           done'
check E18a.2 "live natives ⊆ declared effects" \
  bash -c 'for k in $(awk "/fn native_signatures/,/^fn sig/" src/facade/ports/blocks.rs \
              | grep -E "^[[:space:]]+\"[a-z-]+\",[[:space:]]*$" \
              | tr -d " \"," ); do
             echo "probe-at read amend commit gate map fold" | grep -qw "$k" || exit 1
           done'
check Rule1 "each live native is used by at least two src sites" \
  bash -c 'for k in probe-at read amend commit gate map fold; do
             n=$(grep -R --include="*.rs" -F "b\"$k\"" src | wc -l)
             [ "$n" -ge 2 ] || { echo "$k used $n times"; exit 1; }
           done'
check Rule2 "kind != native appears in src (composition closes)" \
  bash -c 'grep -R --include="*.rs" -q "kind: \"delegate\"" src'

# D34 · addresses live in exactly one file.
check D34 "no literal well-known address outside addresses.rs" \
  bash -c '! ( find src -name "*.rs" ! -path "src/editor/addresses.rs" -print0 \
               | xargs -0 sed "s://[/!]*.*::" | grep -Eq "\"/(input|style|screen)/" )'

# E4 · the screen is data. An emptied store fails as a finding, not a black frame.
check E4  "genesis discard — empty canvas with a finding; re-seed is bit-identical" \
  cargo test --offline --test genesis

# E5 · the editor's behaviour composition links.
check E5  "the editor's behaviour composition links" \
  cargo test --offline --test link

# E6 · the drag is interpreted; provenance is exact; deleting the composition
# stops the drag while the portal still ticks.
check E6  "drag is performed by the interpreted composition" \
  cargo test --offline --test behaviour

# E7 · a genesis node moves, the change survives restart, nothing was recompiled.
check E7  "the editor edits its own screen and it persists" \
  cargo test --offline --test self_edit

# E8 · a pending wire links (C4); a tag mismatch is said/wanted/remedy and a zoom.
check E8  "pending wire links; mismatch zooms to its site" \
  cargo test --offline --test wiring

# E9 · tier 0 registers by passing the harness on the editor's plan.
check E9  "tier 0 passes the equivalence harness; discard is generic" \
  cargo test --offline --test tier0

# E12 · undo writes the pre-drag value as a new commit, not a rewind; redo
# replays it; discard drops a pending amend without touching the stream; the
# stream itself passes R12's generic harness (D48).
check E12 "undo/redo are new commits; discard is the other verb" \
  cargo test --offline --test undo

# E13.0 · a third primitive draws text measured through the Glyphs port.
check E13.0 "text reaches the screen at two scale factors" \
  cargo test --offline --test text

# E14 · a real font: every digit and mixed case has distinct ink (finding 23).
check E14 "every digit and mixed case has distinct ink" \
  cargo test --offline --test readable_text

# E13.1 · selection is an authored record at SELECT_KEY, not a flag on Placed.
check E13.1 "selection survives restart; Placed has no selected field" \
  cargo test --offline --test selection

# E13.2 · the inspector reads selection through the scene port only.
check E13.2 "inspector fields match scene; inspector names no store type" \
  cargo test --offline --test inspector

# E13.3 · inspector edits go through the behaviour composition, not direct amend.
check E13.3 "origin edit follows canvas same frame; undo restores" \
  cargo test --offline --test inspector_write

# E13.4 · palette drag mints a child address under the parent; survives restart.
check E13.4 "palette drag mints child; survives restart" \
  cargo test --offline --test palette

# E13.5 · wiring by pointer commits a wire; mismatch previews and zooms.
check E13.5 "wire drag commits link; mismatch finding before release" \
  cargo test --offline --test wire_author

# E13.6 · toolbar: undo/redo, zoom readout, run/pause — three spaces, not widgets.
check E13.6 "toolbar undo; zoom readout; run pauses tick" \
  cargo test --offline --test toolbar

# E13.7 · counter app authored by pointer; increments and persists.
check E13.7 "counter increments on bump click; total survives restart" \
  cargo test --offline --test counter

# E17 · open record: fourth shape stores payload under the space.
check E17 "fourth shape draws from payload_key; no field-per-primitive" \
  cargo test --offline --test open_record

# E18b · stored definition; two screens; counter has no Rust installer.
check E18b "two screens share one store definition; no recompile" \
  cargo test --offline --test stored_component

# E19 · focus, type through composition, undo restores.
check E19 "focus + typed payload + undo" \
  cargo test --offline --test focus

# E20 · Innovator screen; role-routed commit; re-seed without builders.
check E20 "Innovator screen reseeds from the store" \
  cargo test --offline --test innovator_screen

echo
if [ "$fail" -eq 0 ]; then echo "all checks passed"; else echo "findings above"; fi
exit "$fail"
