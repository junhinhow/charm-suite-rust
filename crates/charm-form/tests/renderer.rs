//! Tests ported from the Go renderer model and runner.

use std::io::Cursor;

use charm_form::renderer::{self, Model, Options};
use charm_form::{
    Answers, FieldSpec, FormSpec, TYPE_CONFIRM, TYPE_INPUT, TYPE_MULTISELECT, TYPE_SELECT,
    TYPE_TEXT,
};
use charmed_bubbletea::{InterruptMsg, KeyMsg, KeyType, Message, Model as BubbleTeaModel};
use serde_json::json;

fn field(field_type: &str) -> FieldSpec {
    FieldSpec {
        name: "field".to_owned(),
        field_type: field_type.to_owned(),
        label: "Field".to_owned(),
        ..Default::default()
    }
}

fn spec_with(field: FieldSpec) -> FormSpec {
    FormSpec {
        title: "Title".to_owned(),
        description: "Description".to_owned(),
        fields: vec![field],
    }
}

fn send(model: &mut Model, key: KeyMsg) -> Option<charmed_bubbletea::Cmd> {
    BubbleTeaModel::update(model, Message::new(key))
}

fn type_text(model: &mut Model, text: &str) {
    for character in text.chars() {
        send(model, KeyMsg::from_char(character));
    }
}

