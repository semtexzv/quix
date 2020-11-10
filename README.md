# Quix = Quic + Actix
Quic based transport for communicating between actors on different servers.

Aims to follow conventions set up by actix, and implement distribution capabilties, similar to erlangs [dist](https://erlang.org/doc/apps/erts/erl_dist_protocol.html).

### Messages
In order to pass messages between nodes, we implement simple serialization and deserialization scheme.
For now, we use JSON, however, this will be changed to a binary protocol.

### Processes
In order to better handle distribution, we will introduce a concent of a process, which is just an identified actor.
This actor is not referneced through the `Addr<A>` struct, but rather through `Pid<A>`, which internally: 
1. Wraps `Addr<A>` and transparently passes messages to this addr
2. Wraps `Remote<A>` and passes message to actor on remote node.

```rust
enum Pid<A> {
    // local addr 
    Local {
        addr: Addr<A>,
        id : Uuid,
        node_id: &'static Uuid
    },
    Remote {
        id: Uuid,
        node_id: Uuid,
        stream: Option<quinn::Stream>
    }
}
```


The `Pid<A>` can be obtained in 2 ways: 
1. `Registry::register(self)` - Registers actor in node-local registry.
2. `Handle<M> where M contains Pid<A2>` - Pids are transparent, and can be sent between nodes.
The distribution subsystem should handle node lookup internally, thorugh the node-local registry.


### ProcessRegistry
Is a node-local singleton, which contains a map of actors
```rust
struct ProcessRegistry {
    node_id: Uuid,
    actors: FlurryMap<Uuid, LocalProxy>
}
```
Each actor has to be proxied in order to perform message deserialization.
```rust
struct LocalProxy {
    handlers: HashMap<Symbol, Box<Fn(Arc<[u8]>)>>
}
```

We will have to implement method registration using codegen/tuple generics (Registration of `Handler<M>` for multiple `M`)

 