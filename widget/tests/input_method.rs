//! Tests for the input method integration of text widgets.
//!
//! A text widget that loses focus while the input method is still composing
//! must commit the composed text, like native text fields do; otherwise the
//! last syllable typed with a Korean (or any CJK) input method is lost.
use iced_test::{Error, Simulator};
use iced_widget::core::input_method;
use iced_widget::core::keyboard::{self, key};
use iced_widget::core::widget::Id;
use iced_widget::core::{Element, Event, Theme};
use iced_widget::text_editor::{Action, Content, Edit};
use iced_widget::{Renderer, button, column, text_editor, text_input};

fn simulator<'a, Message>(
    element: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Simulator<'a, Message, Theme, Renderer> {
    Simulator::new(element)
}

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Input(String),
    Send,
}

fn preedit(content: &str) -> Event {
    Event::InputMethod(input_method::Event::Preedit(
        content.to_owned(),
        Some(0..content.len()),
    ))
}

fn commit(content: &str) -> Event {
    Event::InputMethod(input_method::Event::Commit(content.to_owned()))
}

/// Types `ㅊㅊ` with the Korean 2-set layout: the first `ㅊ` is committed
/// once the second one starts a new syllable, which stays in preedit.
fn compose_two_syllables() -> [Event; 3] {
    [
        Event::InputMethod(input_method::Event::Opened),
        commit("ㅊ"),
        preedit("ㅊ"),
    ]
}

#[test]
fn text_input_commits_preedit_when_clicking_outside() -> Result<(), Error> {
    let mut ui = simulator(column![
        text_input("", "").id("input").on_input(Message::Input),
        button("Send").on_press(Message::Send),
    ]);

    let _ = ui.click(Id::new("input"))?;
    let _ = ui.simulate(compose_two_syllables());
    let _ = ui.click("Send")?;

    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [
            Message::Input("ㅊ".to_owned()),
            Message::Input("ㅊㅊ".to_owned()),
            Message::Send,
        ]
    );

    Ok(())
}

#[test]
fn text_input_commits_preedit_when_escaping() -> Result<(), Error> {
    let mut ui =
        simulator(text_input("", "").id("input").on_input(Message::Input));

    let _ = ui.click(Id::new("input"))?;
    let _ = ui.simulate(compose_two_syllables());
    let _ = ui.tap_key(keyboard::Key::Named(key::Named::Escape));

    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [
            Message::Input("ㅊ".to_owned()),
            Message::Input("ㅊㅊ".to_owned()),
        ]
    );

    Ok(())
}

#[test]
fn text_input_ignores_cleared_preedit() -> Result<(), Error> {
    let mut ui = simulator(column![
        text_input("", "").id("input").on_input(Message::Input),
        button("Send").on_press(Message::Send),
    ]);

    let _ = ui.click(Id::new("input"))?;
    let _ = ui.simulate([
        Event::InputMethod(input_method::Event::Opened),
        preedit("ㅊ"),
        preedit(""),
    ]);
    let _ = ui.click("Send")?;

    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Send]);

    Ok(())
}

#[derive(Debug, Clone)]
enum EditorMessage {
    Edit(Action),
    Send,
}

#[test]
fn text_editor_commits_preedit_when_clicking_outside() -> Result<(), Error> {
    let content = Content::new();

    let mut ui = simulator(column![
        text_editor(&content)
            .id("editor")
            .on_action(EditorMessage::Edit),
        button("Send").on_press(EditorMessage::Send),
    ]);

    let _ = ui.click(Id::new("editor"))?;
    let _ = ui.simulate(compose_two_syllables());
    let _ = ui.click("Send")?;

    let pasted: Vec<String> = ui
        .into_messages()
        .filter_map(|message| match message {
            EditorMessage::Edit(Action::Edit(Edit::Paste(text))) => {
                Some(text.as_ref().clone())
            }
            EditorMessage::Send => Some("<send>".to_owned()),
            EditorMessage::Edit(_) => None,
        })
        .collect();

    assert_eq!(pasted, ["ㅊ", "ㅊ", "<send>"]);

    Ok(())
}
