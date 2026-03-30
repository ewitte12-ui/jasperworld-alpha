use bevy::prelude::*;

// Colors
pub const COLOR_BACKGROUND: Color = Color::srgb(0.1, 0.1, 0.15);
pub const COLOR_BUTTON_NORMAL: Color = Color::srgb(0.25, 0.25, 0.3);
pub const COLOR_BUTTON_HOVERED: Color = Color::srgb(0.35, 0.35, 0.4);
pub const COLOR_BUTTON_PRESSED: Color = Color::srgb(0.15, 0.15, 0.2);
pub const COLOR_BUTTON_ACTIVE_TAB: Color = Color::srgb(0.3, 0.3, 0.5);
pub const COLOR_TEXT: Color = Color::srgb(0.9, 0.9, 0.9);
pub const COLOR_TEXT_TITLE: Color = Color::srgb(0.95, 0.85, 0.4);
pub const COLOR_TEXT_SUBTITLE: Color = Color::srgb(0.7, 0.7, 0.7);
pub const COLOR_OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);
pub const COLOR_SLOT_EMPTY: Color = Color::srgb(0.2, 0.2, 0.25);
pub const COLOR_SLOT_OCCUPIED: Color = Color::srgb(0.25, 0.3, 0.25);

// Font sizes
pub const FONT_SIZE_TITLE: f32 = 64.0;
pub const FONT_SIZE_HEADING: f32 = 40.0;
pub const FONT_SIZE_BUTTON: f32 = 28.0;
pub const FONT_SIZE_BODY: f32 = 22.0;
pub const FONT_SIZE_SMALL: f32 = 18.0;

// Button sizes
pub const BUTTON_WIDTH: Val = Val::Px(300.0);
pub const BUTTON_HEIGHT: Val = Val::Px(55.0);
pub const BUTTON_SMALL_WIDTH: Val = Val::Px(50.0);
pub const BUTTON_SMALL_HEIGHT: Val = Val::Px(40.0);
pub const BUTTON_TAB_WIDTH: Val = Val::Px(160.0);
pub const BUTTON_TAB_HEIGHT: Val = Val::Px(45.0);

// Spacing
pub const MENU_GAP: Val = Val::Px(12.0);
pub const SECTION_GAP: Val = Val::Px(24.0);
