use umap::UMap;
use update::Updatable;
use ustack::UStack;

mod umap;
mod update;
mod ustack;

fn main() {
    // let mut ustack: UStack<i32> = UStack::new();
    // let push_5 = ustack.push(5);
    // let pop = ustack.pop();

    // ustack.apply_update(push_5);
    // println!("Current top: {}", ustack.top());
    // let push_7 = ustack.push(7);
    // println!("Current top: {}", ustack.top());
    // ustack.apply_update(push_7);
    // println!("Current top: {}", ustack.top());
    // ustack.apply_update(pop);
    // println!("Current top: {}", ustack.top());

    let mut umap: UMap<String, i32> = UMap::new();
    let insert_5 = umap.insert(String::from("foo"), 5);
    let insert_7 = umap.insert(String::from("bar"), 7);
    let remove_5 = umap.remove(String::from("foo"));
    let remove_7 = umap.remove(String::from("bar"));

    assert_eq!(umap.get(&String::from("foo")), None);
    assert_eq!(umap.get(&String::from("bar")), None);
    umap.apply_update(insert_5);
    assert_eq!(umap.get(&String::from("foo")), Some(&5));
    assert_eq!(umap.get(&String::from("bar")), None);
    umap.apply_update(insert_7);
    assert_eq!(umap.get(&String::from("foo")), Some(&5));
    assert_eq!(umap.get(&String::from("bar")), Some(&7));
    umap.apply_update(remove_5);
    assert_eq!(umap.get(&String::from("foo")), None);
    assert_eq!(umap.get(&String::from("bar")), Some(&7));
    umap.apply_update(remove_7);
    assert_eq!(umap.get(&String::from("foo")), None);
    assert_eq!(umap.get(&String::from("bar")), None);
}
