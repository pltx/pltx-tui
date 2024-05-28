use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Default, Clone, Copy)]
pub struct WidgetMargin {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl WidgetMargin {
    pub const fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn zero() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }

    pub const fn uniform(margin: u16) -> Self {
        Self {
            top: margin,
            right: margin,
            bottom: margin,
            left: margin,
        }
    }

    pub const fn proportional(margin: u16) -> Self {
        Self {
            top: margin,
            right: margin * 2,
            bottom: margin,
            left: margin * 2,
        }
    }

    pub const fn symmetric(x: u16, y: u16) -> Self {
        Self {
            top: y,
            right: x,
            bottom: y,
            left: x,
        }
    }

    pub const fn vertical(margin: u16) -> Self {
        Self {
            top: margin,
            right: 0,
            bottom: margin,
            left: 0,
        }
    }

    pub const fn horizontal(margin: u16) -> Self {
        Self {
            top: 0,
            right: margin,
            bottom: 0,
            left: margin,
        }
    }

    pub const fn top(margin: u16) -> Self {
        Self {
            top: margin,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }

    pub const fn right(margin: u16) -> Self {
        Self {
            top: 0,
            right: margin,
            bottom: 0,
            left: 0,
        }
    }

    pub const fn bottom(margin: u16) -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: margin,
            left: 0,
        }
    }

    pub const fn left(margin: u16) -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: margin,
        }
    }

    /// Create a new layout with margins applied
    /// ```
    /// # use pltx_utils::WidgetMargin;
    /// # use ratatui::layout::Rect;
    /// let rect = Rect::new(5, 5, 10, 12);
    /// let margin = WidgetMargin::uniform(2);
    /// let margin_rect = margin.apply(rect);
    /// assert_eq!(margin_rect.x, 7);
    /// assert_eq!(margin_rect.y, 7);
    /// assert_eq!(margin_rect.width, 6);
    /// assert_eq!(margin_rect.height, 8);
    /// ```
    pub fn apply(&self, area: Rect) -> Rect {
        let [_, post_vertical_margin_layout, _] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.top),
                Constraint::Length(area.height.saturating_sub(self.top + self.bottom)),
                Constraint::Length(self.bottom),
            ])
            .areas(area);

        let [_, post_horizontal_margin_layout, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(self.left),
                Constraint::Length(
                    post_vertical_margin_layout
                        .width
                        .saturating_sub(self.left + self.right),
                ),
                Constraint::Length(self.right),
            ])
            .areas(post_vertical_margin_layout);

        post_horizontal_margin_layout
    }
}
