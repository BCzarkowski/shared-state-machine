use update::Updatable;
use ustack::UStack;

mod server;
mod update;
mod ustack;

#[tokio::main]
async fn main() {
    let mut ustack: UStack<i32> = UStack::new();
    let push_5 = ustack.push(5);
    let pop = ustack.pop();

    ustack.apply_update(push_5);
    println!("Current top: {}", ustack.top());
    let push_7 = ustack.push(7);
    println!("Current top: {}", ustack.top());
    ustack.apply_update(push_7);
    println!("Current top: {}", ustack.top());
    ustack.apply_update(pop);
    println!("Current top: {}", ustack.top());

    server::run_server().await;
}
