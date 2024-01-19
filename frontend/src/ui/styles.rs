use iced::widget::button::{self, Appearance};
use iced::widget::container::StyleSheet;
use iced::{theme, Theme};
pub static DEFAULT_TEXT_SIZE: f32 = 16.0;

pub struct CircleButtonStyle {
  theme: theme::Button,
}

impl CircleButtonStyle {
  pub fn new(theme: theme::Button) -> Self {
    Self { theme }
  }
}

impl button::StyleSheet for CircleButtonStyle {
  type Style = Theme;

  fn active(&self, style: &Self::Style) -> Appearance {
    let mut appearance = style.active(&self.theme);
    appearance.border_radius = 25.0.into();

    appearance
  }
}

pub struct ListPreviewContainerStyle {
  theme: theme::Container,
}

impl ListPreviewContainerStyle {
  pub fn new(theme: theme::Container) -> Self {
    Self { theme: theme }
  }
}

impl iced::widget::container::StyleSheet for ListPreviewContainerStyle {
  type Style = Theme;

  fn appearance(&self, style: &Self::Style) -> iced::widget::container::Appearance {
    let mut appearance = style.appearance(&self.theme);
    appearance.border_width = 1.0;
    appearance.border_radius = 5.0.into();

    match style {
      Theme::Light => {
        appearance.border_color = iced::Color::BLACK;
      }
      Theme::Dark | Theme::Custom(_) => {
        appearance.border_color = iced::Color::WHITE;
      }
    }

    appearance
  }
}
