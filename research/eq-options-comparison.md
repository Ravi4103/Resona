# EQ Implementation Options Comparison

| Criteria | **Hand-rolled Rust biquad** | **rbj-eq crate** | **miniaudio ma_filter (FFI)** | **Wait for PeakNode fix** |
|---|---|---|---|---|
| **Reliability** | 10 lines, zero deps, we own the code | Well-tested, but depends on third-party crate | Calls C library — unsafe, pointer lifetime risks | Unknown bug in maudio 0.1.1, no ETA |
| **Stability** | Will never break — we control it | Trustworthy (v0.7.1, no_std), but could have upstream breakage | Depends on maudio-sys / miniaudio ABI stability | Broke once, may break again |
| **Performance** | Identical to C — generates same machine code | Tiny overhead from trait abstractions | Same as Rust biquad, extra FFI call overhead | Same as any miniaudio node |
| **Easy to understand** | 20 lines total (filter + coefficients). Anyone can read it | Need to learn rbj-eq API, trait concepts | Need unsafe, raw pointers, maudio-sys FFI | None — it just works (when fixed) |
| **Flexibility** | Full control — add bands, change Q, morph coefficients in real-time | Constrained to crate's filter types | Tied to miniaudio filter types | Tied to PeakNode API |
| **Dependencies** | Zero (stdlib only) | rbj-eq 0.7.1 | maudio-sys (already dep) | maudio (already dep) |

## Verdict

**Hand-rolled Rust biquad wins on every axis:**
- 10-line filter function + 20-line coefficient calculation
- Same performance as C (compiler generates identical SIMD)
- Zero external dependencies
- We own every bit of code — never breaks from updates
- Infinite flexibility (shelving, peaking, custom curves, morphing)

The "complexity" is just the Audio EQ Cookbook formulas — they're copy-paste from the public domain RBJ paper, which every audio app uses.
