//! Demonstrates the `charm-dialog` boxes.
//!
//! ```text
//! cargo run -p charm-dialog --example charm_dialog_demo
//! ```

use charm_dialog::Dialog;

fn main() {
    let mut confirm = Dialog::new_confirm("Confirmar", "Deseja salvar as alteracoes?");
    println!("{}", confirm.render(60));

    confirm.next_button();
    println!("Botao selecionado: {}", confirm.selected_button());

    let mut input = Dialog::new_input("Entrada", "Digite seu nome:");
    input.input_value.push_str("Ana");
    println!("\n{}", input.render(60));

    let alert = Dialog::new_alert("Aviso", "Operacao concluida");
    println!("\n{}", alert.render_overlay(60, 14));
}
