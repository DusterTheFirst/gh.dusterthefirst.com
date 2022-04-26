use dioxus::{hooks::use_state, prelude::ScopeState};
use gloo_events::EventListener;
use web_sys::window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub scroll_y: i32,
    pub client_height: i32,
    pub client_width: i32,
}

pub fn use_viewport(cx: &ScopeState) -> &Viewport {
    let window = window().expect("window should always exist");
    let document = window.document().expect("document should always exist");
    let document_element = document
        .document_element()
        .expect("document element should always exist");

    let get_scroll_y = {
        let window = window.clone();

        move || {
            let scroll_y = window.scroll_y().expect("scroll_y should not error");
            assert_eq!(
                scroll_y.fract(),
                0.0,
                "scroll_y returned a fractional amount of pixels"
            );

            scroll_y as i32
        }
    };
    let get_client_size = move || {
        (
            document_element.client_width(),
            document_element.client_height(),
        )
    };

    let viewport = use_state(cx, || {
        let (client_width, client_height) = get_client_size();

        Viewport {
            scroll_y: get_scroll_y(),
            client_height,
            client_width,
        }
    });

    cx.use_hook(|_| {
        EventListener::new(&window, "resize", {
            let viewport = viewport.to_owned();
            move |_e| {
                viewport.modify(|viewport| {
                    let (client_width, client_height) = get_client_size();

                    Viewport {
                        client_height,
                        client_width,
                        ..*viewport
                    }
                })
            }
        })
        .forget();

        EventListener::new(&document, "scroll", {
            let viewport = viewport.to_owned();

            move |_e| {
                viewport.modify(|viewport| Viewport {
                    scroll_y: get_scroll_y(),
                    ..*viewport
                });
            }
        })
        .forget();
    });

    viewport.get()
}
