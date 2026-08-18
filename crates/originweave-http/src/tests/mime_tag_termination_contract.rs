use crate::ContentRiskClass;
use crate::mime::classify_observed_mime;

#[test]
fn every_html_tag_signature_requires_tag_termination() {
    for content in [
        b"<html".as_slice(),
        b"\xef\xbb\xbf \t<htmlx>not an html tag</htmlx>",
        b"<htmlx>not an html tag</htmlx>",
        b"<headless>not a head tag</headless>",
        b"<scripture>not a script tag</scripture>",
        b"<iframed>not an iframe tag</iframed>",
        b"<h10>not an h1 tag</h10>",
        b"<diverse>not a div tag</diverse>",
        b"<fontawesome>not a font tag</fontawesome>",
        b"<tabletop>not a table tag</tabletop>",
        b"<styleguide>not a style tag</styleguide>",
        b"<titlecase>not a title tag</titlecase>",
        b"<bodyguard>not a body tag</bodyguard>",
        b"<bravo>not a br tag</bravo>",
    ] {
        let observed = classify_observed_mime(content, None);
        assert_eq!(observed.mime_type().essence(), "text/plain");
        assert_eq!(observed.risk_class(), ContentRiskClass::Passive);
    }

    for content in [
        b"<html>document</html>".as_slice(),
        b"<head profile=\"example\">",
        b"<script\tsrc=\"app.js\"></script>",
        b"<iframe\nname=\"content\"></iframe>",
        b"<h1>heading</h1>",
        b"<div class=\"panel\">content</div>",
        b"<font face=\"serif\">text</font>",
        b"<table><tr><td>cell</td></tr></table>",
        b"<style>body { display: block; }</style>",
        b"<title>title</title>",
        b"<body>body</body>",
        b"<br>break",
    ] {
        let observed = classify_observed_mime(content, None);
        assert_eq!(observed.mime_type().essence(), "text/html");
        assert_eq!(observed.risk_class(), ContentRiskClass::ActiveOrScriptable);
    }
}
