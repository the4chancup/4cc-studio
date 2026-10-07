//! The deep pass's texture checks (`team_compiler/messages.md` "Textures"), from a
//! texture's header alone: a file renamed from another format, a side under one block, and
//! the size rules a texture is held to by where it is and the target's engine.

use dds_convert::SourceFormat;
use pes_version::{Engine, PesVersion};

use crate::messages::Code;

/// The size rule a texture is held to past the smallest side (`messages.md` "Textures").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SizeRule {
    /// A kit's main texture, on any target: `kit_texture_too_big` past 2048 pixels a side or
    /// on a side that is not a power of two.
    MainKit,
    /// Any other texture on a Fox target: `texture_not_pow2` on a side that is not a power of
    /// two when the converted texture carries a mip chain.
    FoxMipmapped,
    /// Any other texture on a pre-Fox target: no size rule here.
    PreFox,
    /// A portrait, a `Portraits/` file or a player folder's `portrait.*`, on any target:
    /// `texture_not_pow2` on a side that is not a power of two, whatever its level count.
    Portrait,
}

impl SizeRule {
    /// The rule of every texture but a kit's main one, on `version`'s engine.
    pub(super) fn of(version: PesVersion) -> SizeRule {
        match version.engine() {
            Engine::Fox => SizeRule::FoxMipmapped,
            Engine::PreFox => SizeRule::PreFox,
        }
    }
}

/// The accepted formats that open with a fixed signature, and that signature; WebP's `RIFF`
/// is checked with its `WEBP` tag below, and TGA has none.
const SIGNATURES: [(&[u8], SourceFormat); 7] = [
    (b"DDS ", SourceFormat::Dds),
    (b"FTEX", SourceFormat::Ftex),
    (b"\x89PNG", SourceFormat::Png),
    (&[0xff, 0xd8, 0xff], SourceFormat::Jpeg),
    (b"BM", SourceFormat::Bmp),
    (b"II*\0", SourceFormat::Tiff),
    (b"MM\0*", SourceFormat::Tiff),
];

/// The accepted format whose signature `bytes` open with, if any.
fn signature_format(bytes: &[u8]) -> Option<SourceFormat> {
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(SourceFormat::WebP);
    }
    SIGNATURES
        .iter()
        .find(|(signature, _)| bytes.starts_with(signature))
        .map(|(_, format)| *format)
}

/// The one texture finding `bytes`, a texture in `format` held to `rule`, get, by the first
/// rule that fires: `texture_type_mismatch` when they open with another accepted format's
/// signature (a file renamed, not resaved; its header is not read), then, from the header,
/// `texture_too_small` on a side under 4 pixels (one block), then the size rule. A header
/// that cannot be read is no finding: converting the texture fails its task.
pub(super) fn texture_finding(format: SourceFormat, rule: SizeRule, bytes: &[u8]) -> Option<Code> {
    if signature_format(bytes).is_some_and(|sniffed| sniffed != format) {
        return Some(Code::TextureTypeMismatch);
    }
    // The error is not reported here: the texture's task meets it again and fails on it.
    let Ok(probe) = dds_convert::probe(bytes, format) else {
        return None;
    };
    if probe.width < 4 || probe.height < 4 {
        return Some(Code::TextureTooSmall);
    }
    let power_of_two = probe.width.is_power_of_two() && probe.height.is_power_of_two();
    match rule {
        SizeRule::MainKit => (probe.width > 2048 || probe.height > 2048 || !power_of_two)
            .then_some(Code::KitTextureTooBig),
        SizeRule::FoxMipmapped => {
            (probe.mipmaps > 1 && !power_of_two).then_some(Code::TextureNotPow2)
        }
        SizeRule::PreFox => None,
        SizeRule::Portrait => (!power_of_two).then_some(Code::TextureNotPow2),
    }
}

#[cfg(test)]
mod tests {
    use aesthetics_export::Disposition;

    use super::*;
    use crate::deep::tests::{
        bc1_dds, findings_for, findings_of, fixture, folder, texture, texture_finding_on,
    };
    use crate::testing::scratch;

    #[test]
    fn each_accepted_format_s_signature_is_recognized_and_tga_has_none() {
        // A real file of each format opens with its signature (the kit and portrait fixtures
        // are real encoder output); a signature pasted onto nothing is still that format's.
        assert_eq!(
            signature_format(&fixture("tracer/studio/egg Midcup Tracer/Kits/g1/kit.dds")),
            Some(SourceFormat::Dds),
            "dds"
        );
        assert_eq!(
            signature_format(&texture("kit.png")),
            Some(SourceFormat::Png),
            "png"
        );
        assert_eq!(
            signature_format(&texture("portrait.webp")),
            Some(SourceFormat::WebP),
            "webp"
        );
        assert_eq!(
            signature_format(&texture("kit_back.tga")),
            None,
            "tga has no signature"
        );
        for (bytes, format) in [
            (&b"FTEX\x00\x00\x00\x00"[..], SourceFormat::Ftex),
            (&[0xff, 0xd8, 0xff, 0xe0, 0, 0x10], SourceFormat::Jpeg),
            (&b"BM\x36\x00\x00\x00"[..], SourceFormat::Bmp),
            (&b"II*\0\x08\x00\x00\x00"[..], SourceFormat::Tiff),
            (&b"MM\0*\x00\x00\x00\x08"[..], SourceFormat::Tiff),
            (&b"RIFF\x00\x00\x00\x00WEBPVP8 "[..], SourceFormat::WebP),
        ] {
            assert_eq!(signature_format(bytes), Some(format), "{format:?}");
        }
        // A RIFF that is not WebP, and bytes opening with none of them, are no format.
        assert_eq!(signature_format(b"RIFF\x00\x00\x00\x00WAVEfmt "), None);
        assert_eq!(signature_format(b"RIFF"), None, "too short for the tag");
        assert_eq!(signature_format(b"not a texture"), None);
        assert_eq!(signature_format(b""), None);
    }

