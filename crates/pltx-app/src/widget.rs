use ratatui::{layout::Rect, Frame};

use crate::{state::View, App, KeyEventHandler};

/// Create a default widget, which only required the `render()` method.
pub trait DefaultWidget {
    /// Render the widget.
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused: bool);
}

/// Create a form widget. Used by the Form widget to
/// create forms.
pub trait FormWidgetOld: DefaultWidget + KeyEventHandler {
    /// Modify the widget values to ensure it is form form compatible.
    fn form_compatible(&mut self);
    /// Set the view that the widget should be interactable in. Keys will not
    /// be processed if the app is in any other view.
    fn view(&mut self, view: View);
    /// Reset the state values of the widget.
    fn reset(&mut self);
}

/// Create a composite widget. Contains multiple focusable elements, such as a
/// selection.
pub trait CompositeWidget {
    /// Focus on the first element.
    fn focus_first(&mut self);
    /// Focus on the last element.
    fn focus_last(&mut self);
    /// Focus on the next element.
    fn focus_next(&mut self);
    /// Focus on the previus element.
    fn focus_prev(&mut self);
    /// Check if the first element has the focus.
    fn is_focus_first(&self) -> bool;
    /// Check if the last element has the focus.
    fn is_focus_last(&self) -> bool;
}
