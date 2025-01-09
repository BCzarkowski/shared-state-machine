# Shared State Machine

## Motivation
In distributed systems with regular deployment, maintaining a consistent shared data-structures among multiple clients is a common challenge.

## Objective
Enable multiple clients to share data-structures over the network, with the source-of-truth provided by an independent entity.
Clients can obtain the latest state of the shared data on demand, thus enabling them to restart/deploy freely without the need
to shutdown their services.
Provided library's interface is as similar as possible to the one of regular data-structures.

## Library interface
Users can create a shared data-structure using structures like `SMap<K, T>` from the `score` crate.
Each shared structure provides blocking methods modifying structure's state.
Data types that can be stored in such structures must implement trait `Updatable`
$-$ can be represented as a series of atomic changes (`Update`s) to an initial state.
Primitive types like `i32` have been provided with implementations of this trait.
In crate `ucore`, are structures like `UMap<K, T>`, which implement `Updatable` and can be used to
create nested structures like `SMap<i32, UVec<String>>`.

## Server details
Each data-structure is stored as a queue of atomic changes to it's state.
Each client (instance of a shared data-structure) subscribes to a group sharing the same structure.
The server can be perceived as a shared messaging queue, agnostic of it's contents.
Incoming changes are synchronized by the server - each client has to send the id of the latest received change.

## Contents
1. **Async messaging server**:
   - Serves multiple clients.
   - Manages groups and synchronization of incoming changes.
   - Broadcasts changes to connected clients.
2. **Updatable data-structures**:
   - `UMap`, `UVec` and `UStack` with essential methods for `Update` generation.
   - Convenient wrappers for operations on nested types.
   - `Update` sizes are proportional to the sizes of actual changes to the data-structure.
   - Data look-up runtime is comparable with the one of a regular data-structure.
3. **`Updatable` trait implementations for primitive types**:
   - Implemented using a macro, thus reducing boilerplate code.
4. **Synchronizable data-structures**:
   - `SMap`, `SVec` and `SStack` with essential methods for state modification.
5. **Unit-tests**:
   - Verified the implementation with unit tests.