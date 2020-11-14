# Quix
Distribution layer for Actix.
Aims to follow conventions set up by actix, and implement distribution capabilities, similar to erlang's [dist](https://erlang.org/doc/apps/erts/erl_dist_protocol.html).

## Usage 
See [docs](https://docs.rs/quix)


Protobuf:
```protobuf
message M1 {

}
service Exec{
  rpc Method(M1) returns(M1);
}
```

Code:
```rust
#[derive(quix::ProcessDispatch)]
#[dispatch(Method)]
pub struct Act {}

impl Actor for Act {

    type Context = Process<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let pid : Pid<Self> = ctx.pid();
        pid.do_send(M1 { v: 0 });
    }
}
impl Handler<Method> for Act { 
    type Response = Result<M1, DispatchError>;
... 
}
```
### Messages
Both internal and user messages use Protobuf format for serialization. The indivudal `Messages` from protobuf just passed to
prost.

We use the `RPC` entry in protobuf services as a unit of communication. Each RPC generates a single struct, which implements
`Service` and `actix::Message`. The user needs to:
1. Implement `Handler<M>` for RPCs he wants to consume using the specified `Process` actor
2. Derive `ProcessDispatch` on the actor with dispatch containing the RPC struct

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