#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use std::sync::{Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::any::{TypeId, Any};

#[macro_export]
macro_rules! subscribe_event {
    ($b:expr, $h:expr) => {
        $crate::subscribe_event($b, $h, 0)
    };
    ($b:expr, $h:expr, $p:expr) => {
        $crate::subscribe_event($b, $h, $p)
    };
}

#[macro_export]
macro_rules! dispatch_event {
    ($b:expr, $e:expr) => {
        $crate::dispatch_event($b, $e)
    };
}

lazy_static! {
    static ref EVENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static ref EVENT_ID_MAP: Mutex<HashMap<TypeId, usize>> = Mutex::new(HashMap::new());

    // Map (bus name, set of handlers + priority)
    static ref EVENT_HANDLER_MAP: RwLock<HashMap<
        String, // bus name
        RwLock<HashMap<
            usize, // event id
            Box<dyn Any + Send + Sync + 'static>, // event handlers
        >>
    >> = RwLock::new(HashMap::new());
}

pub trait Event: 'static {
    fn cancellable(&self) -> bool {
        false
    }

    fn cancelled(&self) -> bool {
        false
    }

    fn set_cancelled(&mut self, _cancel: bool) {
        panic!("Cannot cancel event that is not cancellable!");
    }

    fn cancel(&mut self) {
        self.set_cancelled(true);
    }
}

pub struct EventBus {
    name: String,
}

struct EventHandlers<T: Event>(Vec<(usize, Box<dyn Fn(&mut T) + Send + Sync + 'static>)>);

impl<T: Event> Default for EventHandlers<T> {
    fn default() -> Self {
        EventHandlers(vec![])
    }
}

impl EventBus {
    pub fn new<S: Into<String>>(name: S) -> EventBus {
        let name = name.into();
        let mut map = EVENT_HANDLER_MAP.write()
            .expect("Failed to get write guard on handler map");

        // ensure bus doesn't already exist
        if map.contains_key(&name) {
            panic!("Event bus named '{}' already exists!", name);
        }

        // insert bus into handlers map
        map.entry(name.clone()).or_insert_with(||
            RwLock::new(HashMap::new())
        );

        EventBus { name }
    }
}

impl Drop for EventBus {
    fn drop(&mut self) {
        EVENT_HANDLER_MAP.write()
            .expect("Failed to get write guard on handler map")
            .remove(&self.name);
    }
}

pub fn dispatch_event<T: Event>(bus: &str, event: &mut T) {
    let event_id = get_event_id::<T>();
    let map = EVENT_HANDLER_MAP.read()
        .expect("Failed to get read guard on handler map");

    if map.contains_key(bus) {
        let event_id_map = map.get(bus).unwrap()
            .read().expect("Failed to get read guard on event id map");

        if let Some(handlers) = event_id_map.get(&event_id) {
            let handlers = handlers.downcast_ref::<EventHandlers<T>>().unwrap();
            let cancellable = event.cancellable();

            for handler in handlers.0.iter().rev() {
                handler.1(event);

                if cancellable && event.cancelled() {
                    break;
                }
            }
        }
    } else {
        warn!("Cannot dispatch event on invalid bus: '{}'", bus);
    }
}

pub fn subscribe_event<T: Event, H: Fn(&mut T) + Send + Sync + 'static>(bus: &str, handler: H, priority: usize) {
    let event_id = get_event_id::<T>();
    let map = EVENT_HANDLER_MAP.read()
        .expect("Failed to get read guard on handler map");

    if map.contains_key(bus) {
        let mut event_id_map = map.get(bus).unwrap()
            .write().expect("Failed to get write guard on event id map");

        let handlers = event_id_map
            .entry(event_id)
            .or_insert(Box::new(EventHandlers::<T>::default()))
            .downcast_mut::<EventHandlers<T>>()
            .unwrap();

            // et pos = match vec.binary_search_by(|probe| probe.0.cmp(&priority)) { Ok(p) => p, Err(p) => p };
        let pos = match handlers.0.binary_search_by(|probe| probe.0.cmp(&priority)) {
            Ok(pos) => pos,
            Err(pos) => pos,
        };

        handlers.0.insert(pos, (priority, Box::new(handler)));
    } else {
        warn!("Cannot subscribe on invalid bus: '{}'", bus);
    }
}

fn get_event_id<T: Event>() -> usize {
    *EVENT_ID_MAP.lock()
        .expect("Failed to lock event id map")
        .entry(TypeId::of::<T>()).or_insert_with(||
            EVENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
        )
}