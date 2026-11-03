use dioxus::prelude::*;

mod map_interaction;
mod world_map;

fn main() {
    dioxus::LaunchBuilder::desktop().launch(app);
}

fn app() -> Element {
    let outlines = world_map::world_outlines();
    let mut viewport = use_signal(map_interaction::MapViewport::default);
    let stylesheet = include_str!("../assets/app.css");

    rsx! {
        style { "{stylesheet}" }

        div { class: "app",
            header { class: "hero",
                div {
                    h1 { "GRIB2Sail GUI" }
                    p { "MVP desktop shell with a local GeoJSON world outline map." }
                }
                div { class: "pill", "country outlines from bundled GeoJSON" }
            }

            div {
                class: "map-shell",
                onpointerdown: move |evt| {
                    let point = evt.element_coordinates();
                    viewport.write().pointer_down(evt.pointer_id(), &evt.pointer_type(), (point.x, point.y));
                },
                onpointermove: move |evt| {
                    let point = evt.element_coordinates();
                    viewport.write().pointer_move(evt.pointer_id(), &evt.pointer_type(), (point.x, point.y));
                },
                onpointerup: move |evt| {
                    viewport.write().pointer_up(evt.pointer_id(), &evt.pointer_type());
                },
                onpointercancel: move |evt| {
                    viewport.write().pointer_up(evt.pointer_id(), &evt.pointer_type());
                },
                onwheel: move |evt| {
                    let point = evt.element_coordinates();
                    let delta = evt.delta().strip_units();
                    viewport.write().zoom_wheel((point.x, point.y), delta.y);
                },
                style: "touch-action: none; overscroll-behavior: contain;",
                div {
                    class: "map-viewport",
                    style: "{viewport.read().transform_style()}",
                    svg {
                        class: "map",
                        view_box: "0 0 1000 500",
                        role: "img",
                        title { "World map with country outlines" },
                        for shape in outlines {
                            path {
                                key: "{shape.name}",
                                d: "{shape.d}",
                                class: "country"
                            }
                        }
                    }
                }
            }
        }
    }
}
