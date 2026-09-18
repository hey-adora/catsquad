use catsquad_log::prelude::*;
use leptos::{ev::EventDescriptor, prelude::*};
use std::{fmt::Debug, marker::PhantomData};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Element, HtmlElement, MutationObserver, MutationRecord};

#[derive(Clone, Copy)]
pub struct EventListener<T, F>
where
    T: EventDescriptor + Debug + 'static,
    F: FnMut(&mut EventListenerInner<T>, <T as EventDescriptor>::EventType) + Clone + 'static,
{
    pub callback: StoredValue<F, LocalStorage>,
    pub inner: StoredValue<EventListenerInner<T>, LocalStorage>,
}

#[derive(Clone)]
pub struct EventListenerInner<T>
where
    T: EventDescriptor + Debug + 'static,
{
    pub event: T,
    pub js: Option<EventListenerJs<T>>,
}

#[derive(Clone)]
pub struct EventListenerJs<T>
where
    T: EventDescriptor + Debug + 'static,
{
    pub js_elm: Element,
    pub js_value: JsValue,
    phantom: PhantomData<T>,
}

impl<T> EventListenerJs<T>
where
    T: EventDescriptor + Debug + 'static,
{
    pub fn new(elm: Element, value: JsValue) -> Self {
        Self {
            js_elm: elm,
            js_value: value,
            phantom: PhantomData,
        }
    }
}

pub mod event_listener_remove {
    use super::*;

    impl<T> EventListenerJs<T>
    where
        T: EventDescriptor + Debug + 'static,
    {
        pub fn remove(&self, event: &T) {
            let event_name = event.name();
            self.js_elm
                .remove_event_listener_with_callback(
                    &event_name,
                    self.js_value.as_ref().unchecked_ref(),
                )
                .unwrap();
        }
    }

    impl<T> EventListenerInner<T>
    where
        T: EventDescriptor + Debug + 'static,
    {
        pub fn remove(&mut self) {
            if let Some(ref js) = self.js {
                js.remove(&self.event);
            }
            self.js = None;
        }
    }

    impl<T, F> EventListener<T, F>
    where
        T: EventDescriptor + Debug + 'static,
        F: FnMut(&mut EventListenerInner<T>, <T as EventDescriptor>::EventType) + Clone + 'static,
    {
        pub fn remove(&self) {
            self.inner.update_value(|v| {
                v.remove();
            });
        }
    }
}

impl<T, F> EventListener<T, F>
where
    T: EventDescriptor + Debug + 'static,
    F: FnMut(&mut EventListenerInner<T>, <T as EventDescriptor>::EventType) + Clone + 'static,
{
    pub fn new(event: T, f: F) -> Self {
        Self {
            callback: StoredValue::new_local(f),
            inner: StoredValue::new_local(EventListenerInner { event, js: None }),
        }
    }

    pub fn is_set(&self) -> bool {
        self.inner.with_value(|v| v.js.is_some())
    }

    pub fn add<E>(&self, element: E)
    where
        E: JsCast + Clone + 'static + Into<Element>,
    {
        let node: Element = element.into();
        let inner = self.inner;
        let event = self.inner.with_value(|v| v.event.clone()).name();
        let callback = self.callback;

        let closure = Closure::<dyn FnMut(_)>::new(move |e: <T as EventDescriptor>::EventType| {
            inner.update_value(|inner| {
                (callback.get_value())(inner, e);
            });
        })
        .into_js_value();

        self.inner.update_value({
            let node = node.clone();
            let closure = closure.clone();
            |v| {
                v.js = Some(EventListenerJs::new(node, closure));
            }
        });

        node.add_event_listener_with_callback(&event, closure.as_ref().unchecked_ref())
            .unwrap();
    }
}
