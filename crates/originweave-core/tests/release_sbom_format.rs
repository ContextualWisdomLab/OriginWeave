use originweave_core::release_manifest::ReleaseSbomFormat;

#[test]
fn spdx_30_json_ld_exposes_required_global_context_identity() {
    let format = ReleaseSbomFormat::Spdx30JsonLd;

    assert_eq!(format.spdx_specification_version(), "3.0.1");
    assert_eq!(
        format.json_ld_context_uri(),
        "https://spdx.org/rdf/3.0.1/spdx-context.jsonld"
    );
}
