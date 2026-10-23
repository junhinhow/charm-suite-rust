#![allow(missing_docs)]

use charm_input::{Date, Masked, Range, Time};
use charmed_bubbletea::{KeyMsg, KeyType, Message};
use chrono::NaiveDate;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).expect("valid")
}

fn apply_date(model: &mut Date, msg: Message) {
    model.update(msg);
}
fn apply_time(model: &mut Time, msg: Message) {
    model.update(msg);
}
fn apply_range(model: &mut Range, msg: Message) {
    model.update(msg);
}
fn apply_masked(model: &mut Masked, msg: Message) {
    model.update(msg);
}

#[test]
fn date_navigation() {
    let mut dd = Date::new(d(2026, 10, 22));
    dd.focus();
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::Left)));
    assert_eq!(dd.value(), d(2026, 10, 21));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::Right)));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::Right)));
    assert_eq!(dd.value(), d(2026, 10, 23));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::Up)));
    assert_eq!(dd.value(), d(2026, 10, 16));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::Down)));
    assert_eq!(dd.value(), d(2026, 10, 23));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::CtrlLeft)));
    assert_eq!(dd.value(), d(2026, 9, 23));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::CtrlRight)));
    assert_eq!(dd.value(), d(2026, 10, 23));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::PgUp)));
    assert_eq!(dd.value(), d(2025, 10, 23));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::PgDown)));
    assert_eq!(dd.value(), d(2026, 10, 23));
}

#[test]
fn date_clamp_min_max() {
    let mut dd = Date::new(d(2026, 10, 22));
    dd.focus();
    dd.set_min(d(2026, 10, 20));
    dd.set_max(d(2026, 10, 25));
    for _ in 0..3 {
        apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::Left)));
    }
    assert_eq!(dd.value(), d(2026, 10, 20));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::CtrlLeft)));
    assert_eq!(dd.value(), d(2026, 10, 20));
    for _ in 0..10 {
        apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::Right)));
    }
    assert_eq!(dd.value(), d(2026, 10, 25));
    dd.set_value(d(2026, 10, 1));
    assert_eq!(dd.value(), d(2026, 10, 20));
    dd.set_value(d(2026, 10, 30));
    assert_eq!(dd.value(), d(2026, 10, 25));
}

#[test]
fn date_inverted_bounds() {
    let mut dd = Date::new(d(2026, 10, 22));
    dd.set_min(d(2026, 10, 30));
    dd.set_max(d(2026, 10, 1));
    assert_eq!(dd.value(), d(2026, 10, 30));
    dd.focus();
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::Left)));
    assert_eq!(dd.value(), d(2026, 10, 29));
}

#[test]
fn date_leap() {
    let mut dd = Date::new(d(2028, 2, 29));
    dd.focus();
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::CtrlRight)));
    assert_eq!(dd.value(), d(2028, 3, 29));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::CtrlLeft)));
    assert_eq!(dd.value(), d(2028, 2, 29));
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::PgUp)));
    assert_eq!(dd.value(), d(2027, 2, 28));
}

#[test]
fn date_format() {
    let mut dd = Date::new(d(2026, 10, 22));
    dd.set_size(40, 1);
    assert_eq!(dd.view(), "22/10/2026");
    dd.set_format("2006-01-02");
    assert_eq!(dd.view(), "2026-10-22");
    dd.set_format("");
    assert_eq!(dd.view(), "22/10/2026");
}

#[test]
fn date_compact_mode() {
    let mut dd = Date::new(d(2026, 10, 22));
    dd.set_size(40, 7);
    assert_eq!(dd.view(), "22/10/2026");
    dd.set_size(40, 8);
    let out = dd.view();
    let lines: Vec<&str> = out.split('\n').collect();
    assert_eq!(lines.len(), 8);
    assert!(lines[0].contains("October 2026"));
    assert_eq!(lines[1], "Su Mo Tu We Th Fr Sa");
}

#[test]
fn date_today_highlight() {
    let mut dd = Date::new(d(2026, 10, 22));
    dd.set_today(d(2026, 10, 5));
    dd.set_size(40, 8);
    let out = dd.view();
    assert!(out.contains("* 5"));
    assert!(out.contains("[22]"));
    assert!(out.split('\n').next().unwrap().contains('*'));
    assert_eq!(dd.value(), d(2026, 10, 22));
    dd.set_today(d(2026, 12, 1));
    let out2 = dd.view();
    assert!(!out2.split('\n').next().unwrap().contains('*'));
}

