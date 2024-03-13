use std::{marker::PhantomData, time::Instant};

use bevy_ecs::{
    prelude::*,
    schedule::{self, ScheduleLabel},
    system::{BoxedSystem, SystemId, SystemParam},
    world,
};

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}
#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Hash, Debug, PartialEq, Eq, Clone, ScheduleLabel)]
struct MySchedule;

#[derive(Hash, Debug, PartialEq, Eq, Clone, ScheduleLabel)]
struct PreMain;

#[derive(Hash, Debug, PartialEq, Eq, Clone, ScheduleLabel)]
struct Clicked;
fn main() {
    let mut world = World::new();

    let mut schedule = Schedule::new(MySchedule);
    let mut schedule2 = Schedule::new(PreMain);
    let mut clickedsched = Schedule::new(Clicked);

    schedule.add_systems(my_system);
    schedule2.add_systems(post_update);
    clickedsched.add_systems(clickedfn);

    world.add_schedule(schedule);
    world.add_schedule(schedule2);
    world.add_schedule(clickedsched);

    let now = Instant::now();

    world.run_schedule(MySchedule);
    world.run_schedule(PreMain);
    world.run_schedule(Clicked);
    world.run_schedule(Clicked);
    world.run_schedule(Clicked);

    println!("{:?}", now.elapsed());

    // let entity = world
    //     .spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }))
    //     .id();
}

#[derive(Component)]
struct Button {
    text: String,
}

#[derive(Component)]
struct OnClick {
    system: Option<Box<dyn System<In = (), Out = ()>>>,
}

#[derive(Component)]
struct ActualOnClick {
    system: SystemId,
}

macro_rules! modify {
    ($entity:ident, |$x:ident: $xty: ty, $y: ident: $typ: ty| $a: block) => {
        #[allow(unused_mut)]
        move |mut $x: $xty, mut query: Query<$typ>| {
            #[allow(unused_mut)]
            let mut $y = query.get_mut($entity).unwrap();
            $a
        }
    };
}

type State<'w> = ResMut<'w, MyState>;

#[derive(Resource)]
struct MyState {
    string: String,
}

fn my_system(world: &mut World) {
    let entity = world.spawn(()).id();

    world.insert_resource(MyState {
        string: "bb2".into(),
    });

    world.entity_mut(entity).insert((
        Button { text: "bb".into() },
        on_click(modify!(entity, |state: State, button: &mut Button| {
            println!("{}", state.string);
            button.text = state.string.clone();
        })),
    ));

    // on_update(
    //     world,
    //     modify!(entity, |reader: EventReader<MyEvent>,
    //                      button: &mut Button| {
    //         for event in reader.read() {
    //             button.text = event.0.clone();
    //         }
    //     }),
    // );
}

fn on_click<M>(f: impl IntoSystem<(), (), M>) -> OnClick {
    OnClick {
        system: Some(Box::new(IntoSystem::into_system(f))),
    }
}

#[derive(Event)]
struct MyEvent(String);

fn on_update<M>(world: &mut World, f: impl IntoSystemConfigs<M>) {
    world.schedule_scope(Clicked, |_, schedule| {
        schedule.add_systems(f);
    });
}

fn post_update(world: &mut World) {
    let mut vec = Vec::new();
    for click in world.query::<(Entity, &OnClick)>().iter(world) {
        vec.push(click.0);
    }

    for entity in vec {
        let on_click = world
            .entity_mut(entity)
            .get_mut::<OnClick>()
            .unwrap()
            .system
            .take()
            .unwrap();

        world.entity_mut(entity).remove::<OnClick>();

        let id = world.register_boxed_system(on_click);

        world
            .entity_mut(entity)
            .insert(ActualOnClick { system: id });
    }
}

fn clickedfn(mut commands: Commands, q: Query<&ActualOnClick>) {
    for q in q.iter() {
        commands.run_system(q.system);
    }
}
