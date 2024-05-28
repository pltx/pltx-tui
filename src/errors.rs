use std::panic;

use color_eyre::{config::HookBuilder, eyre, Result};

use crate::tui::Tui;

/// This replaces the standard color_eyre panic and error hooks with hooks that
/// restore the terminal before printing the panic or error.
pub fn install_hooks() -> Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    // Convert from a color_eyre PanicHook to a standard panic hook.
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        Tui::restore().expect("failed to restore the terminal");
        panic_hook(panic_info);
    }));

    // Convert from a color_eyre EyreHook to a eyre ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            Tui::restore().expect("failed to restore the terminal");
            eyre_hook(error)
        },
    ))?;

    Ok(())
}