#[test]
fn date_enter_emits() {
    let mut dd = Date::new(d(2026, 10, 22));
    dd.focus();
    let cmd = dd.update(Message::new(KeyMsg::from_type(KeyType::Enter)));
    assert!(cmd.is_some());
    let msg = cmd.unwrap().execute().unwrap();
    let sel = msg.downcast_ref::<charm_input::DateSelectedMsg>().unwrap();
    assert_eq!(sel.value, d(2026, 10, 22));
}

#[test]
fn date_size_zero() {
    let mut dd = Date::new(d(2026, 10, 22));
    for w in [0, 10, 40, 80] {
        dd.set_size(w, 8);
        let out = dd.view();
        if w <= 0 {
            assert_eq!(out, "");
        } else {
            assert_ne!(out, "");
        }
    }
    dd.set_size(40, 0);
    assert_eq!(dd.view(), "");
    dd.set_size(40, -1);
    assert_eq!(dd.view(), "");
}

#[test]
fn date_focus_no_change_view() {
    let mut dd = Date::new(d(2026, 10, 22));
    dd.set_size(40, 8);
    let before = dd.view();
    dd.focus();
    assert!(dd.focused());
    // view same regardless of focus
    assert_eq!(dd.view(), before);
    dd.blur();
    apply_date(&mut dd, Message::new(KeyMsg::from_type(KeyType::Left)));
    assert_eq!(dd.value(), d(2026, 10, 22));
}

#[test]
fn date_deterministic() {
    let mut dd = Date::new(d(2026, 10, 22));
    dd.set_today(d(2026, 10, 22));
    dd.set_size(40, 8);
    let first = dd.view();
    for _ in 0..100 {
        assert_eq!(dd.view(), first);
    }
}

#[test]
fn time_string_view() {
    let mut clock = Time::new(15, 4, 5);
    assert_eq!(clock.string(), "15:04:05");
    clock.set_size(40, 1);
    assert_eq!(clock.view(), "15:04:05");
    clock.focus();
    assert_eq!(clock.view(), "[15]:04:05");
    clock.set_second_visible(false);
    assert_eq!(clock.view(), "[15]:04");
    clock.blur();
    clock.set_24h(false);
    assert_eq!(clock.view(), "03:04 PM");
    clock.set_second_visible(true);
    assert_eq!(clock.view(), "03:04:05 PM");
}

#[test]
fn time_wrap() {
    let mut clock = Time::new(23, 59, 59);
    clock.focus();
    apply_time(&mut clock, Message::new(KeyMsg::from_type(KeyType::Up)));
    assert_eq!(clock.value(), (0, 59, 59));
    apply_time(&mut clock, Message::new(KeyMsg::from_type(KeyType::Right)));
    apply_time(&mut clock, Message::new(KeyMsg::from_type(KeyType::Up)));
    assert_eq!(clock.value(), (0, 0, 59));
    apply_time(&mut clock, Message::new(KeyMsg::from_type(KeyType::Right)));
    apply_time(&mut clock, Message::new(KeyMsg::from_type(KeyType::Up)));
    assert_eq!(clock.value(), (0, 0, 0));
    apply_time(&mut clock, Message::new(KeyMsg::from_type(KeyType::Down)));
    assert_eq!(clock.value(), (0, 0, 59));
}

#[test]
fn time_ampm() {
    for (hour, want) in [
        (0, "12:05 AM"),
        (12, "12:05 PM"),
        (13, "01:05 PM"),
        (23, "11:05 PM"),
    ] {
        let mut clock = Time::new(hour, 5, 0);
        clock.set_second_visible(false);
        clock.set_24h(false);
        clock.set_size(40, 1);
        assert_eq!(clock.view(), want);
    }
}

#[test]
fn range_basic() {
    let mut r = Range::new(0.0, 100.0, 50.0);
    r.focus();
    apply_range(&mut r, Message::new(KeyMsg::from_type(KeyType::Right)));
    assert_eq!(r.value(), 51.0);
    apply_range(&mut r, Message::new(KeyMsg::from_type(KeyType::Left)));
    assert_eq!(r.value(), 50.0);
    r.set_step(5.0);
    apply_range(&mut r, Message::new(KeyMsg::from_type(KeyType::Right)));
    assert_eq!(r.value(), 55.0);
    apply_range(&mut r, Message::new(KeyMsg::from_type(KeyType::Home)));
    assert_eq!(r.value(), 0.0);
    apply_range(&mut r, Message::new(KeyMsg::from_type(KeyType::End)));
    assert_eq!(r.value(), 100.0);
    apply_range(&mut r, Message::new(KeyMsg::from_type(KeyType::PgUp)));
    // from 100, +10% of range (10) clamped to 100
    assert_eq!(r.value(), 100.0);
}

