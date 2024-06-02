//! Reusable widgets. Widgets implement
//! [`DefaultWidget`](pltx_app::DefaultWidget) or
//! [`CompositeWidget`](pltx_app::CompositeWidget), which must be imported to
//! call the `render()` method.

mod buttons;
mod card;
mod form;
mod input;
mod popup;
mod scrollable;
mod selection;
mod switch;
mod tabs;

pub use buttons::*;
pub use card::*;
pub use form::*;
pub use input::*;
pub use popup::*;
pub use scrollable::*;
pub use selection::*;
pub use switch::*;
pub use tabs::*;
