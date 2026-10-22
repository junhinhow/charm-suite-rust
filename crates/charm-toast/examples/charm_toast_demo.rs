//! Demonstrates the `charm-toast` notifications.
//!
//! ```text
//! cargo run -p charm-toast --example charm_toast_demo
//! ```

use std::thread::sleep;
use std::time::Duration;

use charm_toast::{Toast, ToastManager, ToastType};

fn main() {
    let mut manager = ToastManager::new();
    manager.add_info("Conectando ao servidor", Duration::from_secs(2));
    manager.add_success("Dados sincronizados", Duration::from_secs(2));
    manager.add_warning("Espaco em disco baixo", Duration::from_secs(4));
    manager.add(Toast::new("Falha ao enviar", Duration::from_secs(4)).with_type(ToastType::Error));

    println!("{}", manager.render(60));

    sleep(Duration::from_secs(3));
    manager.cleanup();
    println!("\nAtivos apos 3s: {}", manager.count());
    println!("{}", manager.render(60));
}