#[test]
fn range_clamp_nan() {
    let r = Range::new(0.0, 100.0, f64::NAN);
    assert_eq!(r.value(), 0.0);
    let swapped = Range::new(10.0, 0.0, 5.0);
    assert_eq!(swapped.value(), 5.0);
    assert_eq!(swapped.percent(), 50.0);
    let deg = Range::new(5.0, 5.0, 5.0);
    assert_eq!(deg.percent(), 0.0);
}

#[test]
fn range_bar() {
    let mut r = Range::new(0.0, 10.0, 5.0);
    r.set_size(20, 1);
    let out = r.view();
    // 9 filled + 9 empty + space + 5
    assert!(out.contains('5'));
}

#[serial_test::serial]
#[test]
fn masked_basic() {
    std::env::set_var("NO_COLOR", "1");
    let mut m = Masked::new('*');
    m.set_size(40, 1);
    m.focus();
    apply_masked(&mut m, Message::new(KeyMsg::from_char('a')));
    apply_masked(&mut m, Message::new(KeyMsg::from_char('b')));
    apply_masked(&mut m, Message::new(KeyMsg::from_char('c')));
    assert_eq!(m.value(), "abc");
    assert_eq!(m.masked(), "***");
    assert_eq!(m.view(), "***|");
    apply_masked(&mut m, Message::new(KeyMsg::from_type(KeyType::Backspace)));
    assert_eq!(m.value(), "ab");
    assert_eq!(m.view(), "**|");
    apply_masked(&mut m, Message::new(KeyMsg::from_type(KeyType::CtrlU)));
    assert_eq!(m.value(), "");
    assert_eq!(m.view(), "|");
    std::env::remove_var("NO_COLOR");
}

#[serial_test::serial]
#[test]
fn masked_reveal() {
    std::env::set_var("NO_COLOR", "1");
    let mut m = Masked::new('*');
    m.set_value("segredo");
    m.set_size(40, 1);
    m.focus();
    assert_eq!(m.view(), "*******|");
    apply_masked(&mut m, Message::new(KeyMsg::from_type(KeyType::CtrlR)));
    assert_eq!(m.view(), "segredo|");
    apply_masked(&mut m, Message::new(KeyMsg::from_char('!')));
    assert_eq!(m.value(), "segredo!");
    assert_eq!(m.view(), "********|");
    std::env::remove_var("NO_COLOR");
}

#[serial_test::serial]
#[test]
fn masked_unicode() {
    std::env::set_var("NO_COLOR", "1");
    let mut m = Masked::new('*');
    m.set_value("café");
    m.set_size(40, 1);
    m.focus();
    // café = 4 chars
    assert_eq!(m.masked(), "****");
    m.reveal(true);
    assert!(m.view().contains("café"));
    std::env::remove_var("NO_COLOR");
}

#[test]
fn masked_default_mask() {
    for mask in ['\0', '\n', '\t'] {
        let mut m = Masked::new(mask);
        m.set_value("ab");
        assert_eq!(m.masked(), "**");
    }
    let mut z = Masked::new('\0');
    z.set_value("ab");
    assert_eq!(z.masked(), "**");
}

#[test]
fn masked_sanitize() {
    let mut m = Masked::new('*');
    m.set_value("a\nb\tc\u{1b}");
    assert_eq!(m.value(), "abc");
    assert_eq!(m.masked(), "***");
}

#[serial_test::serial]
#[test]
fn masked_size_zero() {
    std::env::set_var("NO_COLOR", "1");
    let mut m = Masked::new('*');
    m.set_value("long text");
    m.focus();
    for w in [0, 10, 40, 80] {
        m.set_size(w, 1);
        let out = m.view();
        if w <= 0 {
            assert_eq!(out, "");
        } else {
            assert_ne!(out, "");
            assert!(out.ends_with('|'));
        }
    }
    m.set_size(40, 0);
    assert_eq!(m.view(), "");
    m.set_size(40, -1);
    assert_eq!(m.view(), "");
    std::env::remove_var("NO_COLOR");
}
