# Quix
Distribution layer for Actix.
Aims to follow conventions set up by actix, and implement distribution capabilties, similar to erlangs [dist](https://erlang.org/doc/apps/erts/erl_dist_protocol.html).

## Usage 
```rust
#[derive(Debug, prost::Message)]
pub struct M1 {
    #[prost(int32, required, tag = "1")]
    v: i32
}

impl actix::Message for M1 {
    type Result = ();
}


#[derive(quix::ProcessDispatch)]
#[dispatch(M1)]
pub struct Act {}

impl Actor for Act {

    type Context = Process<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let pid : Pid<Self> = ctx.pid();
        pid.do_send(M1 { v: 0 });
    }
}
impl Handler<M1> for Act { ... }
```
### Messages
Both internal and user messages use Protobuf format for serialization

### Processes
In order to better handle distribution, we will introduce a concept of a process, which is just an identified actor.
This actor is not referneced through the `Addr<A>` struct, but rather through `Pid<A>`, which can be: 
1. `Pid::Local` Wraps `Addr<A>` and transparently passes messages to this addr
2. `Pid::Remote` contains global process ID, and uses local process registry to determine where to send the message

The `Pid<A>` can be obtained in 2 ways: 
1. Using `Process<A>` instead of `Context<A>` - New context type for actors, which have stable identity
2. Receiving a message containing a `Pid<A>` - Pids are transparent, and can be sent between nodes.
The distribution subsystem should handle node lookup internally, thorugh the node-local registry.

### ProcessRegistry
Is a node-local singleton, which contains a map of actors. Process is automatically registered and unregistered from this 
registry when started and stopping. The registry uses gossip protocol to get rough image of whole cluster.

### Message dispatching
Each message which is passable across network boundaries must be serializable using protobuf.

The `ProcessDispatch` trait is responsible for creating a `Dispatcher` which performs message serialization and
dispatching. It can be implemented by a proc macro:

```rust
#[derive(quix::ProcessDispatch)]
#[dispatch(M1, M2)]
pub struct Act {}

impl Actor for Act {
    type Context = Process<Self>;
}
impl Handler<M1> for Act { ... }
impl Handler<M2> for Act { ... }
impl Handler<M3> for Act { ... }
```

The `Act` actor can locally respond to `M1`,`M2` and `M3` but only respond to remotely submitted `M1` or `M2`