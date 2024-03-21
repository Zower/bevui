use bevy_ecs::{
    prelude::*,
    schedule::ScheduleLabel,
    system::{StaticSystemParam, SystemId, SystemParam},
};

use ahash::HashMap;

pub fn init(f: impl Fn(&mut Builder)) {
    let mut context = Context::new();

    context.under_root(f);

    let mut pre_update_sched = Schedule::new(PreUpdate);
    let mut update = Schedule::new(Update);

    pre_update_sched.add_systems(pre_update);
    update.add_systems(clickedfn);

    context.world.add_schedule(pre_update_sched);
    context.world.add_schedule(update);

    context.world.run_schedule(PreUpdate);
    context.world.run_schedule(Update);
    context.world.run_schedule(Update);
    context.world.run_schedule(Update);
}

#[derive(Debug)]
pub struct Context {
    root: Entity,
    parents: HashMap<Entity, Entity>,
    children: HashMap<Entity, Vec<Entity>>,
    world: World,
}

impl Context {
    pub fn new() -> Self {
        let mut world = World::new();

        let root = world.spawn(()).id();

        let mut context = Context {
            root,
            world,
            parents: Default::default(),
            children: Default::default(),
        };

        context.parents.insert(root, root);

        context
    }

    fn under_root(&mut self, f: impl Fn(&mut Builder)) {
        f(&mut Builder {
            current: self.root,
            context: self,
        })
    }
}

#[derive(Debug)]
pub struct Builder<'c> {
    current: Entity,
    context: &'c mut Context,
}

impl<'c> Builder<'c> {
    pub fn under(&mut self, f: impl Fn(&mut Builder)) {
        f(&mut Builder {
            current: self.context.world.spawn_empty().id(),
            context: self.context,
        })
    }

    pub fn spawn(&mut self, b: impl Bundle) {
        let id = self.context.world.spawn(b).id();

        self.context.parents.insert(id, self.current);

        self.context
            .children
            .get_mut(&self.current)
            .unwrap()
            .push(id);
    }
}

pub mod widgets {
    use bevy_ecs::component::Component;

    #[derive(Component)]
    pub struct Button {
        pub text: String,
    }

    #[derive(Component)]
    pub struct Text {
        pub text: String,
    }

    mod aliases {
        use bevy_ecs::system::Query;

        pub type Texts<'w, 's> = Query<'w, 's, &'static mut super::Text>;
    }
}

#[derive(Hash, Debug, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct PreUpdate;

#[derive(Hash, Debug, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct Update;

#[derive(Component)]
pub struct OnClick {
    system: Option<Box<dyn System<In = (), Out = ()>>>,
}

impl OnClick {
    pub fn new<M>(s: impl IntoSystem<(), (), M>) -> Self {
        Self {
            system: Some(Box::new(IntoSystem::into_system(s))),
        }
    }
}

#[derive(Component)]
struct ActualOnClick {
    system: SystemId,
}

type State<'w> = ResMut<'w, MyState>;

#[derive(Resource)]
struct MyState {
    string: String,
}

pub type Get<'w, 's, S> = StaticSystemParam<'w, 's, S>;

pub fn do_thing<T: SystemParam + 'static, C: Component>(
    entity: Entity,
    world: &mut World,
    // f: F,
    f: impl SingleComponentExtractor<C, T>, // f: impl Fn(&mut C, StaticSystemParam<T>) + Send + Sync + 'static,
) {
    f.run(entity, world);
    // world.run_system_once(move |mut q: Query<&mut C>, mut t: StaticSystemParam<T>| {
    //     // f(&mut *q.get_mut(entity).unwrap(), t)
    // })
}

pub trait SingleComponentExtractor<C, S>: Send + Sync + 'static {
    type C;
    type S;

    fn run(self, entity: Entity, world: &mut World);
}

#[derive(Event)]
struct MyEvent(String);

fn on_update<M>(world: &mut World, f: impl IntoSystemConfigs<M>) {
    world.schedule_scope(Update, |_, schedule| {
        schedule.add_systems(f);
    });
}

fn pre_update(world: &mut World) {
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

impl<
        C: Component,
        S: SystemParam + 'static,
        F: Fn(&mut C, StaticSystemParam<S>) + Send + Sync + 'static,
    > SingleComponentExtractor<C, (S,)> for F
{
    type C = C;
    type S = S;

    fn run(self, entity: Entity, world: &mut World) {
        on_update(
            world,
            move |mut q: Query<&mut C>, t: StaticSystemParam<S>| {
                self(&mut *q.get_mut(entity).unwrap(), t)
            },
        );
    }
}

impl<
        C: Component,
        S: SystemParam + 'static,
        S1: SystemParam + 'static,
        F: Fn(&mut C, StaticSystemParam<S>, StaticSystemParam<S1>) + Send + Sync + 'static,
    > SingleComponentExtractor<C, (S, S1)> for F
{
    type S = (S, S1);
    type C = C;

    fn run(self, entity: Entity, world: &mut World) {
        on_update(
            world,
            move |mut q: Query<&mut C>, t: StaticSystemParam<S>, b: StaticSystemParam<S1>| {
                self(&mut *q.get_mut(entity).unwrap(), t, b)
            },
        );
    }
}
