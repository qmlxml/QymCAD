//! THE DOCUMENT'S PROPERTIES ARE TYPED AS ONE STEP OF UNDO, NAMED. The title, author, version and comment used to be
//! written straight into the document, outside any edit: a title typed and left with Escape made no step of undo at
//! all, so Ctrl+Z could not take it back. (Follow-up of issue #145's fix, which put the fields into the key that says the
//! document changed.)
#[cfg(test)]
mod tests {
    use super::super::import_door::tests::{click, frame, key, running};
    use crate::gui::App;
    use qymcad_ui_state::WinKind;

    /// The window open, and the place of the title's field: right of its label, on its row.
    fn open_the_properties(app: &mut App, ctx: &egui::Context) -> egui::Pos2 {
        app.win.open(WinKind::DocProps);
        let _ = frame(app, ctx, Vec::new());
        // the second frame's layout: a window can move after its first
        let texts = frame(app, ctx, Vec::new());
        let label = texts.iter().find(|(t, _)| *t == crate::i18n::tr("doc-props-name")).map(|(_, r)| *r).expect("the title's label");
        egui::pos2(label.right() + 120.0, label.center().y)
    }

    /// Typed into, then left: the title is the new one, there is one more step of undo, named for the window, and undo
    /// brings the old title back.
    #[test]
    fn a_title_typed_is_one_named_step() {
        let (mut app, ctx) = running();
        let field = open_the_properties(&mut app, &ctx);
        let steps = app.disk.edits.undo.len();
        let old = app.project.meta.title.clone();
        let _ = frame(&mut app, &ctx, click(field));
        let _ = frame(&mut app, &ctx, vec![egui::Event::Text("Bracket".into())]);
        let _ = frame(&mut app, &ctx, key(egui::Key::Escape));
        for _ in 0..3 {
            let _ = frame(&mut app, &ctx, Vec::new());
        }
        assert_eq!(app.project.meta.title, format!("{old}Bracket"), "the typing did not reach the title");
        assert_eq!(app.disk.edits.undo.len(), steps + 1, "the typing is not one step of undo");
        assert_eq!(app.disk.edits.undo.last().map(|s| s.name.clone()), Some(crate::i18n::tr("doc-props-title")), "the step is not named for the window");
        assert!(app.disk.edits.open.is_none(), "an edit was left open");
        app.undo();
        assert_eq!(app.project.meta.title, old, "undo did not bring the old title back");
    }

    /// The window closed while a field still holds the keyboard - no field reports losing it: the edit is still closed,
    /// as one step, and nothing is left open to hold the next operation back.
    #[test]
    fn closing_the_window_while_typing_closes_the_edit() {
        let (mut app, ctx) = running();
        let field = open_the_properties(&mut app, &ctx);
        let steps = app.disk.edits.undo.len();
        let _ = frame(&mut app, &ctx, click(field));
        let _ = frame(&mut app, &ctx, vec![egui::Event::Text("Bracket".into())]);
        app.win.close(WinKind::DocProps);
        for _ in 0..3 {
            let _ = frame(&mut app, &ctx, Vec::new());
        }
        assert!(app.disk.edits.open.is_none(), "closing the window left the edit open");
        assert_eq!(app.disk.edits.undo.len(), steps + 1, "the typing is not one step of undo");
        assert!(app.project.meta.title.ends_with("Bracket"), "the typing was lost");
    }

    /// A field only looked into - focused and left with nothing typed - makes no step.
    #[test]
    fn a_field_left_unchanged_makes_no_step() {
        let (mut app, ctx) = running();
        let field = open_the_properties(&mut app, &ctx);
        let steps = app.disk.edits.undo.len();
        let _ = frame(&mut app, &ctx, click(field));
        assert!(ctx.memory(|m| m.focused().is_some()), "setup: the click did not reach the field");
        let _ = frame(&mut app, &ctx, key(egui::Key::Escape));
        for _ in 0..3 {
            let _ = frame(&mut app, &ctx, Vec::new());
        }
        assert!(app.disk.edits.open.is_none(), "an edit was left open");
        assert_eq!(app.disk.edits.undo.len(), steps, "a field left unchanged made a step of undo");
    }

