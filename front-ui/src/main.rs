use leptos::{
    ev,
    html::{button, div, span},
    prelude::*,
};

fn main() {
    mount_to_body(|| counter(0));
}

fn counter(initial_value: i32) -> impl IntoView {
    let (count, set_count) = signal(initial_value);

    div().child((
        button()
            .on(ev::click, move |_| set_count.set(0))
            .child("Clear"),
        button()
            .on(ev::click, move |_| *set_count.write() -= 1)
            .child("-1"),
        span().child(("Value: ", count, "!")),
        button()
            .on(ev::click, move |_| *set_count.write() += 1)
            .child("+1"),
    ))
}
