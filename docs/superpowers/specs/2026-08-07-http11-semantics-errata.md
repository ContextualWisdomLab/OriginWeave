# Bounded HTTP/1.1 Semantics — Normative Errata

**Status:** Active normative correction to `2026-08-07-http11-semantics-design.md`  
**Applies to:** PR #37 / `originweave-http` framing contract

## Content-Length framing width

The original approved design text shows `BodyFraming::ContentLength(u64)` and says admitted `Content-Length` values must fit `u64`. That wording is superseded for this implementation lineage.

The production API intentionally exposes:

```rust
pub enum BodyFraming {
    NoContent,
    ContentLength(usize),
    Chunked,
    CloseDelimited,
}
```

For responses that can carry content, every canonical decimal `Content-Length` must fit `usize` and the encoded-content budget before the value becomes framing state. This matches the memory/resource authority used by the bounded body collector and avoids a later lossy `u64`-to-`usize` conversion.

For `HEAD`, informational responses, `204`, and `304`, body semantics suppress content. Their `Content-Length` metadata is therefore syntax-validated and duplicate-consistency-validated without converting the decimal value to an allocation-sized integer; no body allocation or read is authorized from that metadata.

This is a product-boundary choice, not a claim that HTTP itself limits `Content-Length` to `usize`. RFC 9110/9112 syntax remains the protocol baseline; OriginWeave applies the narrower platform/resource admission rule only where content would be consumed.

Any future change to a wider public framing integer must add an explicit checked conversion at the allocation/read boundary and retain the same configured encoded-content ceiling.
