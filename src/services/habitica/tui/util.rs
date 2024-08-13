use ratatui::style::Color;

pub enum Direction {
  UP,
  DOWN,
  LEFT,
  RIGHT,
}

#[repr(u32)]
pub enum Palette {
  FG = 0x005c6166,    // #5c6166
  BG = 0x00fafafa,    // #fafafa
  BG2 = 0x00f0f0f0,   // #f0f0f0
  GREEN = 0x00e3f2c4, // #e3f2c4 
  RED,
  YELLOW
}

impl Into<Color> for Palette {
  fn into(self) -> Color {
    Color::from_u32(self as u32)
  }
}