    #[test]
    fn each_texture_rule_drops_its_player_folder_with_one_finding() {
        let temp = scratch("deep_texture_rules");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/tiny.png", texture("tiny.png")),
                ("Players/04 - B/face/odd.png", texture("odd.png")),
                // PNG bytes under a `.dds` and a `.tga` name: renamed, not resaved; TGA has
                // no signature, but PNG's is another's.
                ("Players/05 - C/skin.dds", texture("portrait.png")),
                ("Players/06 - D/skin.tga", texture("tiny.png")),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [
                texture_finding_on(
                    "texture_too_small",
                    &folder("Players/03 - A"),
                    "tiny.png",
                    Disposition::DropFolder,
                    true
                ),
                texture_finding_on(
                    "texture_not_pow2",
                    &folder("Players/04 - B"),
                    "face/odd.png",
                    Disposition::DropFolder,
                    true
                ),
                texture_finding_on(
                    "texture_type_mismatch",
                    &folder("Players/05 - C"),
                    "skin.dds",
                    Disposition::DropFolder,
                    false
                ),
                texture_finding_on(
                    "texture_type_mismatch",
                    &folder("Players/06 - D"),
                    "skin.tga",
                    Disposition::DropFolder,
                    false
                ),
            ]
        );
    }

    #[test]
    fn a_single_level_odd_dds_on_fox_and_an_odd_png_on_pre_fox_pass() {
        let temp = scratch("deep_texture_fox_single_level");
        let findings = findings_of(
            temp.path(),
            &[("Players/03 - A/skin.dds", bc1_dds(300, 300))],
            &[],
        );
        assert_eq!(findings, []);
        let temp = scratch("deep_texture_pre_fox");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[("Players/03 - A/odd.png", texture("odd.png"))],
            &[],
            &[],
        );
        assert_eq!(findings, []);
    }

    #[test]
    fn a_texture_whose_header_is_cut_gets_no_finding() {
        let temp = scratch("deep_texture_cut");
        let findings = findings_of(
            temp.path(),
            &[
                (
                    "Players/03 - A/skin.dds",
                    texture("bc7.dds")[..100].to_vec(),
                ),
                (
                    "Players/03 - A/hair.png",
                    texture("tiny.png")[..20].to_vec(),
                ),
            ],
            &[],
        );
        assert_eq!(findings, []);
    }

    #[test]
    fn a_kit_s_main_texture_too_big_or_odd_is_kit_texture_too_big_alone() {
        let temp = scratch("deep_texture_kits");
        let findings = findings_of(
            temp.path(),
            &[
                ("Kits/p1/kit.dds", bc1_dds(4096, 4096)),
                ("Kits/p2/kit.png", texture("odd.png")),
                ("Kits/g1/kit.dds", bc1_dds(2048, 2048)),
                ("Kits/p3/kit.png", texture("kit.png")),
                ("Kits/p3/kit_back.png", texture("odd.png")),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [
                texture_finding_on(
                    "kit_texture_too_big",
                    &folder("Kits/p1"),
                    "kit.dds",
                    Disposition::DropFolder,
                    true
                ),
                texture_finding_on(
                    "kit_texture_too_big",
                    &folder("Kits/p2"),
                    "kit.png",
                    Disposition::DropFolder,
                    true
                ),
                texture_finding_on(
                    "texture_not_pow2",
                    &folder("Kits/p3"),
                    "kit_back.png",
                    Disposition::DropFolder,
                    true
                ),
            ]
        );
    }

    #[test]
    fn each_size_rule_reads_each_side_on_its_own() {
        // Single-level DDS headers, so only the size matters: one side past a bound and the
        // other not, and a side exactly at a bound.
        for (rule, width, height, expected) in [
            (SizeRule::FoxMipmapped, 3, 8, Some(Code::TextureTooSmall)),
            (SizeRule::FoxMipmapped, 8, 3, Some(Code::TextureTooSmall)),
            (SizeRule::FoxMipmapped, 4, 8, None),
            (SizeRule::FoxMipmapped, 8, 4, None),
            (SizeRule::MainKit, 256, 300, Some(Code::KitTextureTooBig)),
            (SizeRule::MainKit, 300, 256, Some(Code::KitTextureTooBig)),
            (SizeRule::MainKit, 4096, 2048, Some(Code::KitTextureTooBig)),
            (SizeRule::MainKit, 2048, 4096, Some(Code::KitTextureTooBig)),
            (SizeRule::MainKit, 2048, 1024, None),
            (SizeRule::PreFox, 3, 8, Some(Code::TextureTooSmall)),
            (SizeRule::Portrait, 256, 300, Some(Code::TextureNotPow2)),
            (SizeRule::Portrait, 300, 256, Some(Code::TextureNotPow2)),
            (SizeRule::Portrait, 4096, 8, None),
            (SizeRule::Portrait, 3, 8, Some(Code::TextureTooSmall)),
        ] {
            assert_eq!(
                texture_finding(SourceFormat::Dds, rule, &bc1_dds(width, height)),
                expected,
                "{rule:?} {width}x{height}"
            );
        }
    }
}
