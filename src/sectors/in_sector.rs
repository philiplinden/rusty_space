use crate::sectors::sector_entity::SectorEntity;
use bevy::prelude::Component;

/// Component for entities inside sectors.
///
/// These are managed by [SectorData], so if you feel a need to manually add or remove them,
/// you should probably use the respective functions there.
#[derive(Component, PartialEq, Eq)]
pub struct InSector {
    pub(in crate::sectors) sector: SectorEntity,
}

impl InSector {
    pub fn get(&self) -> SectorEntity {
        self.sector
    }
}

impl PartialEq<SectorEntity> for InSector {
    fn eq(&self, other: &SectorEntity) -> bool {
        &self.sector == other
    }
}

impl PartialEq<SectorEntity> for &InSector {
    fn eq(&self, other: &SectorEntity) -> bool {
        &self.sector == other
    }
}
