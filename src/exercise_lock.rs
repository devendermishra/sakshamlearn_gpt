use crate::{language::Language, progress::Input};
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct Controls(pub Signal<bool>);

pub fn locked() -> bool {
    try_consume_context::<Controls>().is_some_and(|controls| *controls.0.read())
}

// Capture the context during render, so callbacks do not depend on ambient scope.
pub fn guard<E: 'static>(mut action: impl FnMut(E) + 'static) -> impl FnMut(E) {
    let controls = try_consume_context::<Controls>();
    move |event| {
        if !controls.is_some_and(|controls| *controls.0.peek()) { action(event); }
    }
}

pub fn input_handler(action: EventHandler<Input>) -> EventHandler<Input> {
    let controls = try_consume_context::<Controls>();
    EventHandler::new(move |input: Input| {
        if input.is_draft() || !controls.is_some_and(|controls| *controls.0.peek()) {
            action.call(input);
        }
    })
}

#[component]
pub fn LockControl(language: Language, #[props(default)] compact: bool) -> Element {
    let Some(mut controls) = try_consume_context::<Controls>() else { return rsx! {}; };
    let locked = (controls.0)();
    rsx! {
        div { class: "exercise-lock",
            button { class: "quiet", aria_pressed: locked,
                onclick: move |_| controls.0.toggle(),
                {if locked { language.text("अभ्यास का ताला खोलें", "Unlock exercise") }
                 else { language.text("अभ्यास लॉक करें", "Lock exercise") }}
            }
            if locked && !compact { span { role: "status",
                {language.text("अभ्यास बदलना बंद है। बदलने के लिए ताला खोलें।", "Navigation is locked. Unlock to change exercise.")}
            } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::{Cell, RefCell}, rc::Rc};

    struct Fixture {
        controls: Controls,
        navigate: Box<dyn FnMut(())>,
        input: EventHandler<Input>,
        navigations: Rc<Cell<usize>>,
        inputs: Rc<Cell<usize>>,
    }
    #[derive(Clone, Props)]
    struct HarnessProps { shared: Rc<RefCell<Option<Fixture>>> }
    impl PartialEq for HarnessProps {
        fn eq(&self, other: &Self) -> bool { Rc::ptr_eq(&self.shared, &other.shared) }
    }
    fn harness(props: HarnessProps) -> Element {
        let signal = use_signal(|| false);
        let controls = use_context_provider(|| Controls(signal));
        let navigations = Rc::new(Cell::new(0));
        let inputs = Rc::new(Cell::new(0));
        let nav_counter = navigations.clone();
        let input_counter = inputs.clone();
        let navigate = Box::new(guard(move |()| nav_counter.set(nav_counter.get()+1)));
        let input = input_handler(EventHandler::new(move |_: Input| input_counter.set(input_counter.get()+1)));
        *props.shared.borrow_mut() = Some(Fixture { controls, navigate, input, navigations, inputs });
        rsx! { LockControl { language: Language::English } }
    }

    #[test]
    fn lock_guards_callbacks_and_keeps_drafts_without_queuing_advancement() {
        let shared = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new_with_props(harness, HarnessProps { shared: shared.clone() });
        dom.rebuild_in_place();
        let mut fixture = shared.borrow_mut().take().unwrap();
        assert!(!*fixture.controls.0.peek());
        (fixture.navigate)(());
        assert_eq!(fixture.navigations.get(), 1);
        fixture.controls.0.set(true);
        (fixture.navigate)(());
        fixture.input.call(Input::Answer(false));
        fixture.input.call(Input::Submit(vec![]));
        fixture.input.call(Input::SubmitSolution { ink: vec![], correct: true });
        assert_eq!(fixture.navigations.get(), 1);
        assert_eq!(fixture.inputs.get(), 0);
        fixture.input.call(Input::Draft(vec![]));
        fixture.input.call(Input::Helper(Default::default()));
        assert_eq!(fixture.inputs.get(), 2);
        fixture.controls.0.set(false);
        assert_eq!(fixture.navigations.get(), 1);
        assert_eq!(fixture.inputs.get(), 2);
        (fixture.navigate)(());
        fixture.input.call(Input::Answer(false));
        assert_eq!(fixture.navigations.get(), 2);
        assert_eq!(fixture.inputs.get(), 3);
    }
}
