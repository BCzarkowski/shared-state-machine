# Shared State Machine

## Motivation
In distributed systems, maintaining a consistent shared state among multiple clients is a common challenge.

## Objective
Enable multiple clients to share a single state structure.

## Features
1. A centralized server manages multiple clients simultaneously.
2. Each structure is modified through a stream of events/updates, ensuring consistency.
3. Clients can subscribe to the server's update queue.
4. From the client's perspective, the exposed state structure behaves like a typical data structure.

## Implementation details
1. **State Modification and Broadcasting**:
   - Before executing an operation that modifies the state, the structure sends an update to the server.
   - The server broadcasts this update to all other clients.
2. **Trait Implementation**:
   - Trait `Updatable` represents updatable structures.
   - Implemented for basic data structures like `Umap`, `Ustack`, and `Uvec`.
3. **Macro Derivation**:
   - Reducing boilerplate code by deriving the `Updatable` trait for simple data types,
4. **Testing**:
   - Verified the implementation with unit tests.
5. **Server Functionality**:
   - Created an initial server capable of handling multiple client connections.
   - Implemented broadcasting of updates to all connected clients.