#[test]
fn input_typing_confirmation_and_navigation_collects_answers() {
    let mut model = Model::new(FormSpec {
        title: "T".to_owned(),
        fields: vec![
            field(TYPE_INPUT),
            FieldSpec {
                name: "next".to_owned(),
                field_type: TYPE_INPUT.to_owned(),
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    type_text(&mut model, "abc");
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    assert_eq!(model.answers().get("field"), Some(&json!("abc")));
    assert!(!model.done());
    send(&mut model, KeyMsg::from_type(KeyType::Tab));
    type_text(&mut model, "done");
    assert!(send(&mut model, KeyMsg::from_type(KeyType::Enter)).is_some());
    assert!(model.done());
    assert_eq!(model.answers().get("next"), Some(&json!("done")));
}

#[test]
fn input_regex_and_required_validation_show_error_until_corrected() {
    let mut input = field(TYPE_INPUT);
    input.required = true;
    input.validate = Some("^[0-9]+$".to_owned());
    let mut model = Model::new(spec_with(input));
    assert!(send(&mut model, KeyMsg::from_type(KeyType::Enter)).is_none());
    assert!(model.view().contains("required"));
    type_text(&mut model, "abc");
    assert!(send(&mut model, KeyMsg::from_type(KeyType::Enter)).is_none());
    assert!(model.view().contains("does not match"));
    for _ in 0..3 {
        send(&mut model, KeyMsg::from_type(KeyType::Backspace));
    }
    type_text(&mut model, "123");
    assert!(send(&mut model, KeyMsg::from_type(KeyType::Enter)).is_some());
    assert_eq!(model.answers().get("field"), Some(&json!("123")));
}

#[test]
fn select_moves_wraps_and_returns_the_selected_option() {
    let mut select = field(TYPE_SELECT);
    select.options = Some(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]);
    let mut model = Model::new(spec_with(select));
    send(&mut model, KeyMsg::from_type(KeyType::Down));
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    assert_eq!(model.answers().get("field"), Some(&json!("b")));

    let mut select = field(TYPE_SELECT);
    select.options = Some(vec!["a".to_owned(), "b".to_owned()]);
    let mut model = Model::new(spec_with(select));
    send(&mut model, KeyMsg::from_type(KeyType::Up));
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    assert_eq!(model.answers().get("field"), Some(&json!("b")));
}

#[test]
fn multiselect_collects_checked_values_in_option_order() {
    let mut multi = field(TYPE_MULTISELECT);
    multi.options = Some(vec!["a".to_owned(), "b".to_owned()]);
    let mut model = Model::new(spec_with(multi));
    send(&mut model, KeyMsg::from_type(KeyType::Space));
    send(&mut model, KeyMsg::from_type(KeyType::Down));
    send(&mut model, KeyMsg::from_type(KeyType::Space));
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    assert_eq!(model.answers().get("field"), Some(&json!(["a", "b"])));
}

#[test]
fn confirm_toggles_y_and_accepts_false() {
    let mut model = Model::new(spec_with(field(TYPE_CONFIRM)));
    send(&mut model, KeyMsg::from_char('y'));
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    assert_eq!(model.answers().get("field"), Some(&json!(true)));

    let mut model = Model::new(spec_with(field(TYPE_CONFIRM)));
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    assert_eq!(model.answers().get("field"), Some(&json!(false)));
}

#[test]
fn multiline_text_uses_enter_for_newline_and_ctrl_s_to_confirm() {
    let mut model = Model::new(spec_with(field(TYPE_TEXT)));
    type_text(&mut model, "line1");
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    type_text(&mut model, "line2");
    assert!(send(&mut model, KeyMsg::from_type(KeyType::CtrlS)).is_some());
    assert_eq!(model.answers().get("field"), Some(&json!("line1\nline2")));
}

#[test]
fn cancellation_and_interrupt_return_quit_commands() {
    let mut model = Model::new(spec_with(field(TYPE_INPUT)));
    let quit = send(&mut model, KeyMsg::from_type(KeyType::Esc));
    assert!(quit.is_some());
    assert!(model.cancelled());

    let mut model = Model::new(spec_with(field(TYPE_INPUT)));
    let quit = BubbleTeaModel::update(&mut model, Message::new(InterruptMsg));
    assert!(quit.is_some());
    assert!(model.cancelled());
}

#[test]
fn shift_tab_goes_back_and_paste_inserts_text() {
    let mut model = Model::new(FormSpec {
        title: "T".to_owned(),
        fields: vec![
            field(TYPE_INPUT),
            FieldSpec {
                name: "next".to_owned(),
                field_type: TYPE_INPUT.to_owned(),
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    send(&mut model, KeyMsg::from_type(KeyType::Tab));
    send(&mut model, KeyMsg::from_type(KeyType::ShiftTab));
    BubbleTeaModel::update(
        &mut model,
        Message::new(KeyMsg::from_runes("pasted".chars().collect()).with_paste()),
    );
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    assert_eq!(model.answers().get("field"), Some(&json!("pasted")));
}

#[test]
fn view_shows_title_description_label_and_validation_error() {
    let mut input = field(TYPE_INPUT);
    input.validate = Some("^[0-9]+$".to_owned());
    let mut model = Model::new(spec_with(input));
    for text in ["Title", "Description", "Field"] {
        assert!(model.view().contains(text));
    }
    type_text(&mut model, "abc");
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    assert!(model.view().contains("does not match"));
}

#[test]
fn model_normalizes_field_types_and_empty_models_are_safe() {
    let mut model = Model::new(FormSpec {
        title: "T".to_owned(),
        fields: vec![FieldSpec {
            name: "a".to_owned(),
            field_type: "Input".to_owned(),
            ..Default::default()
        }],
        ..Default::default()
    });
    type_text(&mut model, "x");
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    assert_eq!(model.answers().get("a"), Some(&json!("x")));

    let mut empty = Model::new(FormSpec::default());
    assert!(empty.view().contains("esc cancel"));
    assert!(send(&mut empty, KeyMsg::from_type(KeyType::Enter)).is_none());
}

#[test]
fn run_with_options_collects_scripted_answers() {
    let spec = FormSpec {
        title: "Form".to_owned(),
        fields: vec![
            FieldSpec {
                name: "name".to_owned(),
                field_type: TYPE_INPUT.to_owned(),
                required: true,
                ..Default::default()
            },
            FieldSpec {
                name: "age".to_owned(),
                field_type: TYPE_INPUT.to_owned(),
                validate: Some("^[0-9]+$".to_owned()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let answers = renderer::run_with_options(
        spec,
        Options {
            input: Some(Box::new(Cursor::new(b"Ana\r30\r".to_vec()))),
            output: Some(Box::new(Vec::<u8>::new())),
            width: 80,
            height: 24,
        },
    )
    .expect("scripted run succeeds");
    assert_eq!(answers.get("name"), Some(&json!("Ana")));
    assert_eq!(answers.get("age"), Some(&json!("30")));
}

#[test]
fn run_with_options_reports_cancelled_input() {
    let result = renderer::run_with_options(
        spec_with(field(TYPE_INPUT)),
        Options {
            input: Some(Box::new(Cursor::new(vec![3]))),
            output: Some(Box::new(Vec::<u8>::new())),
            width: 80,
            height: 24,
        },
    );
    assert!(matches!(result, Err(charm_form::Error::Cancelled)));
}

#[test]
fn answers_returned_by_model_are_independent_copies() {
    let mut model = Model::new(spec_with(field(TYPE_INPUT)));
    type_text(&mut model, "original");
    send(&mut model, KeyMsg::from_type(KeyType::Enter));
    let mut copy: Answers = model.answers();
    copy.insert("field".to_owned(), json!("changed"));
    assert_eq!(model.answers().get("field"), Some(&json!("original")));
}