    /// Typing in two fields in turn - Tab from the title to the author - is one step, and undo takes both back.
    #[test]
    fn two_fields_in_turn_are_one_step() {
        let (mut app, ctx) = running();
        let field = open_the_properties(&mut app, &ctx);
        let steps = app.disk.edits.undo.len();
        let (title, author) = (app.project.meta.title.clone(), app.project.meta.author.clone());
        let _ = frame(&mut app, &ctx, click(field));
        let _ = frame(&mut app, &ctx, vec![egui::Event::Text("Bracket".into())]);
        let _ = frame(&mut app, &ctx, key(egui::Key::Tab));
        let _ = frame(&mut app, &ctx, vec![egui::Event::Text("Somebody".into())]);
        let _ = frame(&mut app, &ctx, key(egui::Key::Escape));
        for _ in 0..3 {
            let _ = frame(&mut app, &ctx, Vec::new());
        }
        assert!(
            app.project.meta.title.ends_with("Bracket") && app.project.meta.author.ends_with("Somebody"),
            "the typing did not reach both fields: {:?}, {:?}",
            app.project.meta.title,
            app.project.meta.author
        );
        assert_eq!(app.disk.edits.undo.len(), steps + 1, "two fields typed in turn are not one step");
        app.undo();
        assert_eq!((app.project.meta.title.clone(), app.project.meta.author.clone()), (title, author), "undo did not take both back");
    }

    /// A press outside the window while a field holds the keyboard - the start of a drag in the viewport - ends the
    /// typing there and then: the step is closed, so a part moved by that drag is a step of its own, not inside this one.
    #[test]
    fn a_press_outside_the_window_ends_the_typing() {
        let (mut app, ctx) = running();
        let field = open_the_properties(&mut app, &ctx);
        let steps = app.disk.edits.undo.len();
        let _ = frame(&mut app, &ctx, click(field));
        let _ = frame(&mut app, &ctx, vec![egui::Event::Text("Bracket".into())]);
        // pressed, not released: a drag beginning far from the window
        let far = egui::pos2(20.0, 700.0);
        let _ =
            frame(&mut app, &ctx, vec![egui::Event::PointerMoved(far), egui::Event::PointerButton { pos: far, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() }]);
        assert!(!app.win.props_typing, "the press outside did not end the typing");
        assert_eq!(app.disk.edits.undo.len(), steps + 1, "the typing before the press is not its own step");
        assert_eq!(app.disk.edits.undo.last().map(|s| s.name.clone()), Some(crate::i18n::tr("doc-props-title")));
        assert!(ctx.memory(|m| m.focused().is_none()), "the field still holds the keyboard");
    }

    /// A document opened or begun while a field held the keyboard resets the edits but not the window's mark that it is
    /// typing: the next typing must still be a step, not run outside any edit.
    #[test]
    fn a_mark_left_from_another_document_does_not_stop_the_step() {
        let (mut app, ctx) = running();
        let field = open_the_properties(&mut app, &ctx);
        // as a document replaced under the window leaves it: the mark set, no edit open
        app.win.props_typing = true;
        assert!(app.disk.edits.open.is_none());
        let steps = app.disk.edits.undo.len();
        let _ = frame(&mut app, &ctx, click(field));
        let _ = frame(&mut app, &ctx, vec![egui::Event::Text("Bracket".into())]);
        let _ = frame(&mut app, &ctx, key(egui::Key::Escape));
        for _ in 0..3 {
            let _ = frame(&mut app, &ctx, Vec::new());
        }
        assert_eq!(app.disk.edits.undo.len(), steps + 1, "the typing after a stale mark is not a step");
    }
}
