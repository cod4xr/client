// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Zone5Error;

/// Explicit PC zone type identifiers; a label does not admit a typed child payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Zone5AssetKind {
    /// Legacy model-pieces type, code zero.
    XModelPieces,
    /// Physics preset.
    PhysPreset,
    /// Animation parts.
    XAnimParts,
    /// Model.
    XModel,
    /// Material.
    Material,
    /// Material technique set.
    TechniqueSet,
    /// Graphics image.
    Image,
    /// Sound alias list.
    Sound,
    /// Sound curve.
    SoundCurve,
    /// Loaded sound.
    LoadedSound,
    /// Single-player collision map.
    ClipMap,
    /// Multiplayer collision map/PVS.
    ClipMapPvs,
    /// Common world.
    ComWorld,
    /// Single-player game world.
    GameWorldSp,
    /// Multiplayer game world.
    GameWorldMp,
    /// Map entity text asset.
    MapEntities,
    /// Graphics world.
    GfxWorld,
    /// Light definition.
    LightDef,
    /// Legacy UI map type.
    UiMap,
    /// Font.
    Font,
    /// Menu list.
    MenuList,
    /// Menu definition.
    Menu,
    /// Localization entry.
    LocalizeEntry,
    /// Weapon definition.
    Weapon,
    /// Sound-driver globals; the reviewed loader skips its link word.
    SoundDriverGlobals,
    /// Effect definition.
    Fx,
    /// Impact effect table.
    ImpactFx,
    /// Legacy AI type.
    AiType,
    /// Legacy multiplayer type.
    MpType,
    /// Legacy character type.
    Character,
    /// Legacy model alias.
    XModelAlias,
    /// Raw file.
    RawFile,
    /// String table, code thirty-two.
    StringTable,
}
const KINDS: [Zone5AssetKind; 33] = [
    Zone5AssetKind::XModelPieces,
    Zone5AssetKind::PhysPreset,
    Zone5AssetKind::XAnimParts,
    Zone5AssetKind::XModel,
    Zone5AssetKind::Material,
    Zone5AssetKind::TechniqueSet,
    Zone5AssetKind::Image,
    Zone5AssetKind::Sound,
    Zone5AssetKind::SoundCurve,
    Zone5AssetKind::LoadedSound,
    Zone5AssetKind::ClipMap,
    Zone5AssetKind::ClipMapPvs,
    Zone5AssetKind::ComWorld,
    Zone5AssetKind::GameWorldSp,
    Zone5AssetKind::GameWorldMp,
    Zone5AssetKind::MapEntities,
    Zone5AssetKind::GfxWorld,
    Zone5AssetKind::LightDef,
    Zone5AssetKind::UiMap,
    Zone5AssetKind::Font,
    Zone5AssetKind::MenuList,
    Zone5AssetKind::Menu,
    Zone5AssetKind::LocalizeEntry,
    Zone5AssetKind::Weapon,
    Zone5AssetKind::SoundDriverGlobals,
    Zone5AssetKind::Fx,
    Zone5AssetKind::ImpactFx,
    Zone5AssetKind::AiType,
    Zone5AssetKind::MpType,
    Zone5AssetKind::Character,
    Zone5AssetKind::XModelAlias,
    Zone5AssetKind::RawFile,
    Zone5AssetKind::StringTable,
];
impl Zone5AssetKind {
    /// Returns the stored type code; the explicit profile has thirty-three labels.
    #[must_use]
    pub fn code(self) -> u8 {
        self as u8
    }
    pub(super) fn from_word(value: u32) -> Result<Self, Zone5Error> {
        usize::try_from(value)
            .ok()
            .and_then(|index| KINDS.get(index).copied())
            .ok_or(Zone5Error::AssetKind)
    }
}
