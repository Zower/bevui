use bevy_ecs::{entity::Entity, schedule::Schedule, world::World};
use std::time::Instant;
mod ui;

use ui::*;

fn main() {
    let now = Instant::now();

    init(root);

    println!("{:?}", now.elapsed());
}

fn root(ctx: &mut Builder) {
    ctx.under(|ctx| {
        ctx.under(|ctx| {
            println!("{ctx:?}");
            ctx.spawn(());
        });

        ctx.spawn(());
    })
    // let entity = world.spawn(()).id();

    // world.insert_resource(MyState {
    //     string: "bb2".into(),
    // });

    // let text = world.spawn(Text { text: "".into() }).id();

    // world.entity_mut(entity).insert((
    //     Button { text: "bb".into() },
    //     OnClick::new(move |state: State, mut texts: Texts| {
    //         texts.get_mut(text).unwrap().text = state.string.clone()
    //     }),
    // ));

    // do_thing(
    //     text,
    //     world,
    //     |text: &mut Text, mut state: Get<State>, mut writer: Get<EventWriter<MyEvent>>| {
    //         writer.send(MyEvent("a".into()));
    //         state.string = "abc".into();
    //         text.text = state.string.clone();
    //     },
    // );
}
