use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, Storable,
};
use std::{borrow::Cow, cell::RefCell};
use ic_stable_structures::storable::Bound;

// Define the memory type
type Memory = VirtualMemory<DefaultMemoryImpl>;

// Define the Task struct
#[derive(CandidType, Deserialize, Clone)]
pub struct Task {
    title: String,
    completed: bool,
    important: bool,
    // due_date: String, // Store date as a String ("YYYY-MM-DD")
}

// Implement Storable trait for Task
impl Storable for Task {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap()) // Serialize to bytes
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(&bytes.as_ref(), Self).unwrap() // Deserialize from bytes
    }
}

impl Task {
    pub fn new(title: String, completed: bool, important: bool) -> Self {
        Self {
            title,
            completed,
            important,
            // due_date: due_date.format("%Y-%m-%d").to_string(), // Convert date to string
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TaskList {
    tasks: Vec<Task>,
}

// Implement Storable for TaskList (a collection of tasks)
impl Storable for TaskList {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap()) // Serialize the entire TaskList to bytes
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(&bytes.as_ref(), TaskList).unwrap() // Deserialize into a TaskList
    }
}

impl TaskList {
    // A helper method to create a new TaskList
    pub fn new() -> Self {
        TaskList {
            tasks: Vec::new(),
        }
    }

    // Method to add a new task to the TaskList
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    // Method to retrieve all tasks in the TaskList
    pub fn get_tasks(&self) -> &Vec<Task> {
        &self.tasks
    }
}

// Thread-local storage for memory manager
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // A stable BTreeMap to store a TaskList for each Principal (multiple tasks per principal)
    pub static TASK_STORAGE: RefCell<StableBTreeMap<Principal, TaskList, Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
    ));
}
#[ic_cdk::update]
pub fn create_task( title: String, completed: bool, important: bool) {
    let principal: Principal = ic_cdk::api::caller();
    let new_task = Task::new(title, completed, important);

    TASK_STORAGE.with(|map| {
        let mut storage = map.borrow_mut();

        // Check if the TaskList exists for the given Principal
        match storage.get(&principal) {
            Some(mut task_list) => {
                // If it exists, add the new task
                task_list.add_task(new_task);
                storage.insert(principal, task_list); // Update the TaskList in storage
            }
            None => {
                // If not, create a new TaskList, add the task, and insert it
                let mut task_list = TaskList::new();
                task_list.add_task(new_task);
                storage.insert(principal, task_list); // Insert the new TaskList into the map
            }
        }
    });
}

// Function to retrieve all tasks for a specific Principal
#[ic_cdk::query]
pub fn get_tasks() -> Option<Vec<Task>> {
    let principal: Principal = ic_cdk::api::caller();
    TASK_STORAGE.with(|map| {
        let map = map.borrow();
        map.get(&principal).map(|task_list| task_list.tasks.clone()) // Return a cloned Vec<Task> if found
    })
}

ic_cdk::export_candid!();
