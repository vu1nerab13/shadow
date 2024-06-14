use super::Display;
use display_info::DisplayInfo;

impl From<DisplayInfo> for Display {
    fn from(value: DisplayInfo) -> Self {
        Self {
            name: value.name,
            id: value.id,
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
            rotation: value.rotation,
            scale_factor: value.scale_factor,
            frequency: value.frequency,
            is_primary: value.is_primary,
        }
    }
}
